use bollard::{
    container::{
        Config, CreateContainerOptions, InspectContainerOptions, ListContainersOptions, LogOutput,
        StartContainerOptions,
    },
    errors::Error,
    exec::{CreateExecOptions, StartExecResults},
    secret::{ContainerStateStatusEnum, ContainerSummary, PortBinding},
    Docker,
};
use futures::{Stream, StreamExt};
use std::{collections::HashMap, pin::Pin};

use crate::GPT_SOVITS_MODEL_DIR;
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

// 开启gpt-sovits
pub async fn start_gpt_sovits() -> Result<String, Error> {
    let docker = get_docker().await?;

    match docker
        .start_container("gpt-sovits", None::<StartContainerOptions<&str>>)
        .await
    {
        Ok(_) => {}
        Err(e) => match e {
            Error::DockerResponseServerError { status_code, .. } => {
                if status_code == 404 {
                    create_gpt_sovits().await?;
                    return Box::pin(start_gpt_sovits()).await;
                } else {
                    return Err(e);
                }
            }
            _ => {
                return Err(e);
            }
        },
    }
    let cmd = vec![
        "python",
        "api.py",
        "-dr",
        "model/syq/我整理完今天的照片就休息了，你也别熬夜打游戏哦。.wav",
        "-dt",
        "我整理完今天的照片就休息了，你也别熬夜打游戏哦。",
        "-dl",
        "zh",
        "-d",
        "cuda",
        "-s",
        "model/syq/sanyueqi_e15_s180.pth",
        "-g",
        "model/syq/sanyueqi-e15.ckpt",
    ];
    let mut stream = run_command_in_container("gpt-sovits", cmd).await?;
    while let Some(result) = stream.next().await {
        match result {
            Ok(output) => {
                println!("输出: {}", output);
                if output.contains("Press CTRL+C to quit") {
                    return Ok("gpt_sovits启动成功".to_string());
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok("gpt_sovits启动后没有输出Press CTRL+C to quit".to_string())
}

// 创建gpt-sovits容器
pub async fn create_gpt_sovits() -> Result<(), Error> {
    let docker = get_docker().await?;

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

    let mount_path = GPT_SOVITS_MODEL_DIR
        .get()
        .unwrap()
        .to_string_lossy()
        .to_string()
        + ":/workspace/model";

    let config = Config::<&str> {
        image: Some("breakstring/gpt-sovits:latest"),
        // cmd: Some(vec![
        //     "python",
        //     "api.py",
        //     "-dr",
        //     "model/syq/我整理完今天的照片就休息了，你也别熬夜打游戏哦。.wav",
        //     "-dt",
        //     "我整理完今天的照片就休息了，你也别熬夜打游戏哦。",
        //     "-dl",
        //     "zh",
        //     "-d",
        //     "cuda",
        //     "-s",
        //     "model/syq/sanyueqi_e15_s180.pth",
        //     "-g",
        //     "model/syq/sanyueqi-e15.ckpt",
        // ]),
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
            let stream = output.map(|chunk| match chunk {
                Ok(LogOutput::StdErr { message }) => Ok(format!(
                    "标准错误输出: {}",
                    String::from_utf8_lossy(&message)
                )),
                Ok(LogOutput::StdOut { message }) => {
                    Ok(format!("标准输出: {}", String::from_utf8_lossy(&message)))
                }
                Ok(LogOutput::StdIn { message }) => {
                    Ok(format!("标准输入: {}", String::from_utf8_lossy(&message)))
                }
                Ok(LogOutput::Console { message }) => {
                    Ok(format!("控制台输出: {}", String::from_utf8_lossy(&message)))
                }
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
