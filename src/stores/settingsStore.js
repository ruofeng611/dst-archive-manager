// src/stores/settingsStore.js

import {defineStore} from 'pinia'
import {ref} from 'vue'
import {useDark, useToggle} from '@vueuse/core'
import {tauriInvokeUtil} from '@/utils/tauriInvokeUtil.js'

export const useSettingsStore = defineStore('settings', () => {
    // --- 主题管理 ---
    // useDark 会自动监听系统设置，并给 html 标签添加 class="dark"
    const isDark = useDark({
        storageKey: 'app-theme-appearance',
    })
    const toggleTheme = useToggle(isDark)

    // --- 语言管理 ---
    // 语言值从后端 key-value 表加载，默认 'zh'
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

    return {
        isDark,
        toggleTheme,
        language,
        loadLanguage,
        saveLanguage,
    }
})