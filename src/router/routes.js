// 定义路由清单
const routeConfigs = [
    {
        path: '/',
        name: 'Home',
        component: () => import('@/views/HomeView.vue'),
        meta: {title: '主页', icon: 'House', description: '主页'}
    },
]

// 导出路由数组（给 router 使用）
export const routes = routeConfigs