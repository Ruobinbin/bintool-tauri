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
        <el-tab-pane label="Task">
        </el-tab-pane>
    </el-tabs>
</template>
<script lang="ts" setup>
import { fetch } from '@tauri-apps/plugin-http';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import { getCurrent } from "@tauri-apps/api/window";
import Database from '@tauri-apps/plugin-sql';
import { resourceDir } from '@tauri-apps/api/path';


import gpt_sovits_model from '../components/gpt_sovits_model.vue';
import { computed, onMounted, ref } from 'vue'
import { stripHtmlTags } from '../utils/defaultUtils'
import { formatNovelText, Novel, fetchNovels } from '../utils/novelUtils'
import { ElMessage, ElMessageBox } from 'element-plus'

const fqNovelIsLoading = ref(false) //番茄小说是否在获取
const fqNovelId = ref('') //番茄小说id
const fqNovelApi = ref('https://fqnovel.pages.dev/content?item_id=') //番茄小说api地址
const fqNovelContent = ref('') //番茄小说内容
const fqNovelCount = computed(() => { return fqNovelContent.value.length }) //番茄小说字数
const novels = ref<Novel[]>([])
const isGptSovitsApiRunning = ref(false) //是否开启sovits

//载入时触发
onMounted(async () => {
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

const openAudioDir = async () => {
    let appdir = await resourceDir()
    let path = `${appdir}\\user_files\\novel_output`
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
    novels.value.splice(novelIndex + 1, 0, new Novel(''));
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