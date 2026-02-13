// src/stores/settingsStore.js

import {defineStore} from 'pinia'
import {ref} from 'vue'
import {useDark, useToggle} from '@vueuse/core'

export const useSettingsStore = defineStore('settings', () => {
    // --- 主题管理 ---
    // useDark 会自动监听系统设置，并给 html 标签添加 class="dark"
    const isDark = useDark({
        storageKey: 'app-theme-appearance',
    })
    const toggleTheme = useToggle(isDark)

    // --- 语言管理 ---
    const language = ref('zh')

    return {
        isDark,
        toggleTheme,
        language,
    }
}, {
    //配置持久化策略
    persist: {
        key: 'settings-storage', // 存储的 key 名
        storage: localStorage,
        pick: ['language'], // 只持久化 `language` 字段
    }
})