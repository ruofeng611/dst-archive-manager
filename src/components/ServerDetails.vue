<template>
  <div
      class="h-full flex flex-col bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden">
    <!-- Header -->
    <div
        class="p-4 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center bg-gray-50 dark:bg-gray-900/50">
      <div>
        <h2 class="text-lg font-bold text-gray-800 dark:text-white flex items-center gap-2">
          {{ server.id }}
          <span
              class="text-xs uppercase font-mono px-2 py-0.5 rounded border border-current opacity-80"
              :class="statusColor"
          >
            {{ t[server.status][lang] }}
          </span>
        </h2>
      </div>
      <div class="flex items-center gap-2">
        <button
            @click="refreshServer"
            :disabled="isRefreshing"
            :title="t.refresh[lang]"
            class="p-2 rounded-md text-gray-500 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-wait transition-colors"
        >
          <RefreshCw :size="18" :class="{ 'animate-spin': isRefreshing }"/>
        </button>
        <button
            @click="toggleStatus"
            :disabled="server.status === 'starting'"
            class="flex items-center gap-2 px-4 py-2 rounded-md font-medium text-white transition-all shadow-sm disabled:opacity-50 disabled:cursor-wait"
            :class="(server.status === 'running' || server.status === 'stopping') ? 'bg-red-500 hover:bg-red-600' : 'bg-green-600 hover:bg-green-700'"
        >
          <component :is="(server.status === 'running' || server.status === 'stopping') ? Square : Play" :size="18" fill="currentColor"/>
          {{ (server.status === 'running' || server.status === 'stopping') ? t.stop[lang] : t.start[lang] }}
        </button>
      </div>
    </div>

    <!-- Tabs -->
    <div class="flex border-b border-gray-200 dark:border-gray-700">
      <button
          v-for="tab in ['config', 'mods', 'saves']"
          :key="tab"
          @click="activeTab = tab"
          class="flex-1 py-3 text-sm font-medium flex justify-center items-center gap-2 transition-colors"
          :class="activeTab === tab
          ? 'text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400 bg-blue-50/50 dark:bg-blue-900/20'
          : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-800'"
      >
        <component :is="tab === 'config' ? Settings : tab === 'mods' ? Layers : FileDigit" :size="16"/>
        {{ t[tab][lang] }}
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-4 custom-scrollbar min-h-0">
      <!-- Configuration Tab -->
      <div v-if="activeTab === 'config'" class="space-y-4 max-w-lg">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{{
              t.serverName[lang]
            }}</label>
          <input
              type="text"
              v-model="localConfig.cluster_name"
              class="w-full bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:text-white"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{{
              t.clusterDescription[lang]
            }}</label>
          <textarea
              v-model="localConfig.cluster_description"
              rows="3"
              class="w-full bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:text-white resize-none"
          />
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{{
                t.maxPlayers[lang]
              }}</label>
            <input
                type="number"
                min="1"
                max="64"
                v-model.number="localConfig.max_players"
                class="w-full bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 dark:text-white"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{{
                t.gameMode[lang]
              }}</label>
            <input
                type="text"
                v-model="localConfig.game_mode"
                disabled
                class="w-full bg-gray-100 dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded px-3 py-2 text-sm text-gray-500 dark:text-gray-400 cursor-not-allowed"
            />
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{{ t.password[lang] }}</label>
          <div class="relative">
            <input
                :type="showPassword ? 'text' : 'password'"
                v-model="localConfig.cluster_password"
                :placeholder="t.noPasswordSet[lang]"
                class="w-full bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded px-3 py-2 pr-10 text-sm focus:ring-2 focus:ring-blue-500 dark:text-white"
            />
            <button
                type="button"
                @click="showPassword = !showPassword"
                class="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
            >
              <component :is="showPassword ? EyeOff : Eye" :size="18"/>
            </button>
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{{
              t.serverToken[lang]
            }}</label>
          <div class="relative">
            <input
                :type="showToken ? 'text' : 'password'"
                v-model="localConfig.cluster_token"
                :placeholder="t.placeholderToken[lang]"
                class="w-full bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded px-3 py-2 pr-10 text-sm focus:ring-2 focus:ring-blue-500 dark:text-white"
            />
            <button
                type="button"
                @click="showToken = !showToken"
                class="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
            >
              <component :is="showToken ? EyeOff : Eye" :size="18"/>
            </button>
          </div>
          <div class="mt-1 flex items-start justify-between gap-2">
            <p class="text-xs text-gray-400">{{ t.serverTokenHint[lang] }}</p>
            <button
                type="button"
                @click="applyGlobalToken"
                :disabled="!globalToken || applyingToken"
                :title="globalToken ? t.useGlobalTokenHint[lang] : t.noGlobalToken[lang]"
                class="shrink-0 flex items-center gap-1 px-2 py-1 text-xs font-medium text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <ArrowDownToLine :size="14"/>
              {{ t.useGlobalToken[lang] }}
            </button>
          </div>
        </div>

        <div class="space-y-3">
          <!-- w-fit：点击范围只覆盖复选框与文字，不撑满整行，避免误触 -->
          <label class="flex w-fit items-center cursor-pointer">
            <input
                type="checkbox"
                v-model="localConfig.pvp"
                class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
            />
            <span class="ml-2 text-sm font-medium text-gray-700 dark:text-gray-300">{{ t.pvpMode[lang] }}</span>
          </label>

          <label class="flex w-fit items-center cursor-pointer">
            <input
                type="checkbox"
                v-model="localConfig.pause_when_empty"
                class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
            />
            <span class="ml-2 text-sm font-medium text-gray-700 dark:text-gray-300">{{ t.pauseWhenEmpty[lang] }}</span>
          </label>
        </div>

        <div class="pt-4">
          <button
              @click="saveConfig"
              :disabled="isSaving"
              class="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-wait"
          >
            <Save :size="16"/>
            {{ isSaving ? t.save[lang] + '...' : t.save[lang] }}
          </button>
        </div>
      </div>

      <!-- Mods Tab -->
      <div v-else-if="activeTab === 'mods'" class="space-y-4">
        <!-- 存档使用的模组 -->
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300 flex items-center gap-2">
              <Package :size="16"/>
              {{ t.archiveMods[lang] }}
            </h3>
            <button
                @click="syncMods"
                :disabled="isSyncingMods"
                class="flex items-center gap-2 px-2 py-1 text-xs text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded transition-colors disabled:opacity-50 disabled:cursor-wait"
                :title="t.syncMods[lang]"
            >
              <RotateCcw :size="14" :class="{ 'animate-spin': isSyncingMods }"/>
              {{ isSyncingMods ? t.syncing[lang] + '...' : t.syncMods[lang] }}
            </button>
          </div>
          <div v-if="archiveMods.length === 0"
               class="text-center py-4 text-gray-500 dark:text-gray-400 italic text-sm bg-gray-50 dark:bg-gray-900/30 rounded border border-gray-200 dark:border-gray-700">
            {{ t.noArchiveMods[lang] }}
          </div>
          <div v-else class="space-y-2">
            <div v-for="mod in archiveMods" :key="mod.id"
                 class="flex items-center justify-between p-3 rounded border"
                 :class="mod.exists 
                   ? 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800' 
                   : 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'">
              <div>
                <h4 class="font-medium text-sm text-gray-800 dark:text-gray-200">{{ mod.name }}</h4>
                <p class="text-xs text-gray-500 font-mono">{{ mod.id }}</p>
              </div>
              <span v-if="!mod.exists"
                    class="text-xs px-2 py-1 rounded bg-red-100 dark:bg-red-900/50 text-red-600 dark:text-red-400 font-medium">
                {{ t.modMissing[lang] }}
              </span>
              <span v-else
                    class="text-xs px-2 py-1 rounded bg-green-100 dark:bg-green-900/50 text-green-600 dark:text-green-400 font-medium">
                ✓
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Saves Tab -->
      <div v-else-if="activeTab === 'saves'" class="space-y-2">
        <div v-if="saves.length === 0"
             class="text-center py-8 text-gray-500 dark:text-gray-400 italic">
          {{ t.noSaves[lang] }}
        </div>

        <!-- 不一致存档提示 + 就地清理 -->
        <div v-if="hasMismatchedSaves"
             class="flex items-center justify-between gap-2 p-3 rounded border border-amber-300 dark:border-amber-800 bg-amber-50 dark:bg-amber-900/20">
          <p class="text-xs text-amber-700 dark:text-amber-400">{{ t.mismatchedSavesHint[lang] }}</p>
          <button
              @click="confirmRepairSaves"
              class="shrink-0 flex items-center gap-1 px-2 py-1 text-xs font-medium text-amber-700 dark:text-amber-300 hover:bg-amber-100 dark:hover:bg-amber-900/40 rounded transition-colors"
              :title="t.repairSaves[lang]"
          >
            <Wrench :size="14"/>
            {{ t.repairSaves[lang] }}
          </button>
        </div>

        <div v-for="(save, index) in saves" :key="save.phase_file_name"
             class="p-3 rounded border"
             :class="save.mismatched
               ? 'bg-amber-50/60 dark:bg-amber-900/10 border-amber-300 dark:border-amber-800 opacity-60'
               : 'bg-gray-50 dark:bg-gray-900/50 border-gray-200 dark:border-gray-700'"
             :title="save.mismatched ? t.mismatchedSaveTip[lang] : ''">
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <FileDigit :size="16" class="text-blue-600 dark:text-blue-400"/>
              <h4 class="font-medium text-sm text-gray-800 dark:text-gray-200">
                {{ save.phase_file_name }}
                <span v-if="index === 0" class="ml-2 text-xs text-green-600 dark:text-green-400 font-semibold">({{
                    t.latest[lang]
                  }})</span>
                <span v-if="save.mismatched"
                      class="ml-2 text-xs px-2 py-0.5 rounded bg-amber-100 dark:bg-amber-900/50 text-amber-700 dark:text-amber-400 font-semibold">
                  {{ t.mismatched[lang] }}
                </span>
              </h4>
            </div>
            <button
                @click="confirmDeleteSave(save.phase_file_name)"
                class="flex items-center gap-1 px-2 py-1 text-xs text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors"
                :title="t.delete[lang]"
            >
              <Trash2 :size="14"/>
              {{ t.delete[lang] }}
            </button>
          </div>
          <div class="grid grid-cols-3 gap-3 text-xs">
            <div class="bg-white dark:bg-gray-800 rounded px-2 py-1.5 border border-gray-200 dark:border-gray-700">
              <span class="text-gray-500 dark:text-gray-400">{{ t.day[lang] }}</span>
              <span class="ml-1 font-semibold text-gray-900 dark:text-white">{{ save.cycles }}</span>
            </div>
            <div class="bg-white dark:bg-gray-800 rounded px-2 py-1.5 border border-gray-200 dark:border-gray-700">
              <span class="text-gray-500 dark:text-gray-400">{{ t.season[lang] }}</span>
              <span class="ml-1 font-semibold text-gray-900 dark:text-white">{{ getSeasonText(save.season) }}</span>
            </div>
            <div class="bg-white dark:bg-gray-800 rounded px-2 py-1.5 border border-gray-200 dark:border-gray-700">
              <span class="text-gray-500 dark:text-gray-400">{{ t.phase[lang] }}</span>
              <span class="ml-1 font-semibold text-gray-900 dark:text-white">{{ getPhaseText(save.now_phase) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import {ref, computed, watch, reactive} from 'vue';
import {
  Play,
  Square,
  Save,
  Settings,
  Layers,
  FileDigit,
  Eye,
  EyeOff,
  Trash2,
  Package,
  RotateCcw,
  Wrench,
  ArrowDownToLine,
  RefreshCw
} from 'lucide-vue-next';
import {TRANSLATIONS} from '@/constants';
import {tauriInvokeUtil} from '@/utils/tauriInvokeUtil.js';
import {ElMessage, ElMessageBox} from 'element-plus';
import {useAppStore} from '@/stores/appStore.js';
import {useSettingsStore} from '@/stores/settingsStore.js';

const props = defineProps({
  server: Object,
  saves: {
    type: Array,
    default: () => []
  },
  lang: String
});

const emit = defineEmits(['update', 'stop', 'start', 'refresh', 'syncMods']);

const activeTab = ref('config');
const t = TRANSLATIONS;
const isSaving = ref(false);
const showPassword = ref(false);
const showToken = ref(false);
const applyingToken = ref(false);
const isRefreshing = ref(false);
const isSyncingMods = ref(false);

// 获取 appStore 以访问所有模组信息
const appStore = useAppStore();
const settingsStore = useSettingsStore();

// 配置页里的全局令牌
const globalToken = computed(() => settingsStore.settings.global_cluster_token || '');

// 存档使用的模组：名称按需向后端解析（带 mtime 缓存），存在性用本地模组 id 判断
const archiveMods = computed(() => {
  const serverModIds = props.server?.mod_ids || [];
  const names = appStore.modNames || {};
  const available = appStore.localModIds || [];

  return serverModIds.map(modId => ({
    id: modId,
    // 名称未解析出来时退回显示 id
    name: names[modId] || modId,
    exists: available.includes(modId),
  }));
});

// 切到模组页（或切换服务器/语言）时才解析名称
watch([() => props.server?.id, activeTab, () => props.lang], () => {
  if (activeTab.value === 'mods') {
    appStore.loadModNames(props.server?.mod_ids || []);
  }
});

// 是否存在只存在于单侧的存档
const hasMismatchedSaves = computed(() => (props.saves || []).some(save => save.mismatched));

// 清理不一致存档（会删除文件，先二次确认）
const confirmRepairSaves = async () => {
  try {
    await ElMessageBox.confirm(
        t.repairSavesConfirm[props.lang],
        t.warning[props.lang],
        {
          confirmButtonText: t.confirm[props.lang],
          cancelButtonText: t.cancel[props.lang],
          type: 'warning',
        }
    );
    // appStore 会顺带刷新当前服务器的详情与存档列表
    await appStore.repairSaves(props.server.id);
  } catch (error) {
    // 用户取消
  }
};

// 使用 localConfig 来绑定表单，避免直接修改 prop
const localConfig = reactive({...props.server});

// 监听 server prop 变化，重置 localConfig (当切换选中的服务器时)
watch(() => props.server, (newServer) => {
  Object.assign(localConfig, newServer);
}, {deep: true});

const statusColor = computed(() => {
  if (props.server.status === 'running') return 'text-green-500';
  if (props.server.status === 'starting') return 'text-yellow-500';
  if (props.server.status === 'stopping') return 'text-yellow-500';
  return 'text-gray-400';
});

const handleUpdate = (updatedServer) => {
  emit('update', updatedServer);
};

const saveConfig = async () => {
  isSaving.value = true;
  try {
    const res = await tauriInvokeUtil('update_server_config_handler', {
      config: {
        server_id: props.server.id,
        max_players: localConfig.max_players,
        pvp: localConfig.pvp,
        pause_when_empty: localConfig.pause_when_empty,
        cluster_password: localConfig.cluster_password,
        cluster_description: localConfig.cluster_description,
        cluster_name: localConfig.cluster_name,
        // 空串表示清除本服务器的覆盖值（回退全局令牌）
        token: localConfig.cluster_token || '',
      }
    });

    if (res.code === 200) {
      ElMessage.success(t.updateConfigSuccess[props.lang]);
      // 留空保存时后端会写入全局令牌，因此这里也要按全局值回显
      const tokenFallback = useSettingsStore().settings.global_cluster_token;
      // 更新父组件的数据（令牌清空时归一为全局值，与文件内容一致）
      handleUpdate({
        ...props.server,
        max_players: localConfig.max_players,
        pvp: localConfig.pvp,
        pause_when_empty: localConfig.pause_when_empty,
        cluster_password: localConfig.cluster_password,
        cluster_description: localConfig.cluster_description,
        cluster_name: localConfig.cluster_name,
        cluster_token: localConfig.cluster_token || tokenFallback || null,
      });
    } else {
      ElMessage.error(res.message || t.updateConfigFailed[props.lang]);
    }
  } catch (error) {
    console.error('保存配置失败:', error);
    ElMessage.error(t.updateConfigFailed[props.lang]);
  } finally {
    isSaving.value = false;
  }
};

// 刷新当前服务器的信息（等同重新点一次这个存档：详情 + 存档列表）
const refreshServer = async () => {
  isRefreshing.value = true;
  try {
    await Promise.all([
      appStore.loadServerDetail(props.server.id),
      appStore.loadServerSaves(props.server.id),
    ]);
  } finally {
    isRefreshing.value = false;
  }
};

// 用配置页的全局令牌填入本服务器并写入文件（只提交 token，不动 cluster.ini）
const applyGlobalToken = async () => {
  if (!globalToken.value) return;

  applyingToken.value = true;
  try {
    const res = await tauriInvokeUtil('update_server_config_handler', {
      config: {
        server_id: props.server.id,
        // 空串表示"用设置页的全局令牌"
        token: '',
      }
    });

    if (res.code === 200) {
      ElMessage.success(t.useGlobalTokenSuccess[props.lang]);
      handleUpdate({...props.server, cluster_token: globalToken.value});
    } else {
      ElMessage.error(res.message || t.updateConfigFailed[props.lang]);
    }
  } catch (error) {
    console.error('Apply global token failed:', error);
    ElMessage.error(t.updateConfigFailed[props.lang]);
  } finally {
    applyingToken.value = false;
  }
};

const toggleStatus = () => {  if (props.server.status === 'running' || props.server.status === 'stopping') {
    // 停止服务器（stopping 状态也可再次点击关闭）
    emit('stop', props.server.id);
  } else {
    // 启动服务器
    emit('start', props.server.id);
  }
};

// 翻译季节
const getSeasonText = (season) => {
  const seasonMap = {
    spring: t.spring[props.lang],
    summer: t.summer[props.lang],
    autumn: t.autumn[props.lang],
    winter: t.winter[props.lang]
  };
  return seasonMap[season] || season;
};

// 翻译阶段
const getPhaseText = (phase) => {
  const phaseMap = {
    day: t.dayPhase[props.lang],
    dusk: t.dusk[props.lang],
    night: t.night[props.lang]
  };
  return phaseMap[phase] || t.unknown[props.lang];
};

// 删除存档 - 二次确认
const confirmDeleteSave = async (phaseFileName) => {
  try {
    await ElMessageBox.confirm(
        t.confirmDeleteSave[props.lang],
        t.warning[props.lang],
        {
          confirmButtonText: t.delete[props.lang],
          cancelButtonText: t.cancel[props.lang],
          type: 'warning',
          confirmButtonClass: 'el-button--danger'
        }
    );

    // 确认后执行删除
    await deleteSave(phaseFileName);
  } catch (error) {
    // 用户取消删除
    console.log('User cancelled delete');
  }
};

// 执行删除存档
const deleteSave = async (phaseFileName) => {
  try {
    const res = await tauriInvokeUtil('delete_archive_phase_handler', {
      request: {
        server_id: props.server.id,
        phase_file_name: phaseFileName
      }
    });

    if (res.code === 200) {
      ElMessage.success(t.deleteArchiveSuccess[props.lang]);
      // 通知父组件刷新数据
      emit('refresh');
    } else {
      ElMessage.error(res.message || t.deleteArchiveFailed[props.lang]);
    }
  } catch (error) {
    console.error('删除存档失败:', error);
    ElMessage.error(t.deleteArchiveFailed[props.lang]);
  }
};

// 同步模组
const syncMods = async () => {
  isSyncingMods.value = true;
  try {
    const res = await tauriInvokeUtil('sync_client_mods_to_server_handler', {});
    if (res.code === 200) {
      ElMessage.success(t.syncModsSuccess[props.lang]);
      // 同步后目录 mtime 变化，重新解析名称；并通知父组件刷新本地模组列表
      await appStore.loadModNames(props.server?.mod_ids || []);
      emit('syncMods');
    } else {
      ElMessage.error(res.message || t.syncModsFailed[props.lang]);
    }
  } catch (error) {
    console.error('同步模组失败:', error);
    ElMessage.error(t.syncModsFailed[props.lang]);
  } finally {
    isSyncingMods.value = false;
  }
};
</script>