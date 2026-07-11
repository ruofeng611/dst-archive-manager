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
                placeholder="Optional"
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

        <div class="space-y-3">
          <label class="flex items-center cursor-pointer">
            <input
                type="checkbox"
                v-model="localConfig.pvp"
                class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
            />
            <span class="ml-2 text-sm font-medium text-gray-700 dark:text-gray-300">{{ t.pvpMode[lang] }}</span>
          </label>

          <label class="flex items-center cursor-pointer">
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
        <div v-if="!server.archive_phase_vo || server.archive_phase_vo.length === 0"
             class="text-center py-8 text-gray-500 dark:text-gray-400 italic">
          No save files found for {{ server.id }}
        </div>
        <div v-for="(save, index) in server.archive_phase_vo" :key="save.phase_file_name"
             class="p-3 bg-gray-50 dark:bg-gray-900/50 rounded border border-gray-200 dark:border-gray-700">
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <FileDigit :size="16" class="text-blue-600 dark:text-blue-400"/>
              <h4 class="font-medium text-sm text-gray-800 dark:text-gray-200">
                {{ save.phase_file_name }}
                <span v-if="index === 0" class="ml-2 text-xs text-green-600 dark:text-green-400 font-semibold">({{
                    t.latest[lang]
                  }})</span>
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
import {ref, computed, watch, reactive, onUnmounted} from 'vue';
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
  RotateCcw
} from 'lucide-vue-next';
import {TRANSLATIONS} from '@/constants';
import {tauriInvokeUtil} from '@/utils/tauriInvokeUtil.js';
import {ElMessage, ElMessageBox} from 'element-plus';
import {useAppStore} from '@/stores/appStore.js';

const props = defineProps({
  server: Object,
  lang: String
});

const emit = defineEmits(['update', 'stop', 'start', 'refresh', 'syncMods']);

const activeTab = ref('config');
const t = TRANSLATIONS;
const isSaving = ref(false);
const showPassword = ref(false);
const isSyncingMods = ref(false);

// 获取 appStore 以访问所有模组信息
const appStore = useAppStore();

// 计算属性：处理存档模组信息，与所有模组融合
const archiveMods = computed(() => {
  const serverModIds = props.server?.mod_ids || [];

  if (serverModIds.length === 0) {
    return [];
  }

  // 获取所有已扫描到的模组
  const allMods = appStore.mods || [];

  return serverModIds.map(modId => {
    // 在已扫描的模组中查找
    // modId 是 workshop-XXX 格式
    const foundMod = allMods.find(m => {
      // 如果是纯数字文件夹，补全为 workshop-数字 格式进行比较
      const normalizedFolderName = /^\d+$/.test(m.folder_name)
          ? `workshop-${m.folder_name}`
          : m.folder_name;
      return normalizedFolderName === modId;
    });

    if (foundMod) {
      return {
        id: modId,
        name: foundMod.mod_name,
        exists: true
      };
    } else {
      return {
        id: modId,
        name: modId,  // 如果找不到，就用 ID 作为名字
        exists: false
      };
    }
  });
});

// 使用 localConfig 来绑定表单，避免直接修改 prop
const localConfig = reactive({...props.server});

// 定时器相关
let statusTimer = null;
let stoppedCount = 0;

// 查询服务器状态
const queryServerStatus = async () => {
  if (!props.server || !props.server.id) return;

  try {
    const res = await tauriInvokeUtil('query_server_status_handler', {serverId: props.server.id}, {showLoading: false});
    if (res.code === 200 && res.data) {
      const newStatus = res.data.status;

      if (newStatus === 'stopped') {
        stoppedCount++;
        // 首次查到已关闭 → 按钮变为"启动服务器"
        if (stoppedCount === 1) {
          emit('update', {...props.server, status: 'stopped'});
        }
        // 连续 3 次 stopped → 停止轮询
        if (stoppedCount >= 3) {
          stopStatusTimer();
        }
      } else {
        // 查到非 stopped（running）→ 重置计数，状态改回 stopping
        stoppedCount = 0;
        if (props.server.status !== 'stopping') {
          emit('update', {...props.server, status: 'stopping'});
        }
      }
    }
  } catch (error) {
    // 静默处理错误，避免频繁弹窗
    console.error('查询服务器状态失败:', error);
  }
};

// 启动定时器
const startStatusTimer = () => {
  stoppedCount = 0; // 重置计数
  if (statusTimer) return; // 已在运行
  // 立即执行一次
  queryServerStatus();
  // 每3秒查询一次
  statusTimer = setInterval(queryServerStatus, 3000);
};

// 停止定时器
const stopStatusTimer = () => {
  if (statusTimer) {
    clearInterval(statusTimer);
    statusTimer = null;
  }
};

// 组件卸载时停止定时器
onUnmounted(() => {
  stopStatusTimer();
});

// 当服务器切换时，停止旧定时器；若新服务器状态为 stopping，启动轮询
watch(() => props.server?.id, (newId, oldId) => {
  if (newId !== oldId) {
    stopStatusTimer();
    // 切换到新服务器后，检查是否需要恢复轮询
    if (props.server?.status === 'stopping') {
      startStatusTimer();
    }
  }
});

// 当状态变为 stopping 时，启动轮询确认服务器关闭
watch(() => props.server?.status, (newStatus) => {
  if (newStatus === 'stopping') {
    startStatusTimer();
  }
});

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
      }
    });

    if (res.code === 200) {
      ElMessage.success(t.updateConfigSuccess[props.lang]);
      // 更新父组件的数据
      handleUpdate({
        ...props.server,
        max_players: localConfig.max_players,
        pvp: localConfig.pvp,
        pause_when_empty: localConfig.pause_when_empty,
        cluster_password: localConfig.cluster_password,
        cluster_description: localConfig.cluster_description,
        cluster_name: localConfig.cluster_name,
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

const toggleStatus = () => {
  if (props.server.status === 'running' || props.server.status === 'stopping') {
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
      // 通知父组件重新扫描模组信息
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