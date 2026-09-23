import {defineStore} from 'pinia';
import {ref} from "vue";
import {listen} from '@tauri-apps/api/event';
import {tauriInvokeUtil} from "@/utils/tauriInvokeUtil.js";
import {ElMessage, ElMessageBox} from "element-plus";
import {TRANSLATIONS} from "@/constants.js";
import {useSettingsStore} from "./settingsStore.js";

export const useAppStore = defineStore('app', () => {
    // --- 列表数据（只含摘要，详情按需加载）---
    const clusters = ref([]);
    const servers = ref([]);

    // 本地可用的模组 id（workshop-N），用于判断服务器所需模组是否缺失
    const localModIds = ref([]);
    // 模组 id -> 名称（按需解析，后端有 mtime 缓存）
    const modNames = ref({});

    // --- 选中状态与详情 ---
    const selectedClusterId = ref(null);
    const selectedServerId = ref(null);
    const serverDetail = ref(null); // ServerDetailVO
    const serverSaves = ref([]);    // ArchivePhaseVO[]
    const isConverting = ref(false);

    // --- 获取翻译文本 ---
    const getTranslation = (key) => {
        const settingsStore = useSettingsStore();
        const lang = settingsStore?.language || 'zh';
        return TRANSLATIONS[key]?.[lang] || TRANSLATIONS[key]?.['zh'] || key;
    };

    // --- 列表加载 ---
    // 全部按配置页里的路径查询（自动发现只在配置页的「自动扫描」按钮里发生）

    const loadMods = async () => {
        const res = await tauriInvokeUtil('list_mods_handler', {}, {showLoading: false});
        if (res.code !== 200 || !res.data) return;
        if (res.data.found) {
            localModIds.value = res.data.mod_ids || [];
        } else {
            ElMessage.warning(getTranslation('dst_mod_scan_warn_message'));
            localModIds.value = [];
        }
    };

    // 按需解析模组名称（后端按目录 mtime 缓存，重复调用很便宜）
    const loadModNames = async (modIds) => {
        if (!modIds || modIds.length === 0) return;
        const res = await tauriInvokeUtil('resolve_mod_names_handler', {
            request: {mod_ids: modIds}
        }, {showLoading: false});
        if (res.code !== 200 || !res.data) return;

        const next = {...modNames.value};
        for (const item of res.data) {
            next[item.mod_id] = item.name || item.mod_id;
        }
        modNames.value = next;
    };

    const loadClusters = async () => {
        const res = await tauriInvokeUtil('list_clusters_handler', {}, {showLoading: false});
        if (res.code !== 200 || !res.data) return;
        clusters.value = res.data.items || [];
        if (!res.data.found) {
            ElMessage.warning(getTranslation('dst_archive_scan_warn_message'));
        }
    };

    // 与 clusters 共用同一根目录，扫描失败的提示由 loadClusters 负责
    const loadServers = async () => {
        const res = await tauriInvokeUtil('list_servers_handler', {}, {showLoading: false});
        if (res.code !== 200 || !res.data) return;
        servers.value = res.data.items || [];
    };

    // --- 详情加载 ---

    const loadServerDetail = async (id) => {
        const res = await tauriInvokeUtil('get_server_handler', {id}, {showLoading: false});
        // 请求期间用户可能已切换选择，丢弃过期结果
        if (selectedServerId.value !== id) return;
        serverDetail.value = res.code === 200 && res.data ? res.data : null;
    };

    const loadServerSaves = async (id) => {
        const saves = await fetchServerSaves(id);
        // 请求期间用户可能已切换选择，丢弃过期结果
        if (selectedServerId.value !== id) return;
        serverSaves.value = saves;
    };

    const fetchServerSaves = async (id) => {
        const res = await tauriInvokeUtil('get_server_saves_handler', {id}, {showLoading: false});
        return res.code === 200 && res.data ? res.data : [];
    };

    // --- Actions ---

    // 服务器状态变化事件（后端崩档监视推送；事件名为跨端契约，与后端 watch::STATUS_EVENT 一致）
    const STATUS_EVENT = 'server-status-changed';

    const initStatusEvents = async () => {
        return listen(STATUS_EVENT, (event) => {
            const {server_id: id, status, reason} = event.payload || {};
            if (!id || !status) return;

            applyStatus(id, status);

            // 崩档互保触发的关闭：用常驻弹窗告知（必须手动确认，避免错过）
            if (reason === 'peer-crashed') {
                notifyPersistent(`${getTranslation('peerCrashedMessage')}（${id}）`);
            } else if (reason === 'startup-failed') {
                notifyPersistent(`${getTranslation('startupFailedMessage')}（${id}）`);
            }
        });
    };

    // 常驻提示：贴在页面最上方居中，需用户点击「知道了」才消失
    const notifyPersistent = (message) => {
        ElMessageBox.alert(message, getTranslation('warning'), {
            confirmButtonText: getTranslation('gotIt'),
            type: 'warning',
            customClass: 'persistent-alert',
        }).catch(() => {
            // 用户用右上角关闭 / ESC 关掉，忽略
        });
    };

    // 转换存档为服务器
    const convertCluster = async () => {
        if (!selectedClusterId.value) return;
        // token 由后端取配置页的全局令牌；这里只做前置提示，避免白跑一次
        if (!useSettingsStore().settings.global_cluster_token) {
            ElMessage.warning(getTranslation('tokenRequired'));
            return;
        }

        const cluster = clusters.value.find(c => c.id === selectedClusterId.value);
        if (!cluster) return;

        isConverting.value = true;
        try {
            const res = await tauriInvokeUtil('convert_cluster_to_server_handler', {
                request: {
                    cluster_id: selectedClusterId.value,
                    has_caves: cluster.has_caves || false
                }
            });

            if (res.code === 200) {
                const serverId = res.data;

                // 转换只负责复制存档；模组同步是独立的写动作，这里显式再调一次
                await tauriInvokeUtil('sync_client_mods_to_server_handler', {});
                await loadMods();

                // 用后端列表替换本地拼接，随后按需拉取新服务器的详情
                selectedClusterId.value = null;
                await loadServers();
                selectedServerId.value = serverId;
                serverDetail.value = null;
                serverSaves.value = [];
                await Promise.all([loadServerDetail(serverId), loadServerSaves(serverId)]);

                ElMessage.success(getTranslation('convertSuccess'));
            }
        } catch (error) {
            console.error("Conversion failed", error);
            ElMessage.error(getTranslation('convertFailed') + ': ' + error.message);
        } finally {
            isConverting.value = false;
        }
    };

    // 将服务器转为本地存档
    const convertServerToCluster = async () => {
        if (!selectedServerId.value) return;

        isConverting.value = true;
        try {
            const res = await tauriInvokeUtil('convert_server_to_cluster_handler', {
                request: {
                    server_id: selectedServerId.value
                }
            });

            if (res.code === 200) {
                const clusterId = res.data;

                selectedServerId.value = null;
                serverDetail.value = null;
                serverSaves.value = [];
                await loadClusters();
                selectedClusterId.value = clusterId;

                ElMessage.success(getTranslation('convertToLocalSuccess'));
            }
        } catch (error) {
            console.error("Convert to local failed", error);
            ElMessage.error(getTranslation('convertFailed') + ': ' + error.message);
        } finally {
            isConverting.value = false;
        }
    };

    // 更新服务器信息（详情对象与列表项是两个对象，需一并同步）
    const updateServer = (updated) => {
        if (serverDetail.value?.id === updated.id) {
            serverDetail.value = {...serverDetail.value, ...updated};
        }

        const index = servers.value.findIndex(s => s.id === updated.id);
        if (index !== -1) {
            servers.value[index] = {
                ...servers.value[index],
                status: updated.status ?? servers.value[index].status,
                cluster_name: updated.cluster_name ?? servers.value[index].cluster_name,
            };
        }
    };

    // 删除服务器
    const deleteServer = async (id) => {
        try {
            const res = await tauriInvokeUtil('delete_server_handler', {
                serverId: id
            });

            if (res.code === 200) {
                if (selectedServerId.value === id) {
                    selectedServerId.value = null;
                    serverDetail.value = null;
                    serverSaves.value = [];
                }
                await loadServers();
                ElMessage.success(getTranslation('deleteSuccess'));
            }
        } catch (error) {
            console.error("Delete failed", error);
            ElMessage.error(getTranslation('deleteFailed') + ': ' + error.message);
        }
    };

    // 选中服务器：只在此处按需加载详情与存档列表
    const changeSelectedServer = async (id) => {
        const next = selectedServerId.value === id ? null : id;
        selectedServerId.value = next;
        serverDetail.value = null;
        serverSaves.value = [];

        if (next) {
            await Promise.all([loadServerDetail(next), loadServerSaves(next)]);
        }
    };

    const changeSelectedCluster = (id) => {
        selectedClusterId.value = selectedClusterId.value === id ? null : id;
    };

    // 同步状态到详情与列表
    const applyStatus = (id, status) => {
        if (serverDetail.value?.id === id) {
            serverDetail.value = {...serverDetail.value, status};
        }

        const index = servers.value.findIndex(s => s.id === id);
        if (index !== -1) {
            servers.value[index] = {...servers.value[index], status};
        }
    };

    // 停止服务器
    const stopServer = async (id) => {
        try {
            const res = await tauriInvokeUtil('stop_server_handler', {
                serverId: id
            });

            if (res.code === 200) {
                // 过渡态 stopping：实际关闭后由后端监视任务推送 stopped
                applyStatus(id, 'stopping');
                ElMessage.success(getTranslation('stopSuccess'));
            }
        } catch (error) {
            console.error("Stop server failed", error);
            ElMessage.error(getTranslation('stopFailed') + ': ' + error.message);
        }
    };

    // 检查当前服务器的模组是否都存在
    const checkServerModsExist = (serverId) => {
        const serverModIds = serverDetail.value?.id === serverId ? (serverDetail.value.mod_ids || []) : [];
        if (serverModIds.length === 0) {
            return {allExist: true, missingMods: []};
        }

        const available = localModIds.value || [];
        const missingMods = serverModIds.filter(modId => !available.includes(modId));

        return {
            allExist: missingMods.length === 0,
            missingMods
        };
    };

    // 只警告不拦截的确认框：用户选择继续返回 true
    const confirmContinue = async (message, title) => {
        try {
            await ElMessageBox.confirm(message, title, {
                confirmButtonText: getTranslation('continueStart'),
                cancelButtonText: getTranslation('cancel'),
                type: 'warning',
            });
            return true;
        } catch (error) {
            // 用户取消
            return false;
        }
    };

    // 填模板里的 {count} 占位
    const withCount = (key, count) => getTranslation(key).replace('{count}', String(count));

    // 启动服务器
    const startServer = async (id) => {
        // 启动前的两类风险只提示、不拦截，由用户决定是否继续

        // 1. 缺失模组
        const {allExist, missingMods} = checkServerModsExist(id);
        if (!allExist) {
            const goOn = await confirmContinue(
                withCount('missingModsMessage', missingMods.length),
                getTranslation('missingModsTitle')
            );
            if (!goOn) return;
        }

        // 2. 存档不一致（只存在于单侧的存档无法加载）
        const saves =
            selectedServerId.value === id ? (serverSaves.value || []) : await fetchServerSaves(id);
        const mismatchedCount = saves.filter(save => save.mismatched).length;
        if (mismatchedCount > 0) {
            const goOn = await confirmContinue(
                withCount('mismatchedSavesOnStartMessage', mismatchedCount),
                getTranslation('warning')
            );
            if (!goOn) return;
        }

        try {
            // 更新服务器状态为 starting
            applyStatus(id, 'starting');

            // 调用后端启动服务器
            const res = await tauriInvokeUtil('start_server_handler', {
                serverId: id
            });

            if (res.code === 200) {
                // 启动成功后，更新状态为 running
                applyStatus(id, 'running');
                ElMessage.success(getTranslation('startSuccess'));
            } else {
                // 业务错误（例如未配置 cluster token）：回滚状态，避免卡在 starting
                applyStatus(id, 'stopped');
            }
        } catch (error) {
            console.error("Start server failed", error);
            // 启动失败，恢复状态为 stopped
            applyStatus(id, 'stopped');
            ElMessage.error(getTranslation('startFailed') + ': ' + error.message);
        }
    };

    // 清理 Master/Caves 不一致的存档（会删除文件，调用前需二次确认）
    const repairSaves = async (id) => {
        const res = await tauriInvokeUtil('repair_server_saves_handler', {id});
        if (res.code === 200) {
            // 只刷新当前选中服务器的详情/存档，列表项不受影响
            if (selectedServerId.value === id) {
                await Promise.all([loadServerDetail(id), loadServerSaves(id)]);
            }
            ElMessage.success(getTranslation('repairSavesSuccess'));
        }
    };

    return {
        clusters,
        servers,
        localModIds,
        modNames,
        selectedClusterId,
        selectedServerId,
        serverDetail,
        serverSaves,
        isConverting,
        loadMods,
        loadModNames,
        loadClusters,
        loadServers,
        loadServerDetail,
        loadServerSaves,
        initStatusEvents,
        updateServer,
        convertCluster,
        convertServerToCluster,
        changeSelectedServer,
        changeSelectedCluster,
        deleteServer,
        stopServer,
        startServer,
        repairSaves,
    };
});
