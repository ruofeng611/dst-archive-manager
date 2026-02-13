/**
 * 延迟指定毫秒数后继续执行
 * @param {number} ms - 延迟的毫秒数
 * @returns {Promise<void>}
 */
export function delay(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}