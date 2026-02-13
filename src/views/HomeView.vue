<template>
  <div
      class="flex flex-col h-screen bg-gray-100 dark:bg-gray-900 text-gray-900 dark:text-gray-100 transition-colors duration-200">

    <!-- --- Top Bar: Global Settings & Paths --- -->
    <header class="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700 z-10">
      <div class="max-w-7xl mx-auto px-4 py-3">
        <div class="flex justify-between items-center mb-4">
          <h1 class="text-xl font-bold flex items-center gap-2 text-orange-600 dark:text-orange-500">
            <HardDrive/>
            {{ t.appTitle[lang] }}
          </h1>
          <div class="flex items-center gap-2">
            <!-- Language Toggle -->
            <button
                @click="toggleLanguage"
                class="p-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-200"
                :title="lang === 'en' ? 'Switch to Chinese' : 'Switch to English'"
            >
              <div class="flex items-center gap-2">
                <Languages :size="20"/>
                <span class="text-sm font-medium">{{ lang === 'en' ? 'EN' : '中文' }}</span>
              </div>
            </button>
            <!-- Theme Toggle -->
            <button
                @click="settingsStore.toggleTheme()"
                class="p-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-200"
                title="Toggle Theme"
            >
              <component :is="settingsStore.isDark ? Moon : Sun" :size="20"/>
            </button>
          </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-12 gap-4 items-end">
          <div class="md:col-span-3">
            <label class="text-xs font-semibold text-gray-500 dark:text-gray-400 tracking-wider mb-1 block">
              {{ t.serverToken[lang] }}
            </label>
            <div
                class="flex items-center bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-md px-3 py-2 focus-within:ring-2 focus-within:ring-blue-500">
              <Key :size="16" class="text-gray-400 mr-2"/>
              <input
                  type="password"
                  v-model="appStore.appInfo.serverToken"
                  :placeholder="t.placeholderToken[lang]"
                  class="bg-transparent border-none outline-none text-sm w-full text-gray-900 dark:text-gray-100 placeholder-gray-400"
              />
            </div>
          </div>

          <div class="md:col-span-4">
            <PathSelector
                :label="t.localSavePath[lang]"
                v-model="appStore.appInfo.localSavePath"
                :placeholder="t.file_select_message[lang]"
                @search="scanArchives"
                @reset="defaultScanArchives"
                :maxLength="6"
            />
          </div>

          <div class="md:col-span-4">
            <PathSelector
                :label="t.serverPath[lang]"
                v-model="appStore.appInfo.serverInstallPath"
                :placeholder="t.file_select_message[lang]"
                @search="scanMods"
                @reset="defaultScanMods"
                :maxLength="7"
            />
          </div>

          <div class="md:col-span-1 flex justify-center pb-0.5">
            <div class="p-2 rounded-full bg-gray-100 dark:bg-gray-700 text-gray-400">
              <Settings :size="20"/>
            </div>
          </div>
        </div>
      </div>
    </header>

    <!-- --- Main Content --- -->
    <main class="flex-1 overflow-hidden max-w-7xl w-full mx-auto p-4 gap-4 grid grid-cols-12 min-h-0">

      <!-- Left Col: Local Clusters -->
      <div
          class="col-span-3 bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 flex flex-col overflow-hidden min-h-0">
        <div class="p-3 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900/50">
          <h3 class="font-semibold text-sm flex items-center gap-2">
            <FileDigit :size="16"/>
            {{ t.localSaves[lang] }}
          </h3>
        </div>
        <div class="flex-1 overflow-y-auto p-2 space-y-1 custom-scrollbar">
          <div v-if="appStore.clusters.length === 0" class="text-center py-10 text-gray-400 text-sm">
            No clusters found.<br/>Check path settings.
          </div>
          <div
              v-for="cluster in appStore.clusters"
              :key="cluster.id"
              @click="appStore.changeSelectedCluster(cluster.id)"
              class="p-3 rounded-md cursor-pointer transition-all border"
              :class="appStore.selectedClusterId === cluster.id
              ? 'bg-blue-50 dark:bg-blue-900/30 border-blue-200 dark:border-blue-700 ring-1 ring-blue-400'
              : 'bg-transparent border-transparent hover:bg-gray-50 dark:hover:bg-gray-700/50'"
          >
            <div class="flex items-center gap-3">
              <div
                  class="p-2 rounded-full transition-colors"
                  :class="appStore.selectedClusterId === cluster.id ? 'bg-blue-100 text-blue-600 dark:bg-blue-800 dark:text-blue-200' : 'bg-gray-100 dark:bg-gray-700 text-gray-500'"
              >
                <FileDigit :size="20"/>
              </div>
              <div class="overflow-hidden flex-1">
                <p class="font-medium text-sm truncate text-gray-800 dark:text-gray-200">
                  {{ cluster.cluster_name || cluster.id }}
                </p>
                <p class="text-xs text-gray-500 truncate font-mono">{{ cluster.id }}</p>
                <div v-if="cluster.archive_phase_vo && cluster.archive_phase_vo.length > 0"
                     class="text-[10px] text-gray-400 mt-0.5 flex items-center gap-2">
                  <span>{{ t.day[lang] }} {{ cluster.archive_phase_vo[0].cycles }}</span>
                  <span>•</span>
                  <span>{{ getSeasonText(cluster.archive_phase_vo[0].season) }}</span>
                  <span>•</span>
                  <span>{{ getPhaseText(cluster.archive_phase_vo[0].now_phase) }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Middle Col: Conversion Action & Server List -->
      <div class="col-span-3 flex flex-col gap-4 min-h-0">

        <!-- Action Area -->
        <div class="flex-none flex justify-center items-center py-2">
          <button
              @click="appStore.convertCluster"
              :disabled="!appStore.selectedClusterId || appStore.isConverting"
              class="w-full bg-linear-to-r from-orange-500 to-red-600 hover:from-orange-600 hover:to-red-700 text-white font-semibold py-3 px-4 rounded-lg shadow-md disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2 transition-all transform active:scale-95"
          >
            <RefreshCw v-if="appStore.isConverting" class="animate-spin" :size="20"/>
            <ArrowRight v-else :size="20"/>
            {{ t.convert[lang] }}
          </button>
        </div>

        <!-- Server List -->
        <div
            class="flex-1 bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 flex flex-col overflow-hidden min-h-0">
          <div class="p-3 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900/50">
            <h3 class="font-semibold text-sm flex items-center gap-2">
              <Server :size="16"/>
              {{ t.serverList[lang] }}
            </h3>
          </div>
          <div class="flex-1 overflow-y-auto p-2 space-y-1 custom-scrollbar">
            <div
                v-for="server in appStore.servers"
                :key="server.id"
                @click="appStore.changeSelectedServer(server.id)"
                class="p-3 rounded-md cursor-pointer transition-all border group"
                :class="appStore.selectedServerId === server.id
                ? 'bg-blue-50 dark:bg-blue-900/30 border-blue-200 dark:border-blue-700 ring-1 ring-blue-400'
                : 'bg-transparent border-transparent hover:bg-gray-50 dark:hover:bg-gray-700/50'"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-3 overflow-hidden">
                  <div
                      class="w-2 h-2 shrink-0 rounded-full"
                      :class="server.status === 'running'
                      ? 'bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)]'
                      : server.status === 'starting' ? 'bg-yellow-500 animate-pulse' : 'bg-gray-400'"
                  ></div>
                  <div class="min-w-0">
                    <p class="font-medium text-sm text-gray-800 dark:text-gray-200 truncate">
                      {{ server.cluster_name || server.id }}
                    </p>
                    <p class="text-xs text-gray-500 font-mono truncate">{{ server.id }}</p>
                  </div>
                </div>

                <button
                    @click.stop="confirmDelete(server)"
                    class="p-2 opacity-0 group-hover:opacity-100 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-full transition-all"
                    :title="t.delete[lang]"
                >
                  <Trash2 :size="16"/>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Right Col: Server Details -->
      <div class="col-span-6 min-h-0">
        <ServerDetails
            v-if="selectedServer"
            :server="selectedServer"
            :lang="lang"
            @update="appStore.updateServer"
            @stop="appStore.stopServer"
            @start="appStore.startServer"
            @refresh="refreshServerDetails"
            @syncMods="handleSyncMods"
        />
        <div v-else
             class="h-full flex flex-col items-center justify-center bg-gray-50 dark:bg-gray-800/50 rounded-lg border-2 border-dashed border-gray-300 dark:border-gray-700 text-gray-400">
          <Server :size="48" class="mb-4 opacity-50"/>
          <p>{{ t.noSelection[lang] }}</p>
        </div>
      </div>

    </main>
  </div>
</template>

<script setup>
import {computed, onMounted} from 'vue';
import {
  Settings, RefreshCw, Key, HardDrive, Server, FileDigit,
  ArrowRight, Trash2, Languages, Moon, Sun
} from 'lucide-vue-next';
import {TRANSLATIONS} from '@/constants';
import {useSettingsStore} from '@/stores/settingsStore';
import {useAppStore} from '@/stores/appStore';

import PathSelector from '@/components/PathSelector.vue';
import ServerDetails from '@/components/ServerDetails.vue';
import {ElMessage, ElMessageBox} from "element-plus";
import {tauriInvokeUtil} from "@/utils/tauriInvokeUtil.js";
import {delay} from "@/utils/timeUtil.js";

const settingsStore = useSettingsStore();
const appStore = useAppStore();

const t = TRANSLATIONS;
// 简单的 lang 快捷访问
const lang = computed(() => settingsStore.language);

// 查找当前选中的 Server 对象
const selectedServer = computed(() =>
    appStore.servers.find(s => s.id === appStore.selectedServerId)
);

// 初始化加载
onMounted(() => {
  scanMods();
  scanArchives();
});

// 切换语言
const toggleLanguage = () => {
  settingsStore.language = settingsStore.language === 'en' ? 'zh' : 'en';
};

// 删除确认
const confirmDelete = async (server) => {
  ElMessageBox.confirm(
      `${t.confirmDelete[lang.value]} "${server.cluster_name || server.id}"?`,
      t.warning[lang.value],
      {
        type: 'warning',
      }
  ).then(() => {
    // 用户确认删除
    appStore.deleteServer(server.id);
  });
};

// 翻译季节
const getSeasonText = (season) => {
  const seasonMap = {
    spring: t.spring[lang.value],
    summer: t.summer[lang.value],
    autumn: t.autumn[lang.value],
    winter: t.winter[lang.value]
  };
  return seasonMap[season] || season;
};

// 翻译阶段
const getPhaseText = (phase) => {
  const phaseMap = {
    day: t.dayPhase[lang.value],
    dusk: t.dusk[lang.value],
    night: t.night[lang.value]
  };
  return phaseMap[phase] || t.unknown[lang.value];
};


// 调用模组扫描（可以传入自定义路径，或者传 null）
const invokeScanMods = async (customPath) => {
  const res = await tauriInvokeUtil('scan_dst_mods_handler', {customPath: customPath}, {showLoading: false});
  if (res.code === 200) {
    if (res.data.found) {
      // 新的数据结构: mods 是 [{ folder_name, mod_name }] 数组
      appStore.mods = res.data.mods || [];
      appStore.appInfo.serverInstallPath = res.data.server_path;
    } else {
      ElMessage.warning(t.dst_mod_scan_warn_message[settingsStore.language]);
      appStore.appInfo.serverInstallPath = '';
      appStore.mods = [];
    }
  }
};

// 调用存档扫描（可以传入自定义路径，或者传 null）
const invokeScanArchives = async (customPath) => {
  const res = await tauriInvokeUtil('scan_dst_archives_handler', {customPath: customPath}, {showLoading: false});
  if (res.code === 200) {
    if (res.data.found) {
      appStore.clusters = res.data.clusters;
      appStore.servers = res.data.servers;
      appStore.appInfo.localSavePath = res.data.path;
    } else {
      ElMessage.warning(t.dst_archive_scan_warn_message[settingsStore.language]);
      appStore.appInfo.localSavePath = '';
      appStore.clusters = res.data.clusters;
      appStore.servers = res.data.servers;
    }
  }
}


const scanArchives = async () => {
  if (appStore.appInfo.localSavePath === '') {
    ElMessage.warning(t.file_select_message[settingsStore.language]);
    return;
  }
  await invokeScanArchives(appStore.appInfo.localSavePath);
};

const defaultScanArchives = async () => {
  appStore.appInfo.localSavePath = '';

  await delay(500);
  appStore.appInfo.localSavePath = t.dst_scan_prompt_message[settingsStore.language];

  await delay(500)
  await invokeScanArchives(null)
}

const scanMods = async () => {
  if (appStore.appInfo.serverInstallPath === '') {
    ElMessage.warning(t.file_select_message[settingsStore.language]);
    return;
  }
  await invokeScanMods(appStore.appInfo.serverInstallPath);
};

const defaultScanMods = async () => {
  appStore.appInfo.serverInstallPath = '';

  await delay(500);
  appStore.appInfo.serverInstallPath = t.dst_scan_prompt_message[settingsStore.language];

  await delay(500)
  await invokeScanMods(null)
};

// 刷新服务器详情（删除存档后重新扫描）
const refreshServerDetails = async () => {
  await invokeScanArchives(appStore.appInfo.localSavePath || null);
};

// 处理同步模组后的刷新
const handleSyncMods = async () => {
  // 重新扫描模组以获取最新状态
  await invokeScanMods(appStore.appInfo.serverInstallPath || null);
};
</script>