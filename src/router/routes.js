// 定义路由清单
const routeConfigs = [
    {
        path: '/',
        name: 'Home',
        component: () => import('@/views/HomeView.vue'),
        meta: {title: '主页', icon: 'House', description: '主页'}
    },
    {
        path: '/settings',
        name: 'Settings',
        component: () => import('@/views/SettingsView.vue'),
        meta: {title: '设置', icon: 'Setting', description: '应用配置'}
    },
]

// 导出路由数组（给 router 使用）
export const routes = routeConfigs
