<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Window } from "@tauri-apps/api/window";
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { register } from '@tauri-apps/plugin-global-shortcut';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

const currentWindow = Window.getCurrent();
let inputElement = ref<HTMLInputElement>();
const input = ref("");
async function onEnter() {
    switch (input.value) {
        case '/xs':
            if (!WebviewWindow.getByLabel('novel')) {
                new WebviewWindow('novel', {
                    url: 'src/novel/novel.html'
                })
            } else {
                WebviewWindow.getByLabel('novel')?.unminimize()
                WebviewWindow.getByLabel('novel')?.setFocus()
            }
            break;
        default:
            console.log(await invoke("input_enter", { value: input.value }))
    }
}

onMounted(async () => {
    // 当 input 元素失去焦点时，隐藏窗口
    inputElement.value?.addEventListener("blur", async () => {
        await currentWindow.hide();
    });
    //快捷键控制窗口
    await register('CommandOrControl+Shift+A', async (event) => {
        if (event.state === 'Pressed') {
            if (await currentWindow.isVisible()) {
                await currentWindow.hide();
            } else {
                await currentWindow.show();
                await currentWindow.setFocus();
                inputElement.value?.focus();
            }
        }
    });
    const update = await check();
    console.log(update)
    if (update?.available) {
        console.log(`Update to ${update.version} available! Date: ${update.date}`);
        console.log(`Release notes: ${update.body}`);
        await update.downloadAndInstall();
        await relaunch();
    }
})
</script>

<template>
    <input :style="{ height: '40px', width: '80%', margin: '0 auto', display: 'block' }" @keyup.enter="onEnter"
        v-model="input" ref="inputElement" />
</template>

<style scoped></style>
