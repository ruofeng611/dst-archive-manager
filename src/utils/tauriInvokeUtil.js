import {ElLoading, ElMessage} from 'element-plus'
import {invoke} from "@tauri-apps/api/core";

let loadingInstance = null;

/**
 * 封装 Tauri 的 invoke 调用，自动处理加载和消息提示
 * @param {string} cmd - Rust 端命令名（函数名）
 * @param {Object} args - 传递给 Rust 的参数对象
 * @param {Object} options - 配置选项
 * @param {boolean} [options.showLoading=true] - 是否显示加载遮罩
 * @param {boolean} [options.showMessage=true] - 是否显示失败消息
 * @returns {Promise<any>} - invoke 返回的结果
 */
export async function tauriInvokeUtil(cmd, args = {}, options = {}) {
    const {
        showLoading = true,
        showMessage = true,
    } = options;

    // 显示全局 Loading
    if (showLoading) {
        loadingInstance = ElLoading.service();
    }

    try {
        const result = await invoke(cmd, args);

        // 业务逻辑：检查返回码
        if (result.code !== 200) {
            if (showMessage) {
                ElMessage.error(result.message || '操作失败');
            }
        }

        return result;
    } catch (error) {
        if (showMessage) {
            // 统一错误提示
            ElMessage.error('请求失败，请重试');
        }
        console.error(error);
        throw error;
    } finally {
        // 统一关闭 Loading
        if (loadingInstance) {
            loadingInstance.close();
            loadingInstance = null;
        }
    }
}