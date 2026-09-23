<template>
  <div
      class="flex flex-col h-screen bg-gray-100 dark:bg-gray-900 text-gray-900 dark:text-gray-100 transition-colors duration-200">

    <!-- --- Top Bar: Global Actions --- -->
    <header class="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700 z-10">
      <div class="max-w-7xl mx-auto px-4 py-3">
        <div class="flex justify-between items-center">
          <h1 class="text-xl font-bold flex items-center gap-2 text-orange-600 dark:text-orange-500">
            <HardDrive/>
            {{ t.appTitle[lang] }}
          </h1>
          <div class="flex items-center gap-2">
            <!-- Rescan：按配置页的路径重新扫描，未配置时自动发现 -->
            <button
                @click="rescanAll"
                :disabled="isScanning"
                class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium text-gray-700 dark:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              <RefreshCw :class="{ 'animate-spin': isScanning }" :size="18"/>
              {{ t.rescan[lang] }}
            </button>
            <!-- Language Toggle -->
            <button
                @click="toggleLanguage"
                class="p-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-200"
                :title="lang === 'en' ? t.switchToChinese[lang] : t.switchToEnglish[lang]"
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
                :title="t.toggleTheme[lang]"
            >
              <component :is="settingsStore.isDark ? Moon : Sun" :size="20"/>
            </button>
            <!-- Settings -->
            <button
                @click="openSettings"
                class="p-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-200"
                :title="t.settings[lang]"
            >
              <Settings :size="20"/>
            </button>
          </div>
        </div>
      </div>
    </header>

    <!-- --- Main Content --- -->
    <main class="flex-1 overflow-hidden max-w-7xl w-full mx-auto p-4 gap-4 grid grid-cols-12 min-h-0">

      <!-- Left Col: Local Clusters -->
      <div
          class="col-span-3 flex flex-col gap-4 min-h-0">

        <!-- Convert to Server Action -->
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

        <!-- Clusters List -->
        <div
            class="flex-1 bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 flex flex-col overflow-hidden min-h-0">
          <div class="p-3 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900/50">
            <h3 class="font-semibold text-sm flex items-center gap-2">
              <FileDigit :size="16"/>
              {{ t.localSaves[lang] }}
            </h3>
          </div>
          <div class="flex-1 overflow-y-auto p-2 space-y-1 custom-scrollbar">
          <div v-if="appStore.clusters.length === 0" class="text-center py-10 text-gray-400 text-sm">
            {{ t.noClusters[lang] }}
          </div>
          <div
              v-for="cluster in appStore.clusters"
              :key="cluster.id"
              @click="appStore.changeSelectedCluster(cluster.id)"
                            @contextmenu.prevent="showContextMenu($event, cluster.id, false)"
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
                <div v-if="cluster.latest_phase"
                     class="text-[10px] text-gray-400 mt-0.5 flex items-center gap-2">
                  <span>{{ t.day[lang] }} {{ cluster.latest_phase.cycles }}</span>
                  <span>•</span>
                  <span>{{ getSeasonText(cluster.latest_phase.season) }}</span>
                  <span>•</span>
                  <span>{{ getPhaseText(cluster.latest_phase.now_phase) }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
        </div>
      </div>

      <!-- Middle Col: Server List & Convert to Local -->
      <div class="col-span-3 flex flex-col gap-4 min-h-0">

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
                                @contextmenu.prevent="showContextMenu($event, server.id, true)"
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

        <!-- Convert to Local Action -->
        <div class="flex-none flex justify-center items-center py-2">
          <button
              @click="appStore.convertServerToCluster"
              :disabled="!appStore.selectedServerId || appStore.isConverting"
              class="w-full bg-linear-to-r from-blue-500 to-cyan-600 hover:from-blue-600 hover:to-cyan-700 text-white font-semibold py-3 px-4 rounded-lg shadow-md disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2 transition-all transform active:scale-95"
          >
            <RefreshCw v-if="appStore.isConverting" class="animate-spin" :size="20"/>
            <ArrowLeft v-else :size="20"/>
            {{ t.convertToLocal[lang] }}
          </button>
        </div>
      </div>

      <!-- Right Col: Server Details -->
      <div class="col-span-6 min-h-0">
        <ServerDetails
            v-if="appStore.serverDetail"
            :server="appStore.serverDetail"
            :saves="appStore.serverSaves"
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

    <!-- 右键菜单遮罩（点击任意位置关闭） -->
    <div
        v-if="contextMenu.visible"
        class="fixed inset-0 z-40"
        @click="closeContextMenu"
        @contextmenu.prevent="closeContextMenu"
    ></div>

    <!-- 右键菜单 -->
    <div
        v-if="contextMenu.visible"
        class="fixed z-50 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl py-1 min-w-[180px]"
        :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
    >
      <div
          @click="openInFolder"
          class="flex items-center gap-2 px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer transition-colors"
      >
        <FolderOpen :size="16"/>
        <span>{{ t.openInFolder[lang] }}</span>
      </div>
      <div
          v-if="contextMenu.isServer"
          @click="confirmRepairSaves"
          class="flex items-center gap-2 px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer transition-colors"
      >
        <Wrench :size="16"/>
        <span>{{ t.repairSaves[lang] }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import {computed, onMounted, ref} from 'vue';
import {useRouter} from 'vue-router';
import {
  Settings, RefreshCw, HardDrive, Server, FileDigit,
  ArrowRight, ArrowLeft, Trash2, Languages, Moon, Sun, FolderOpen, Wrench
} from 'lucide-vue-next';
import {TRANSLATIONS} from '@/constants';
import {useSettingsStore} from '@/stores/settingsStore';
import {useAppStore} from '@/stores/appStore';

import ServerDetails from '@/components/ServerDetails.vue';
import {ElMessageBox} from "element-plus";
import {tauriInvokeUtil} from "@/utils/tauriInvokeUtil.js";

const router = useRouter();
const settingsStore = useSettingsStore();
const appStore = useAppStore();

const t = TRANSLATIONS;
// 简单的 lang 快捷访问
const lang = computed(() => settingsStore.language);
const isScanning = ref(false);

// --- 右键菜单 ---
const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  archiveId: null,
  isServer: false
});

