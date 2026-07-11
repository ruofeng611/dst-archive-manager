export const TRANSLATIONS = {
    appTitle: {en: 'DST Save Converter', zh: '饥荒存档转换器'},
    settings: {en: 'Settings', zh: '设置'},
    general: {en: 'General', zh: '常规'},
    serverToken: {en: 'Server Token', zh: '服务器令牌'},
    localSavePath: {
        en: 'Local Save Path (Select the folder containing "DoNotStarveTogether")',
        zh: '本地存档路径（请选择包含 “DoNotStarveTogether” 文件夹的目录）'
    },
    serverPath: {
        en: 'Server Program Path (Select the folder containing "Don\'t Starve Together Dedicated Server")',
        zh: '服务器程序路径（请选择包含 “Don\'t Starve Together Dedicated Server” 文件夹的目录）'
    },
    browse: {en: 'Browse', zh: '浏览'},
    localSaves: {en: 'Local Saves (Clusters)', zh: '本地存档 (Clusters)'},
    serverList: {en: 'Dedicated Servers', zh: '专用服务器列表'},
    convert: {en: 'Convert to Server', zh: '转为服务器'},
    convertToLocal: {en: 'Convert to Local', zh: '转为本地存档'},
    serverDetails: {en: 'Server Details', zh: '服务器详情'},
    start: {en: 'Start Server', zh: '启动服务器'},
    stop: {en: 'Stop Server', zh: '关闭服务器'},
    status: {en: 'Status', zh: '状态'},
    config: {en: 'Configuration', zh: '配置'},
    mods: {en: 'Mods', zh: '模组'},
    saves: {en: 'Save Files', zh: '存档文件'},
    console: {en: 'Console', zh: '控制台'},
    serverName: {en: 'Server Name', zh: '服务器名称'},
    maxPlayers: {en: 'Max Players', zh: '最大人数'},
    password: {en: 'Password', zh: '密码'},
    gameMode: {en: 'Game Mode', zh: '游戏模式'},
    save: {en: 'Save', zh: '保存'},
    running: {en: 'Running', zh: '运行中'},
    stopped: {en: 'Stopped', zh: '已停止'},
    starting: {en: 'Starting', zh: '启动中'},
    stopping: {en: 'Stopping', zh: '关闭中'},
    noSelection: {en: 'Select a server to view details', zh: '请选择一个服务器以查看详情'},
    placeholderToken: {en: 'Enter your Klei Server Token', zh: '输入您的 Klei 服务器令牌'},
    darkMode: {en: 'Dark Mode', zh: '深色模式'},
    lightMode: {en: 'Light Mode', zh: '浅色模式'},
    language: {en: 'Language', zh: '语言'},
    delete: {en: 'Delete', zh: '删除'},
    confirmDelete: {en: 'Are you sure you want to delete this server?', zh: '确定要删除该服务器吗？'},

    dst_mod_scan_warn_message: {
        en: "Current folder does not contain any mod information. Please try another path.",
        zh: "当前文件夹下未找到相关的模组信息，请换个路径试试。"
    },
    dst_archive_scan_warn_message: {
        en: "Current folder does not contain any archive information. Please try another path.",
        zh: "当前文件夹下未找到相关的存档信息，请换个路径试试。"
    },
    dst_scan_prompt_message: {
        en: "Searching...",
        zh: "正在为您自动搜索..."
    },

    file_select_message: {
        en: "Please select a folder",
        zh: "请选择文件夹"
    },

    warning: {en: 'Warning', zh: '警告'},
    cancel: {en: 'Cancel', zh: '取消'},

    // 存档详情字段
    clusterName: {en: 'Cluster Name', zh: '世界名称'},
    clusterDescription: {en: 'Description', zh: '世界描述'},
    clusterPassword: {en: 'Cluster Password', zh: '世界密码'},
    pvpMode: {en: 'PVP Mode', zh: 'PVP模式'},
    pauseWhenEmpty: {en: 'Pause When Empty', zh: '无人时暂停'},
    day: {en: 'Day', zh: '天数'},
    season: {en: 'Season', zh: '季节'},
    phase: {en: 'Phase', zh: '阶段'},
    saveFileName: {en: 'Save File', zh: '存档文件'},

    // 季节翻译
    spring: {en: 'Spring', zh: '春'},
    summer: {en: 'Summer', zh: '夏'},
    autumn: {en: 'Autumn', zh: '秋'},
    winter: {en: 'Winter', zh: '冬'},

    // 阶段翻译
    dayPhase: {en: 'Day', zh: '白天'},
    dusk: {en: 'Dusk', zh: '黄昏'},
    night: {en: 'Night', zh: '夜晚'},
    unknown: {en: 'Unknown', zh: '未知'},

    // 布尔值翻译
    enabled: {en: 'Enabled', zh: '启用'},
    disabled: {en: 'Disabled', zh: '禁用'},

    // 其他
    latest: {en: 'Latest', zh: '最新'},

    // 消息提示
    tokenRequired: {en: 'Please enter server token first', zh: '请先输入服务器 Token'},
    convertSuccess: {en: 'Conversion successful!', zh: '转换成功！'},
    convertToLocalSuccess: {en: 'Converted to local archive successfully!', zh: '转为本地存档成功！'},
    convertFailed: {en: 'Conversion failed', zh: '转换失败'},
    deleteSuccess: {en: 'Deleted successfully!', zh: '删除成功！'},
    deleteFailed: {en: 'Delete failed', zh: '删除失败'},
    startSuccess: {en: 'Server started successfully!', zh: '服务器启动成功！'},
    startFailed: {en: 'Failed to start server', zh: '服务器启动失败'},
    stopSuccess: {en: 'Server stopped successfully!', zh: '服务器已停止！'},
    stopFailed: {en: 'Failed to stop server', zh: '服务器停止失败'},
    updateConfigSuccess: {en: 'Configuration updated successfully!', zh: '配置更新成功！'},
    updateConfigFailed: {en: 'Failed to update configuration', zh: '配置更新失败'},

    // 存档删除相关
    confirmDeleteSave: {
        en: 'Are you sure you want to delete this save file? This will remove the save from both Master and Caves (if exists).',
        zh: '确定要删除此存档文件吗？这将同时删除地面和洞穴（如果存在）的存档。'
    },
    deleteArchiveSuccess: {en: 'Archive deleted successfully!', zh: '存档删除成功！'},
    deleteArchiveFailed: {en: 'Failed to delete archive', zh: '删除存档失败'},

    // 模组同步相关
    syncMods: {en: 'Sync Mods', zh: '同步模组'},
    syncing: {en: 'Syncing', zh: '同步中'},
    noMods: {en: 'No mods detected', zh: '未检测到模组'},
    syncModsSuccess: {en: 'Mods synced successfully!', zh: '模组同步成功！'},
    syncModsFailed: {en: 'Failed to sync mods', zh: '模组同步失败'},

    // 存档模组相关
    noArchiveMods: {en: 'No mods added to this archive', zh: '该存档没有添加模组'},
    modMissing: {en: 'Missing', zh: '缺失'},
    archiveMods: {en: 'Archive Mods', zh: '存档模组'},

    // 缺失模组提示
    missingModsTitle: {en: 'Missing Mods', zh: '模组缺失'},
    missingModsMessage: {
        en: 'Missing mods detected. Please download them from Steam first, then click the "Sync Mods" button to sync.',
        zh: '缺失模组，请先前往Steam下载之后再按同步模组按钮同步'
    },
    confirm: {en: 'OK', zh: '确定'},
    openInFolder: {en: 'Open in Folder', zh: '在文件夹中打开'},
};