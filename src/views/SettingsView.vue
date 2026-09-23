<template>
  <div
      class="flex flex-col h-screen bg-gray-100 dark:bg-gray-900 text-gray-900 dark:text-gray-100 transition-colors duration-200">

    <!-- --- Top Bar --- -->
    <header class="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700 z-10">
      <div class="max-w-4xl mx-auto px-4 py-3 flex items-center justify-between">
        <h1 class="text-xl font-bold flex items-center gap-2 text-orange-600 dark:text-orange-500">
          <SettingsIcon :size="20"/>
          {{ t.settings[lang] }}
        </h1>
        <button
            @click="goHome"
            class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium text-gray-700 dark:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
        >
          <ArrowLeft :size="18"/>
          {{ t.back[lang] }}
        </button>
      </div>
    </header>

    <!-- --- 配置表单 --- -->
    <main class="flex-1 overflow-y-auto w-full max-w-4xl mx-auto p-4 custom-scrollbar">
      <div
          class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-5 space-y-5">
        <div class="flex items-start justify-between gap-3">
          <p class="text-xs text-gray-500 dark:text-gray-400">{{ t.settingsHint[lang] }}</p>
          <button
              @click="autoScan"
              :disabled="scanning"
              class="shrink-0 flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium text-gray-700 dark:text-gray-200 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-wait transition-colors"
              :title="t.discoverPathsHint[lang]"
          >
            <RefreshCw :class="{ 'animate-spin': scanning }" :size="16"/>
            {{ t.discoverPaths[lang] }}
          </button>
        </div>

        <!-- 目录配置 -->
        <div v-for="field in pathFields" :key="field.key" class="flex flex-col gap-1">
          <label class="text-xs font-semibold text-gray-500 dark:text-gray-400 tracking-wider">
            {{ field.label[lang] }}
          </label>
          <div class="flex gap-2">
            <input
                type="text"
                readonly
                :value="form[field.key]"
                :placeholder="t.file_select_message[lang]"
                :title="form[field.key]"
                class="flex-1 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-md px-3 py-2 text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <button
                @click="browse(field.key)"
                :title="t.browse[lang]"
                class="bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 border border-gray-300 dark:border-gray-600 rounded-md px-3 py-2 text-gray-700 dark:text-gray-200 transition-colors"
            >
              <FolderOpen :size="18"/>
            </button>
            <button
                @click="form[field.key] = ''"
                :title="t.clear[lang]"
                class="bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 border border-gray-300 dark:border-gray-600 rounded-md px-3 py-2 text-gray-700 dark:text-gray-200 transition-colors"
            >
              <X :size="18"/>
            </button>
          </div>
        </div>

        <!-- 全局服务器令牌 -->
        <div class="flex flex-col gap-1">
          <label class="text-xs font-semibold text-gray-500 dark:text-gray-400 tracking-wider">
            {{ t.globalToken[lang] }}
          </label>
          <div
              class="flex items-center bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-md px-3 py-2 focus-within:ring-2 focus-within:ring-blue-500">
            <Key :size="16" class="text-gray-400 mr-2 shrink-0"/>
            <input
                type="password"
                v-model="form.global_cluster_token"
                :placeholder="t.placeholderToken[lang]"
                class="bg-transparent border-none outline-none text-sm w-full text-gray-900 dark:text-gray-100 placeholder-gray-400"
            />
          </div>
          <p class="text-[11px] text-gray-400">{{ t.globalTokenHint[lang] }}</p>
        </div>

        <!-- 保存 -->
        <div class="flex justify-end pt-1">
          <button
              @click="save"
              :disabled="saving"
              class="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-5 rounded-lg shadow-sm disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 transition-colors"
          >
            <RefreshCw v-if="saving" class="animate-spin" :size="16"/>
            {{ t.save[lang] }}
          </button>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup>
