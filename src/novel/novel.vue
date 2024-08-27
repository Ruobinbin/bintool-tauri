<template>
    <el-tabs tab-position="left">
        <el-tab-pane label="小说">
            <el-input v-model="fqNovelApi" style="width: 300px" placeholder="番茄小说api" />
            <br />
            <el-input v-model="fqNovelId" style="width: 500px" placeholder="输入小说链接">
                <template #prepend>{{ fqNovelApi }}</template>
            </el-input>
            <el-button @click="getFqNovel" :loading="fqNovelIsLoading">获取小说</el-button>
            <br />
            <el-text class="mx-1" type="info">长度:{{ fqNovelCount }}</el-text>
            <br />
            <el-input v-model="fqNovelContent" style="width: 100%" autosize type="textarea" placeholder="小说内容" />
        </el-tab-pane>
        <el-tab-pane label="生成音频">
            <el-button @click="setNovelTable">清空并且设置表格</el-button>
            <el-button @click="openAudioDir">打开音频目录</el-button>
            <el-button @click="insertNovel(-1)">插入</el-button>
            <el-button @click="test">test</el-button>
            <br />
            <el-table :data="novels">
                <el-table-column label="索引" :min-width="20">
                    <template #default="scope">
                        {{ scope.$index }}
                    </template>
                </el-table-column>
                <el-table-column prop="content" label="content">
                    <template #default="scope">
                        <el-input v-model="scope.row.content" placeholder="请输入内容"></el-input>
                    </template>
                </el-table-column>
                <el-table-column prop="audioSrc" label="audioSrc">
                    <template #default="scope">
                        <audio :src="convertFileSrc(scope.row.audioSrc)" controls></audio>
                    </template>
                </el-table-column>
                <el-table-column label="操作">
                    <template #default="scope">
                        <el-button @click="generateAudio(scope.$index)" :loading="scope.row.loading">生成音频</el-button>
                        <el-button @click="delNovel(scope.$index)">删除</el-button>
                        <el-button @click="insertNovel(scope.$index)">插入</el-button>
                    </template>
                </el-table-column>
            </el-table>
        </el-tab-pane>
        <el-tab-pane label="gpt-sovits">
            <gpt_sovits_model />
        </el-tab-pane>
        <el-tab-pane label="视频">
            <audio src=""></audio>
            <el-input v-model="channelUrl" placeholder="博主主页链接"></el-input>
            <el-button @click="get_video_list">获取视频列表</el-button>
            <audio ref="audios" :src="convertFileSrc(OUTPUT_PATH + '\\audio.wav')" controls></audio>
            <div style="margin-bottom: 100px;">
                已选视频总时长{{ selected_video_list_duration }}
                <div style="display: flex;flex-wrap: wrap;gap: 10px;">
                    <div style="background: lightcoral;flex: 0 1 auto;width: 200px;border-radius: 10px;"
                        v-for="video in selected_video_list" :key="video.id">
                        <img v-if="video.thumbnails.length > 0" :src="video.getLargestThumbnailUrl()" :alt="video.url"
                            @click="del_video(video)" style="width: 100%; height: auto; border-radius: 10px;" />
                        <a :href="video.url" @click.prevent="open(video.url)">{{ video.url }}</a>
                        <p>时长: {{ video.duration }} 秒</p>
                        <el-button @click="video.downloadVideo('/workspace/novel_output/')">下载</el-button>
                    </div>
                </div>
            </div>
            <div>
                <div style="display: flex;flex-wrap: wrap;gap: 10px;">
                    <div style="background: lightgrey;flex: 0 1 auto;width: 200px;border-radius: 10px;"
                        v-for="video in video_list" :key="video.id">
                        <img v-if="video.thumbnails.length > 0" :src="video.getLargestThumbnailUrl()" :alt="video.url"
                            @click="sel_video(video)" style="width: 100%; height: auto; border-radius: 10px;" />
                        <a :href="video.url" @click.prevent="open(video.url)">{{ video.url }}</a>
                        <p>时长: {{ video.duration }} 秒</p>
                    </div>
                </div>
            </div>
        </el-tab-pane>
        <el-tab-pane label="最后合成">
            <el-button @click="test">合成视频</el-button>
            <el-select v-model="videoOrientation" placeholder="选择视频方向">
                <el-option label="横屏 (Landscape)" value="landscape"></el-option>
                <el-option label="竖屏 (Portrait)" value="portrait"></el-option>
            </el-select>
            <el-steps :active="currentStep" finish-status="success">
                <el-step title="合成所有音频文件" />
                <el-step title="生成字幕文件" />
                <el-step title="下载视频" />
                <el-step title="统一视频格式" />
                <el-step title="合成最终视频" />
            </el-steps>
        </el-tab-pane>
    </el-tabs>