const showContextMenu = (event, archiveId, isServer) => {
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    archiveId,
    isServer
  };
};

const closeContextMenu = () => {
  contextMenu.value.visible = false;
};

const openInFolder = async () => {
  const id = contextMenu.value.archiveId;
  if (!id) return;
  closeContextMenu();
  await tauriInvokeUtil('open_archive_in_folder_handler', { id }, { showLoading: false });
};

// 初始化加载：语言与配置在 App.vue 里已读过一次，这里只查数据
onMounted(async () => {
  await rescanAll();
});

// 打开配置页
const openSettings = () => {
  router.push('/settings');
};

// 切换语言：只需保存到后端。界面文案是前端响应式的，后端唯一依赖 locale 的是模组名，
// 而模组名在切到「模组」页时会按当前语言重新解析（缓存按 locale 区分），无需重扫全量数据。
const toggleLanguage = async () => {
  const newLang = settingsStore.language === 'en' ? 'zh' : 'en';
  settingsStore.language = newLang;
  await settingsStore.saveLanguage(newLang);
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


// 按配置页的路径重新扫描
const rescanAll = async () => {
  isScanning.value = true;
  try {
    await appStore.loadMods();
    await appStore.loadClusters();
    await appStore.loadServers();
  } finally {
    isScanning.value = false;
  }
};

// 刷新服务器详情与存档列表（删除存档后重新拉取）
const refreshServerDetails = async () => {
  const id = appStore.selectedServerId;
  if (!id) return;
  await Promise.all([appStore.loadServerDetail(id), appStore.loadServerSaves(id)]);
};

// 处理同步模组后的刷新
const handleSyncMods = async () => {
  await appStore.loadMods();
};

// 清理不一致存档（会删除文件，先二次确认）
const confirmRepairSaves = async () => {
  const id = contextMenu.value.archiveId;
  if (!id) return;
  closeContextMenu();

  try {
    await ElMessageBox.confirm(
        t.repairSavesConfirm[lang.value],
        t.warning[lang.value],
        {
          confirmButtonText: t.confirm[lang.value],
          cancelButtonText: t.cancel[lang.value],
          type: 'warning',
        }
    );
    await appStore.repairSaves(id);
  } catch (error) {
    // 用户取消
  }
};
</script>