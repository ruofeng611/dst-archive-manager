<template>
  <!--
    ElConfigProvider 提供全局配置：
    - locale: 语言包
    - size: 组件默认尺寸
    - z-index: 弹窗层级初始值
  -->
  <el-config-provider :locale="locale" :size="'default'" :z-index="3000">

    <!-- 全局容器：确保高度撑满，背景色跟随主题 -->
    <!-- 这里的 text-primary 和 bg-primary 是我们在 index.css 定义的 CSS 变量 -->
    <div class="min-h-screen w-full font-sans antialiased text-(--text-primary) bg-(--bg-primary)">
      <router-view/>
    </div>

  </el-config-provider>
</template>

<script setup>
import {computed, onMounted, onUnmounted} from 'vue'
import {useSettingsStore} from '@/stores/settingsStore.js'
import {useAppStore} from '@/stores/appStore.js'

// Element Plus 语言包
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import en from 'element-plus/es/locale/lang/en'

// 引入 Element Plus 暗黑模式必要的 CSS 变量
import 'element-plus/theme-chalk/dark/css-vars.css'

const settingsStore = useSettingsStore()
const appStore = useAppStore()

let unlistenStatus = null

// 语言与配置是会话级数据，在这里读一次即可：
// 各页面的后端接口都直接从数据库读配置，不依赖前端镜像，所以无需每次进页面重查。
onMounted(async () => {
  await settingsStore.loadLanguage()
  await settingsStore.loadSettings()

  // 服务器状态由后端崩档监视推送事件，前端不轮询
  unlistenStatus = await appStore.initStatusEvents()
})

onUnmounted(() => {
  if (unlistenStatus) {
    unlistenStatus()
    unlistenStatus = null
  }
})

// 计算属性：将 store 中的字符串语言代码映射为 Element Plus 的对象
const locale = computed(() => {
  switch (settingsStore.language) {
    case 'en':
      return en
    case 'zh':
    default:
      return zhCn
  }
})

// isDark 的逻辑已经被 VueUse 在 store 初始化时自动执行了
// 它会自动给 <html> 添加 class="dark"，Tailwind 和 Element Plus 都会自动响应
</script>