use bollard::{
    container::{
        Config, CreateContainerOptions, InspectContainerOptions, ListContainersOptions,
        LogsOptions, RemoveContainerOptions, StartContainerOptions,
    },
    errors::Error,
    exec::{CreateExecOptions, StartExecResults},
    image::{CreateImageOptions, ListImagesOptions},
    secret::{ContainerStateStatusEnum, ContainerSummary, PortBinding},
    Docker,
};
use futures::{Stream, StreamExt};
use std::{collections::HashMap, pin::Pin};
use tauri::Manager;

use crate::{app_handle, gpt_sovits_model_dir, user_files_dir};

/// 创建并启动一个 ffmpeg 容器，执行指定的命令
pub async fn create_and_run_ffmpeg_container(cmd: Vec<&str>) -> Result<(), Error> {
    let docker = get_docker().await?;

    if !is_image_exists("ruobinqaq/ffmpeg:latest").await? {
        pull_image("ruobinqaq/ffmpeg:latest", "gpt_sovits_api_log").await?;
    }

    if let Ok(_) = get_container_by_name("ffmpeg").await {
        remove_container("ffmpeg").await?;
    }

    let options = Some(CreateContainerOptions::<&str> {
        name: "ffmpeg",
        platform: None,
    });

    let mount_path = user_files_dir().to_string_lossy().to_string() + ":/workspace";

    let config = Config::<&str> {
        image: Some("ruobinqaq/ffmpeg:latest"),
        cmd: Some(cmd),
        host_config: Some(bollard::service::HostConfig {
            binds: Some(vec![mount_path]),
            ..Default::default()
        }),
        ..Default::default()
    };

    docker.create_container(options, config).await?;

    docker
        .start_container("ffmpeg", None::<StartContainerOptions<&str>>)
        .await?;

    get_container_logs("ffmpeg", "gpt_sovits_api_log").await?;

    Ok(())
}

// 开启gpt-sovits
pub async fn start_gpt_sovits_container() -> Result<(), Error> {
    let docker = get_docker().await?;

    match docker
        .start_container("gpt-sovits", None::<StartContainerOptions<&str>>)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => match e {
            Error::DockerResponseServerError { status_code, .. } => {
                if status_code == 404 {
                    create_gpt_sovits().await?;
                    Box::pin(start_gpt_sovits_container()).await
                } else {
                    Err(e)
                }
            }
            _ => Err(e),
        },
    }
}

// 开启gpt-sovits API服务
pub async fn start_gpt_sovits_api() -> Result<(), Error> {
    start_gpt_sovits_container().await?;

    let cmd = vec![
        "python",
        "api.py",
        "-dr",
        "gpt_sovits_model/syq/我整理完今天的照片就休息了，你也别熬夜打游戏哦。.wav",
        "-dt",
        "我整理完今天的照片就休息了，你也别熬夜打游戏哦。",
        "-dl",
        "zh",
        "-d",
        "cuda",
        "-s",
        "gpt_sovits_model/syq/sanyueqi_e15_s180.pth",
        "-g",
        "gpt_sovits_model/syq/sanyueqi-e15.ckpt",
    ];

    let mut stream = run_command_in_container("gpt-sovits", cmd).await?;

    while let Some(result) = stream.next().await {
        match result {
            Ok(output) => {
                app_handle().emit("gpt_sovits_api_log", &output).unwrap();
                if output.contains("Press CTRL+C to quit") {
                    app_handle().emit("gpt_sovits_api_running", true).unwrap();
                }
            }
            Err(e) => {
                app_handle().emit("gpt_sovits_api_running", false).unwrap();
                return Err(e);
            }
        }
    }

    app_handle().emit("gpt_sovits_api_running", false).unwrap();
    Ok(())
}

