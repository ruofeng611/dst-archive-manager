/**
 * PostCSS 配置对象
 * 定义了项目中使用的 PostCSS 插件配置
 *
 * @type {Object}
 * @property {Object} plugins - PostCSS 插件配置对象
 * @property {Object} plugins['@tailwindcss/postcss'] - Tailwind CSS PostCSS 插件配置
 */
export default {
    plugins: {
        '@tailwindcss/postcss': {},
    },
}