</template>
<script lang="ts" setup>
import { fetch } from '@tauri-apps/plugin-http';
import { open } from '@tauri-apps/plugin-shell';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import { getCurrent } from "@tauri-apps/api/window";
import Database from '@tauri-apps/plugin-sql';
import { resourceDir } from '@tauri-apps/api/path';

import gpt_sovits_model from '../components/gpt_sovits_model.vue';
import { computed, onMounted, ref } from 'vue'
import { stripHtmlTags, getFileNameFromPath } from '../utils/defaultUtils'
import { formatNovelText, Novel, fetchNovels } from '../utils/novelUtils'
import { IThumbnail, Video } from '../utils/yt_dlp_uitls'
import { ElMessage, ElMessageBox } from 'element-plus'

const OUTPUT_PATH = ref('');
const fqNovelIsLoading = ref(false) //番茄小说是否在获取
const fqNovelId = ref('') //番茄小说id
const fqNovelApi = ref('https://fqnovel.pages.dev/content?item_id=') //番茄小说api地址
const fqNovelContent = ref('') //番茄小说内容
const fqNovelCount = computed(() => { return fqNovelContent.value.length }) //番茄小说字数
const novels = ref<Novel[]>([])//小说
const video_list = ref<Video[]>([]);//视频列表
const selected_video_list = ref<Video[]>([]);//已选视频
const selected_video_list_duration = computed(() => {
    return selected_video_list.value.reduce((sum, video) => sum + video.duration, 0);
});
const isGptSovitsApiRunning = ref(false) //是否开启sovits
const channelUrl = ref('https://www.youtube.com/@hetongxue') //视频主页链接
const audios = ref<HTMLAudioElement>(); // 所有音频
const videoOrientation = ref('portrait'); // 默认竖屏
const currentStep = ref(0);  // 当前步骤的索引



//载入时触发
onMounted(async () => {
    OUTPUT_PATH.value = await resourceDir() + '\\user_files\\novel_output';
    //创建数据库
    const db = await Database.load('sqlite:bintool.db');
    await db.execute(`
      CREATE TABLE IF NOT EXISTS novels (
        id INTEGER PRIMARY KEY,
        audioSrc TEXT NOT NULL,
        content TEXT NOT NULL
      )
    `);
    //初始化novels
    novels.value = await fetchNovels()
    //监听窗口关闭事件
    await getCurrent().onCloseRequested(async () => {
        await db.execute('DELETE FROM novels');
        for (const novel of novels.value) {
            await db.execute('INSERT INTO novels (content , audioSrc) VALUES (?, ?)', [novel.content, novel.audioSrc]);
        }
    });

    listen('gpt_sovits_api_running', (event) => {
        isGptSovitsApiRunning.value = event.payload as boolean;
    });
});

const sel_video = (video: Video) => {
    video_list.value = video_list.value.filter(v => v.id !== video.id);
    selected_video_list.value.push(video);
}

const del_video = (video: Video) => {
    selected_video_list.value = selected_video_list.value.filter(v => v.id !== video.id);
    video_list.value.unshift(video);
}

