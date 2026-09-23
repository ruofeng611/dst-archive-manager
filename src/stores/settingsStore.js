// src/stores/settingsStore.js

import {defineStore} from 'pinia'
import {ref} from 'vue'
import {useDark, useToggle} from '@vueuse/core'
import {tauriInvokeUtil} from '@/utils/tauriInvokeUtil.js'

// 应用配置字段（与后端 AppSettingsVO 一致）
const emptySettings = () => ({
    dst_client_path: '',
    dst_server_path: '',
    workshop_path: '',
    archive_root: '',
    global_cluster_token: '',
})

export const useSettingsStore = defineStore('settings', () => {
    // --- 主题管理 ---
    // useDark 会自动监听系统设置，并给 html 标签添加 class="dark"
    const isDark = useDark({
        storageKey: 'app-theme-appearance',
    })
    const toggleTheme = useToggle(isDark)

    // --- 语言管理 ---
    // 语言值从后端数据库加载，默认 'zh'
    const language = ref('zh')

    // 从后端加载语言设置
    const loadLanguage = async () => {
        try {
            const res = await tauriInvokeUtil('get_app_language_handler', {}, { showLoading: false })
            if (res.code === 200 && res.data) {
                language.value = res.data
            }
        } catch (e) {
            console.error('加载语言设置失败:', e)
        }
    }

    // 保存语言设置到后端
    const saveLanguage = async (lang) => {
        try {
            await tauriInvokeUtil('set_app_language_handler', {
                request: { language: lang }
            }, { showLoading: false })
        } catch (e) {
            console.error('保存语言设置失败:', e)
        }
    }

    // --- 应用配置 ---
    // 唯一来源是后端数据库，这里只是运行期镜像
    const settings = ref(emptySettings())

    // 从后端加载配置
    const loadSettings = async () => {
        // 清理历史残留：早期版本把 token 与路径存在 localStorage 的 app-storage 键里
        localStorage.removeItem('app-storage')
        try {
            const res = await tauriInvokeUtil('get_settings_handler', {}, { showLoading: false })
            if (res.code === 200 && res.data) {
                const data = res.data
                settings.value = {
                    dst_client_path: data.dst_client_path ?? '',
                    dst_server_path: data.dst_server_path ?? '',
                    workshop_path: data.workshop_path ?? '',
                    archive_root: data.archive_root ?? '',
                    global_cluster_token: data.global_cluster_token ?? '',
                }
            }
        } catch (e) {
            console.error('加载应用配置失败:', e)
        }
    }

    // 保存配置（局部更新：只传要改的字段，空串表示清除该项）
    const saveSettings = async (patch) => {
        const res = await tauriInvokeUtil('update_settings_handler', { request: patch })
        if (res.code === 200) {
            await loadSettings()
        }
        return res
    }

    return {
        isDark,
        toggleTheme,
        language,
        loadLanguage,
        saveLanguage,
        settings,
        loadSettings,
        saveSettings,
    }
})
