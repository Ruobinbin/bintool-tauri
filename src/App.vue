<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Window } from "@tauri-apps/api/window";
import { register } from '@tauri-apps/plugin-global-shortcut';

const input_value = ref("");
async function onEnter() {
    console.log(await invoke("input_enter", { value: input_value.value }))
}

onMounted(async () => {
    //快捷键控制窗口
    const currentWindow = Window.getCurrent();
    await register('CommandOrControl+Shift+A', async (event) => {
        if (event.state === 'Pressed') {
            if (await currentWindow.isVisible()) {
                currentWindow.hide();
            } else {
                currentWindow.show();
            }
        }
    });
})

</script>

<template>
    <input :style="{ width: '80%', margin: '0 auto', display: 'block' }" @keyup.enter="onEnter" v-model="input_value" />
</template>

<style scoped></style>