// 创建gpt-sovits容器
pub async fn create_gpt_sovits() -> Result<(), Error> {
    let docker = get_docker().await?;

    if !is_image_exists("breakstring/gpt-sovits:latest").await? {
        pull_image("breakstring/gpt-sovits:latest", "gpt_sovits_api_log").await?;
    }

    let options = Some(CreateContainerOptions::<&str> {
        name: "gpt-sovits",
        platform: None,
    });

    let mut port_bindings: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();
    port_bindings.insert(
        "9880/tcp".to_string(),
        Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some("9880".to_string()),
        }]),
    );
    port_bindings.insert(
        "9871/tcp".to_string(),
        Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some("9871".to_string()),
        }]),
    );
    port_bindings.insert(
        "9872/tcp".to_string(),
        Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some("9872".to_string()),
        }]),
    );
    port_bindings.insert(
        "9873/tcp".to_string(),
        Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some("9873".to_string()),
        }]),
    );
    port_bindings.insert(
        "9874/tcp".to_string(),
        Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some("9874".to_string()),
        }]),
    );

    let mount_path =
        gpt_sovits_model_dir().to_string_lossy().to_string() + ":/workspace/gpt_sovits_model";

    let config = Config::<&str> {
        image: Some("breakstring/gpt-sovits:latest"),
        host_config: Some(bollard::service::HostConfig {
            port_bindings: Some(port_bindings),
            binds: Some(vec![mount_path]),
            shm_size: Some(16 * 1024 * 1024 * 1024), // 16G
            device_requests: Some(vec![bollard::models::DeviceRequest {
                driver: Some("nvidia".to_string()),
                count: Some(-1),
                capabilities: Some(vec![vec!["gpu".to_string()]]),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        ..Default::default()
    };
    docker.create_container(options, config).await?;
    Ok(())
}

// 运行命令在容器中
async fn run_command_in_container(
    container_name: &str,
    cmd: Vec<&str>,
) -> Result<Pin<Box<dyn Stream<Item = Result<String, Error>> + Send>>, Error> {
    let docker = get_docker().await?;

    let exec_instance = docker
        .create_exec(
            container_name,
            CreateExecOptions {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                cmd: Some(cmd),
                ..Default::default()
            },
        )
        .await?;

    let start_result = docker.start_exec(&exec_instance.id, None).await?;

    match start_result {
        StartExecResults::Attached { output, .. } => {
            let stream = output.map(|log_output| match log_output {
                Ok(log_output) => Ok(format!("{}", log_output)),
                Err(e) => Err(e),
            });
            Ok(stream.boxed())
        }
        StartExecResults::Detached => Err(Error::DockerResponseServerError {
            status_code: 500,
            message: "命令已执行无返回值".to_string(),
        }),
    }
}

// 判断容器是否运行中
pub async fn is_container_running(container_name: &str) -> Result<bool, Error> {
    let container = get_container_by_name(container_name).await?;

    let docker = get_docker().await?;
    let inspect_info = docker
        .inspect_container(
            &container.id.as_ref().unwrap(),
            None::<InspectContainerOptions>,
        )
        .await?;

    let state = inspect_info.state;

    match state {
        Some(state) if state.status == Some(ContainerStateStatusEnum::RUNNING) => Ok(true),
        _ => Ok(false),
    }
}

// 判断镜像是否存在
pub async fn is_image_exists(image_name: &str) -> Result<bool, Error> {
    let docker = get_docker().await?;

    let mut filters = HashMap::new();
    filters.insert("reference".to_string(), vec![image_name.to_string()]);

    let options = ListImagesOptions::<String> {
        all: true,
        filters,
        digests: false,
    };

    let images = docker.list_images(Some(options)).await?;

    if !images.is_empty() {
        return Ok(true);
    }

    Ok(false)
}

// 拉取镜像
pub async fn pull_image(image_name: &str, emit_name: &str) -> Result<(), Error> {
    let docker = get_docker().await?;

    let create_options = CreateImageOptions {
        from_image: image_name,
        ..Default::default()
    };

    let mut stream = docker.create_image(Some(create_options), None, None);

    while let Some(result) = stream.next().await {
        match result {
            Ok(info) => {
                if let Some(progress_detail) = info.progress_detail {
                    let progress = format!(
                        "{}/{}",
                        progress_detail.current.unwrap_or(-1),
                        progress_detail.total.unwrap_or(-1)
                    );
                    println!("{}", progress);
                    app_handle().emit(emit_name, &progress).unwrap();
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(())
}

// 获取docker
pub async fn get_docker() -> Result<Docker, Error> {
    match Docker::connect_with_socket_defaults() {
        Ok(docker) => match docker.ping().await {
            Ok(_) => Ok(docker),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

// 根据名称获取容器
#[allow(dead_code)]
pub async fn get_container_by_name(container_name: &str) -> Result<ContainerSummary, Error> {
    let docker = get_docker().await?;

    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await?;

    for container in containers {
        if let Some(names) = &container.names {
            if names
                .iter()
                .any(|name| name == format!("/{container_name}").as_str())
            {
                return Ok(container);
            }
        }
    }

    Err(Error::DockerResponseServerError {
        status_code: 404,
        message: format!("没有找到容器'{}'", container_name),
    })
}

/// 删除指定名称的容器
pub async fn remove_container(container_name: &str) -> Result<(), Error> {
    let docker = get_docker().await?;

    docker
        .remove_container(
            container_name,
            Some(RemoveContainerOptions {
                v: true,     // 删除容器时也删除其挂载的卷
                force: true, // 强制删除，即使容器正在运行
                link: false, // 默认不删除链接
            }),
        )
        .await?;

    Ok(())
}

/// 获取指定名称的容器的日志
pub async fn get_container_logs(container_name: &str, emit_name: &str) -> Result<(), Error> {
    let docker = get_docker().await?;

    let logs_options = LogsOptions::<String> {
        stdout: true,
        stderr: true,
        tail: "all".to_string(),
        follow: true,
        ..Default::default()
    };

    let mut log_stream = docker.logs(container_name, Some(logs_options));

    while let Some(log_output) = log_stream.next().await {
        match log_output {
            Ok(log_output) => {
                app_handle()
                    .emit(emit_name, format!("{}", log_output))
                    .unwrap();
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(())
}
