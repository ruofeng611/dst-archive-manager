import {createRouter, createWebHistory} from 'vue-router'
import {routes} from "@/router/routes.js";

/**
 * 创建Vue路由器实例
 * @param {Object} options - 路由器配置选项
 * @param {Object} options.history - 路由历史模式配置
 * @param {Array} options.routes - 路由配置数组
 * @returns {router} 返回配置好的Vue路由器实例
 */
const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes
})

export default router