//获取视频列表
const get_video_list = async () => {
    const cmd = [
        '--flat-playlist',
        '--print-json',
        '--playlist-end', '50',
        channelUrl.value
    ];

    const log = await invoke('run_yt_dlp_cmd', { cmd });
    let logStr = log as string;
    video_list.value = logStr.trim().split('\n').map((videoStr) => {
        let video = JSON.parse(videoStr);
        let thumbnails: IThumbnail[] = video.thumbnails
        return new Video(video.id, video.url, video.duration, thumbnails);
    });
}
const test = async () => {
    try {
        currentStep.value = 0;
        // 生成用于合成全部音频的audios.txt文件
        let filePath = `${OUTPUT_PATH.value}\\audios.txt`;
        let text = novels.value.map(novel => `file /workspace/novel_output/${getFileNameFromPath(novel.audioSrc)}`).join('\n');
        await invoke('write_string_to_file', { text, filePath });
        // 音频合成
        let audioSynthesiscmd: string[] = [
            "-y",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            "/workspace/novel_output/audios.txt",
            "-c:a",
            "pcm_s16le",
            "/workspace/novel_output/audios.wav"
        ];
        await invoke('run_ffmpeg_cmd', { cmd: audioSynthesiscmd });
        audios.value!.src = convertFileSrc(OUTPUT_PATH.value + '\\audios.wav');
        audios.value?.load();
        currentStep.value = 1;

        // 生成字幕所需的txt文件
        let novelsTextFilePath = `${OUTPUT_PATH.value}\\text.txt`;
        let novelsText = novels.value.map(novel => novel.content).join('\n');
        await invoke('write_string_to_file', { text: novelsText, filePath: novelsTextFilePath });
        currentStep.value = 3;

        // 字幕生成
        let audioPath = "novel_output/audios.wav";
        let textPath = "novel_output/text.txt";
        let outputPath = "novel_output/audios.srt";
        await invoke('run_aeneas_cmd', { audioPath, textPath, outputPath });
        currentStep.value = 2;

        // 下载已选视频
        for (const video of selected_video_list.value) {
            const filePath = `${OUTPUT_PATH.value}\\${video.id}.mp4`;
            const fileExists = await invoke<boolean>('check_file_exists', { path: filePath });
            if (fileExists) {
                continue;
            }
            await video.downloadVideo('/workspace/novel_output/');
        }
        currentStep.value = 3;

        //统一视频大小
        const orientation = videoOrientation.value === 'landscape' ? 'landscape' : 'portrait';
        for (const video of selected_video_list.value) {
            const videoPath = `/workspace/novel_output/${video.id}.mp4`;
            const outputPath = `/workspace/novel_output/${video.id}_${orientation}.mp4`;
            const path = `${OUTPUT_PATH.value}\\${video.id}_${orientation}.mp4`
            const fileExists = await invoke<boolean>('check_file_exists', { path: path });
            if (fileExists) {
                continue;
            }
            let cmd = [
                "-y",
                "-i", videoPath,
                '-vf', orientation === 'landscape'
                    ? 'scale=-1:1080,pad=1920:1080:(ow-iw)/2:(oh-ih)/2'
                    : 'scale=1080:-1,pad=1080:1920:(ow-iw)/2:(oh-ih)/2',
                '-preset', 'fast',
                outputPath
            ];
            await invoke('run_ffmpeg_cmd', { cmd });
        }

        currentStep.value = 4;

        // 生成用于合成全部视频的videos.txt文件
        let videoListPath = `${OUTPUT_PATH.value}\\videos.txt`;
        let videoPaths = selected_video_list.value.map(video => `file '/workspace/novel_output/${video.id}_${orientation}.mp4'`).join('\n');
        await invoke('write_string_to_file', { text: videoPaths, filePath: videoListPath });
        let cmd = [
            "-y",
            "-f", "concat",
            "-safe", "0",
            "-i", "/workspace/novel_output/videos.txt",
            "-i", "/workspace/novel_output/audios.wav",
            "-vf", "subtitles=/workspace/novel_output/audios.srt:force_style='FontName=Noto Sans CJK SC,FontSize=25,PrimaryColour=&H00FFFF&,WrapStyle=0'",
            "-c:v", "libx264",
            "-preset", "fast",
            "-c:a", "aac",
            "-shortest",
            "/workspace/novel_output/final_video.mp4"
        ];
        await invoke('run_ffmpeg_cmd', { cmd });
        currentStep.value = 5;

    } catch (error) {
        currentStep.value = 0;
        ElMessage({
            message: `操作失败: ${error as string}`,
            type: 'error',
        });
        return;
    }
};

// 打开音频目录
const openAudioDir = async () => {
    let path = OUTPUT_PATH.value
    await invoke('open_path', { path })
}

// 生成音频
const generateAudio = async (novelIndex: number) => {
    if (!isGptSovitsApiRunning.value) {
        ElMessage({
            message: 'GPT-Sovits未开启',
            type: 'error',
        })
        return
    }
    await novels.value[novelIndex].generateAudio()
        .catch((error) => {
            ElMessage({
                message: error as string,
                type: 'error',
            });
        });
}
// 删除小说
const delNovel = (novelIndex: number) => {
    novels.value.splice(novelIndex, 1)
}
// 插入小说
const insertNovel = (novelIndex: number) => {
    novels.value.splice(novelIndex + 1, 0, new Novel());
}

// 获取番茄小说
const getFqNovel = async () => {
    fqNovelIsLoading.value = true
    const response = await fetch(`${fqNovelApi.value}${fqNovelId.value}`, {
        headers: {
            'Accept-Encoding': 'br' // 请求支持Brotli压缩
        }
    });

    if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
    }

    const text = await response.text();
    fqNovelContent.value = formatNovelText(stripHtmlTags(text))
    fqNovelIsLoading.value = false
}

// 设置表格
const setNovelTable = () => {
    ElMessageBox.confirm(
        '此操作会清空表格, 是否继续?',
        'Warning',
        {
            confirmButtonText: '确定',
            cancelButtonText: '取消',
            type: 'warning',
        }
    )
        .then(() => {
            if (fqNovelContent.value) {
                novels.value = []
                fqNovelContent.value.split('\n').map((content) => {
                    let novel = new Novel(content)
                    novels.value.push(novel)
                })
            }
        })
        .catch(() => {
            ElMessage({
                type: 'info',
                message: '已取消',
            })
        })
}



</script>

<style></style>