import {createApp} from 'vue'
import App from './App.vue'

const app = createApp(App)

// 引入全局 CSS
import './index.css'

import * as ElementPlusIconsVue from '@element-plus/icons-vue'

for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
    app.component(key, component)
}

/**
 * 批量注册自定义图标组件
 * 遍历icons目录下的所有Vue组件文件，并将其注册为全局组件
 */
const iconModules = import.meta.glob('./components/icons/*.vue', {eager: true})
for (const path in iconModules) {
    const name = path.match(/\/([^/]+)\.vue$/)?.[1]
    if (name) {
        app.component(name, iconModules[path].default)
    }
}

import {createPinia} from "pinia";

const pinia = createPinia()

app.use(pinia)

import router from './router'

app.use(router)

app.mount('#app')
