<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const input_value = ref("");

async function onEnter() {
  console.log(await invoke("input_enter", { value: input_value.value }))
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
}
</script>

<template>
  <input :style="{ width: '80%', margin: '0 auto', display: 'block' }" @keyup.enter="onEnter" v-model="input_value" />
</template>
