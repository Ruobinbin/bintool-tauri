<template>
  <el-button :loading="IsGptSovitsApiLoading" @click="startGptSovitsApi">
    {{ IsGptSovitsApiLoading ? "GPT-Sovits API 正在运行" : "启动 GPT-Sovits API" }}
  </el-button>

  <el-scrollbar style="max-height: 300px; overflow-y: auto;">
    <p v-for="log in logs" :key="log">{{ log }}</p>
  </el-scrollbar>

  <div class="container">
    <div v-for="model in models" :key="model.model_name" class="item"
      :class="{ selected: model.model_name === currentModel?.model_name }" @click="selectModel(model)">
      <h3>{{ model.model_name }}</h3>
      <ul>
        <li v-for="wav in model.refer_wavs" :key="wav.refer_wav_path"
          :class="{ 'selected': wav.refer_wav_path === currentReferAudio?.refer_wav_path }"
          @click.stop="selectReferWav(wav)">
          <div>
            <span class="prompt-text">{{ wav.prompt_text }}</span>
            <audio :src="convertFileSrc(wav.refer_wav_path)" controls></audio>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, onMounted } from 'vue';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { listen, emit } from '@tauri-apps/api/event';
import { ElMessage } from 'element-plus';
import { fetch } from '@tauri-apps/plugin-http';

interface ReferWav {
  refer_wav_path: string;
  prompt_text: string;
  prompt_language: string;
}

interface GptSovitsModel {
  model_name: string;
  gpt_model_path: string;
  sovits_model_path: string;
  refer_wavs: ReferWav[];
}

const models = ref<GptSovitsModel[]>([]); // 模型列表
const currentModel = ref<GptSovitsModel | null>(null); // 当前选择的模型
const currentReferAudio = ref<ReferWav | null>(null); // 当前选择的参考音频
const IsGptSovitsApiLoading = ref(false); // 是否正在加载GPT-Sovits API
const isGptSovitsApiRunning = ref(false); // 是否正在运行GPT-Sovits API
const logs = ref<string[]>([]); // 日志输出

onMounted(async () => {
  // 监听GPT-Sovitslog 
  listen('gpt_sovits_api_running', (event) => {
    isGptSovitsApiRunning.value = event.payload as boolean;
    if (isGptSovitsApiRunning.value) {
      ElMessage({
        message: 'GPT-Sovits API 启动成功',
        type: 'success',
      });
    } else {
      ElMessage({
        message: 'GPT-Sovits API 未开启',
        type: 'warning',
      });
    }
  });
  // 监听日志输出
  listen('gpt_sovits_api_log', (event) => {
    logs.value.push(event.payload as string);
  });
  // 获取模型列表
  models.value = await invoke<GptSovitsModel[]>('get_gpt_sovits_models');
});

const localPathToContainerPath = (localPath: string) => {
  let linuxPath = localPath.replace(/\\/g, '/');
  const marker = 'gpt_sovits_model';
  const markerIndex = linuxPath.indexOf(marker);
  return linuxPath.substring(markerIndex);
};

const selectModel = async (model: GptSovitsModel) => {
  if (!isGptSovitsApiRunning.value) {
    ElMessage({
      message: 'GPT-Sovits 未运行，请先启动容器。',
      type: 'error',
    });
    return;
  }

  fetch('http://127.0.0.1:9880/set_model', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      gpt_model_path: localPathToContainerPath(model.sovits_model_path),
      sovits_model_path: localPathToContainerPath(model.gpt_model_path),
    }),
  })
    .then(response => {
      if (response.ok) {
        currentModel.value = model;
        ElMessage({
          message: '模型切换成功。',
          type: 'success',
        });
      } else {
        ElMessage({
          message: `模型切换失败，状态码：${response}`,
          type: 'error',
        });
      }
    })
    .catch(error => {
      ElMessage({
        message: `模型切换失败，错误信息：${error.message}`,
        type: 'error',
      });
    });
};

const startGptSovitsApi = async () => {
  IsGptSovitsApiLoading.value = true;
  await invoke("start_gpt_sovits_api").catch((error) => {
    ElMessage({
      message: error as string,
      type: 'error',
    })
  })
  IsGptSovitsApiLoading.value = false;
}

const selectReferWav = (referWav: ReferWav) => {
  if (!isGptSovitsApiRunning.value) {
    ElMessage({
      message: 'GPT-Sovits 未运行，请先启动容器。',
      type: 'error',
    });
    return;
  }

  fetch('http://127.0.0.1:9880/change_refer', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      refer_wav_path: localPathToContainerPath(referWav.refer_wav_path),
      prompt_text: referWav.prompt_text,
      prompt_language: referWav.prompt_language,
    }),
  }).then(response => {
    if (response.ok) {
      currentReferAudio.value = referWav;
      ElMessage({
        message: '参考音频切换成功。',
        type: 'success',
      });
    } else {
      ElMessage({
        message: `参考音频切换失败，状态码：${response}`,
        type: 'error',
      });
    }
  })
    .catch(error => {
      ElMessage({
        message: `参考音频切换失败，错误信息：${error.message}`,
        type: 'error',
      });
    });
};



</script>

<style>
.container {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  padding: 20px;
}

.item {
  flex: 1 1 auto;
  border-radius: 4px;
  min-height: 36px;
  padding: 10px;
  text-align: center;
  background: lightgray;
  cursor: pointer;
}

.selected {
  border: 2px solid #000;
}

li {
  list-style-type: none;
}
</style>
