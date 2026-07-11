import {defineStore} from 'pinia';
import {ref, reactive} from "vue";
import {tauriInvokeUtil} from "@/utils/tauriInvokeUtil.js";
import {ElMessage, ElMessageBox} from "element-plus";
import {TRANSLATIONS} from "@/constants.js";
import {useSettingsStore} from "./settingsStore.js";

export const useAppStore = defineStore('app', () => {
    // --- 基础配置 ---
    const appInfo = reactive({
        serverToken: '',
        localSavePath: '',
        serverInstallPath: ''
    });

    // --- 业务数据 ---
    // mods 结构: [{ folder_name: 'workshop-123456', mod_name: 'Mod Name' }]
    const mods = ref([]);

    const clusters = ref([]);
    const servers = ref([]);

    // --- 选中状态 ---
    const selectedClusterId = ref(null);
    const selectedServerId = ref(null);
    const isConverting = ref(false);

    // --- 获取翻译文本 ---
    const getTranslation = (key) => {
        const settingsStore = useSettingsStore();
        const lang = settingsStore?.language || 'zh';
        return TRANSLATIONS[key]?.[lang] || TRANSLATIONS[key]?.['zh'] || key;
    };

    // --- Actions ---

    // 转换存档为服务器
    const convertCluster = async () => {
        if (!selectedClusterId.value) return;
        if (!appInfo.serverToken) {
            ElMessage.warning(getTranslation('tokenRequired'));
            return;
        }

        const cluster = clusters.value.find(c => c.id === selectedClusterId.value);
        if (!cluster) return;

        isConverting.value = true;
        try {
            // 调用后端接口
            const res = await tauriInvokeUtil('convert_cluster_to_server_handler', {
                request: {
                    cluster_id: selectedClusterId.value,
                    token: appInfo.serverToken,
                    has_caves: cluster.has_caves || false
                }
            });

            if (res.code === 200) {
                const serverId = res.data;

                // 创建新的服务器对象
                const newServer = {
                    ...cluster,
                    id: serverId, // 使用后端返回的新 ID
                    status: 'stopped',
                    config: {
                        name: cluster.cluster_name || cluster.id,
                        maxPlayers: cluster.max_players,
                        password: cluster.cluster_password,
                        description: cluster.cluster_description,
                        gameMode: cluster.game_mode,
                    },
                    // 将 mods 转换为服务器需要的格式
                    mods: mods.value.map(m => ({
                        id: m.folder_name,
                        name: m.mod_name,
                        enabled: true
                    }))
                };

                servers.value.push(newServer);
                selectedServerId.value = newServer.id;
                selectedClusterId.value = null;

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

                // 从服务器列表中找到对应服务器
                const server = servers.value.find(s => s.id === selectedServerId.value);

                // 创建新的 cluster 对象
                const newCluster = {
                    ...server,
                    id: clusterId,
                };

                clusters.value.push(newCluster);
                selectedClusterId.value = newCluster.id;
                selectedServerId.value = null;

                ElMessage.success(getTranslation('convertToLocalSuccess'));
            }
        } catch (error) {
            console.error("Convert to local failed", error);
            ElMessage.error(getTranslation('convertFailed') + ': ' + error.message);
        } finally {
            isConverting.value = false;
        }
    };

    // 更新服务器信息
    const updateServer = (updatedServer) => {
        const index = servers.value.findIndex(s => s.id === updatedServer.id);
        if (index !== -1) {
            servers.value[index] = updatedServer;
        }
    };

    // 删除服务器
    const deleteServer = async (id) => {
        try {
            // 调用后端接口
            const res = await tauriInvokeUtil('delete_server_handler', {
                serverId: id
            });

            if (res.code === 200) {
                // 从前端列表中移除
                servers.value = servers.value.filter(s => s.id !== id);
                if (selectedServerId.value === id) {
                    selectedServerId.value = null;
                }
                ElMessage.success(getTranslation('deleteSuccess'));
            }
        } catch (error) {
            console.error("Delete failed", error);
            ElMessage.error(getTranslation('deleteFailed') + ': ' + error.message);
        }
    };

    const changeSelectedServer = (id) => {
        selectedServerId.value = selectedServerId.value === id ? null : id;
    };

    const changeSelectedCluster = (id) => {
        selectedClusterId.value = selectedClusterId.value === id ? null : id;
    };

    // 停止服务器
    const stopServer = async (id) => {
        try {
            const res = await tauriInvokeUtil('stop_server_handler', {
                serverId: id
            });

            if (res.code === 200) {
                // 设置为过渡态 stopping，由轮询确认实际关闭后再改为 stopped
                const index = servers.value.findIndex(s => s.id === id);
                if (index !== -1) {
                    servers.value[index].status = 'stopping';
                }
                ElMessage.success(getTranslation('stopSuccess'));
            }
        } catch (error) {
            console.error("Stop server failed", error);
            ElMessage.error(getTranslation('stopFailed') + ': ' + error.message);
        }
    };

    // 检查服务器的模组是否都存在
    const checkServerModsExist = (serverId) => {
        const server = servers.value.find(s => s.id === serverId);
        if (!server) return {allExist: true, missingMods: []};

        const serverModIds = server.mod_ids || [];
        if (serverModIds.length === 0) {
            return {allExist: true, missingMods: []};
        }

        const missingMods = [];
        for (const modId of serverModIds) {
            const foundMod = mods.value.find(m => {
                // 如果是纯数字文件夹，补全为 workshop-数字 格式进行比较
                const normalizedFolderName = /^\d+$/.test(m.folder_name)
                    ? `workshop-${m.folder_name}`
                    : m.folder_name;
                return normalizedFolderName === modId;
            });
            if (!foundMod) {
                missingMods.push(modId);
            }
        }

        return {
            allExist: missingMods.length === 0,
            missingMods
        };
    };

    // 启动服务器
    const startServer = async (id) => {
        // 先检查模组是否都存在
        const {allExist} = checkServerModsExist(id);
        if (!allExist) {
            // 显示消息框提示用户
            await ElMessageBox.alert(
                getTranslation('missingModsMessage'),
                getTranslation('missingModsTitle'),
                {
                    confirmButtonText: getTranslation('confirm'),
                    type: 'warning'
                }
            );
            return;
        }

        try {
            // 更新服务器状态为 starting
            const index = servers.value.findIndex(s => s.id === id);
            if (index !== -1) {
                servers.value[index].status = 'starting';
            }

            // 调用后端启动服务器
            const res = await tauriInvokeUtil('start_server_handler', {
                serverId: id
            });

            if (res.code === 200) {
                // 启动成功后，更新状态为 running
                if (index !== -1) {
                    servers.value[index].status = 'running';
                }
                ElMessage.success(getTranslation('startSuccess'));
            }
        } catch (error) {
            console.error("Start server failed", error);
            // 启动失败，恢复状态为 stopped
            const index = servers.value.findIndex(s => s.id === id);
            if (index !== -1) {
                servers.value[index].status = 'stopped';
            }
            ElMessage.error(getTranslation('startFailed') + ': ' + error.message);
        }
    };

    // 同步模组
    const syncMods = async () => {
        try {
            const res = await tauriInvokeUtil('sync_client_mods_to_server_handler', {});
            if (res.code === 200) {
                ElMessage.success(getTranslation('syncModsSuccess'));
                return true;
            } else {
                ElMessage.error(res.message || getTranslation('syncModsFailed'));
                return false;
            }
        } catch (error) {
            console.error("Sync mods failed", error);
            ElMessage.error(getTranslation('syncModsFailed') + ': ' + error.message);
            return false;
        }
    };

    return {
        appInfo,
        mods,
        clusters,
        servers,
        selectedClusterId,
        selectedServerId,
        isConverting,
        updateServer,
        convertCluster,
        convertServerToCluster,
        changeSelectedServer,
        changeSelectedCluster,
        deleteServer,
        stopServer,
        startServer,
        syncMods
    };
}, {
    persist: {
        key: 'app-storage',
        storage: localStorage,
        pick: ['appInfo'] // 持久化配置和服务器列表
    }
});