import {computed, onMounted, reactive, ref} from 'vue';
import {useRouter} from 'vue-router';
import {ArrowLeft, FolderOpen, Key, RefreshCw, Settings as SettingsIcon, X} from 'lucide-vue-next';
import {open} from '@tauri-apps/plugin-dialog';
import {ElMessage} from 'element-plus';
import {TRANSLATIONS} from '@/constants';
import {tauriInvokeUtil} from '@/utils/tauriInvokeUtil.js';
import {useSettingsStore} from '@/stores/settingsStore';

const router = useRouter();
const settingsStore = useSettingsStore();

const t = TRANSLATIONS;
const lang = computed(() => settingsStore.language);

// 表单先承载一份本地副本，保存时才写回后端
const form = reactive({
  dst_client_path: '',
  dst_server_path: '',
  workshop_path: '',
  archive_root: '',
  global_cluster_token: '',
});

const pathFields = [
  {key: 'dst_client_path', label: t.clientInstallPath},
  {key: 'dst_server_path', label: t.serverInstallPath},
  {key: 'workshop_path', label: t.workshopPath},
  {key: 'archive_root', label: t.archiveRootPath},
];

const saving = ref(false);
const scanning = ref(false);

const fillForm = () => {
  form.dst_client_path = settingsStore.settings.dst_client_path;
  form.dst_server_path = settingsStore.settings.dst_server_path;
  form.workshop_path = settingsStore.settings.workshop_path;
  form.archive_root = settingsStore.settings.archive_root;
  form.global_cluster_token = settingsStore.settings.global_cluster_token;
};

onMounted(async () => {
  await settingsStore.loadSettings();
  fillForm();
});

// 自动扫描：检测本机的 DST 相关目录并写入配置。
// 只覆盖本次扫描到的项，没检测到的保持原值（避免误清空用户手填的路径）。
const autoScan = async () => {
  scanning.value = true;
  try {
    const res = await tauriInvokeUtil('discover_paths_handler', {}, {showLoading: false});
    if (res.code !== 200 || !res.data) {
      ElMessage.error(t.discoverFailed[lang.value]);
      return;
    }

    // 扫描结果的字段名与配置字段一一对应，直接按返回值构造 patch，
    // 不硬编码字段清单（否则后端改字段名时会变成"静默填不上"）
    const patch = {};
    for (const [key, value] of Object.entries(res.data)) {
      if (value) {
        patch[key] = value;
        form[key] = value;
      }
    }

    if (Object.keys(patch).length === 0) {
      ElMessage.warning(t.discoverNothingFound[lang.value]);
      return;
    }

    const saved = await settingsStore.saveSettings(patch);
    if (saved.code !== 200) {
      ElMessage.error(saved.message || t.saveSettingsFailed[lang.value]);
      return;
    }
    fillForm();
    ElMessage.success(t.discoverSuccess[lang.value]);
  } catch (error) {
    console.error('Auto scan failed:', error);
    ElMessage.error(t.discoverFailed[lang.value]);
  } finally {
    scanning.value = false;
  }
};

const browse = async (key) => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: form[key] || undefined,
      title: t.browse[lang.value],
    });
    if (selected && typeof selected === 'string') {
      form[key] = selected;
    }
  } catch (error) {
    console.error('Failed to open folder dialog:', error);
    ElMessage.error(t.browseFailed[lang.value]);
  }
};

const save = async () => {
  saving.value = true;
  try {
    const res = await settingsStore.saveSettings({...form});
    if (res.code === 200) {
      fillForm();
      ElMessage.success(t.saveSettingsSuccess[lang.value]);
    } else {
      ElMessage.error(res.message || t.saveSettingsFailed[lang.value]);
    }
  } catch (error) {
    console.error('Save settings failed:', error);
    ElMessage.error(t.saveSettingsFailed[lang.value]);
  } finally {
    saving.value = false;
  }
};

const goHome = () => {
  router.push('/');
};
</script>
