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
        <el-tab-pane label="Config">Config</el-tab-pane>
        <el-tab-pane label="Role">Role</el-tab-pane>
        <el-tab-pane label="Task">Task</el-tab-pane>
    </el-tabs>
</template>
<script lang="ts" setup>
import { fetch } from '@tauri-apps/plugin-http';
import { computed, ref } from 'vue'
import { stripHtmlTags } from '../utils/defaultUtils'
import { formatNovelText } from '../utils/novelUtils'

const fqNovelIsLoading = ref(false) //番茄小说是否在获取
const fqNovelId = ref('') //番茄小说id
const fqNovelApi = ref('https://fqnovel.pages.dev/content?item_id=') //番茄小说api地址
const fqNovelContent = ref('') //番茄小说内容
const fqNovelCount = computed(() => { return fqNovelContent.value.length }) //番茄小说字数

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
</script>

<style></style>