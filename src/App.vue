<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Window } from "@tauri-apps/api/window";
import { register } from '@tauri-apps/plugin-global-shortcut';

const currentWindow = Window.getCurrent();
let inputElement = ref<HTMLInputElement>();
const input = ref("");
async function onEnter() {
    console.log(await invoke("input_enter", { value: input.value }))
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
})
</script>

<template>
    <input :style="{ height: '40px', width: '80%', margin: '0 auto', display: 'block' }" @keyup.enter="onEnter"
        v-model="input" ref="inputElement" />
</template>

<style scoped></style>
