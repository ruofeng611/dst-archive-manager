export const TRANSLATIONS = {
    appTitle: {en: 'DST Save Converter', zh: '饥荒存档转换器'},
    settings: {en: 'Settings', zh: '设置'},
    general: {en: 'General', zh: '常规'},
    back: {en: 'Back', zh: '返回'},
    rescan: {en: 'Rescan', zh: '重新扫描'},
    serverToken: {en: 'Server Token', zh: '服务器令牌'},
    serverTokenHint: {
        en: 'Stored in this server\'s cluster_token.txt. Saving with an empty value fills in the global token from Settings.',
        zh: '保存在该服务器的 cluster_token.txt 中；留空保存时会自动填入设置页的全局令牌。'
    },
    useGlobalToken: {en: 'Use Global Token', zh: '设置为全局token'},
    useGlobalTokenHint: {
        en: 'Fill in the global token from Settings and save it to this server.',
        zh: '用设置页的全局令牌填入并保存到该服务器。'
    },
    useGlobalTokenSuccess: {en: 'Global token applied.', zh: '已应用全局令牌。'},
    noGlobalToken: {en: 'No global token configured in Settings yet.', zh: '设置页还没有配置全局令牌。'},
    browse: {en: 'Browse', zh: '浏览'},
    clear: {en: 'Clear', zh: '清除'},

    // 配置页
    settingsHint: {
        en: 'Paths are stored in the app database and drive scanning, conversion and server startup.',
        zh: '路径保存在应用数据库中，扫描、转换与启动服务器都按此配置执行。'
    },
    clientInstallPath: {en: 'Client Install Path', zh: '客户端安装目录'},
    serverInstallPath: {en: 'Server Install Path', zh: '服务端安装目录'},
    workshopPath: {en: 'Workshop Content Path', zh: '创意工坊目录'},
    archiveRootPath: {en: 'Save Root Path', zh: '存档根目录'},
    globalToken: {en: 'Global Server Token', zh: '全局服务器令牌'},
    globalTokenHint: {
        en: 'Used when converting to a server and applied to all servers by default; per-server settings override it.',
        zh: '转为服务器时使用，并作为所有服务器的默认值；单个服务器的配置会覆盖它。'
    },
    saveSettingsSuccess: {en: 'Settings saved!', zh: '配置已保存！'},
    saveSettingsFailed: {en: 'Failed to save settings', zh: '配置保存失败'},
    browseFailed: {en: 'Failed to open folder dialog.', zh: '打开文件夹选择框失败。'},
    discoverPaths: {en: 'Auto-detect', zh: '自动扫描'},
    discoverPathsHint: {
        en: 'Detect Steam install paths from the registry and the save root from Documents.',
        zh: '从注册表检测 Steam 安装目录，并从“文档”检测存档根目录。'
    },
    discoverSuccess: {en: 'Detected paths were filled in and saved.', zh: '已自动填入并保存检测到的目录。'},
    discoverNothingFound: {en: 'No DST directories detected. Please select them manually.', zh: '未检测到任何 DST 目录，请手动选择。'},
    discoverFailed: {en: 'Auto-detect failed.', zh: '自动扫描失败。'},
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
    noPasswordSet: {en: 'No password set', zh: '未设置密码'},
    toggleTheme: {en: 'Toggle Theme', zh: '切换主题'},
    switchToChinese: {en: 'Switch to Chinese', zh: '切换到中文'},
    switchToEnglish: {en: 'Switch to English', zh: '切换到英文'},
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
        en: "Server install path is not configured or invalid. Please set it in Settings.",
        zh: "未配置服务端安装目录或路径无效，请到设置页配置。"
    },
    dst_archive_scan_warn_message: {
        en: "Save root path is not configured or invalid. Please set it in Settings.",
        zh: "未配置存档根目录或路径无效，请到设置页配置。"
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
    peerCrashedMessage: {
        en: 'One shard crashed; the other one was shut down.',
        zh: '检测到分片崩档，已关闭对端服务器。'
    },
    startupFailedMessage: {
        en: 'Server did not come up as expected; the started shard was shut down.',
        zh: '服务器未按期启动完成，已关闭已启动的分片。'
    },
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

    // 缺失模组提示（只警告不拦截，用户可选择继续启动）
    missingModsTitle: {en: 'Missing Mods', zh: '模组缺失'},
    missingModsMessage: {
        en: 'Some required mods are not ready (missing: {count}; download them in Steam first, then use "Sync Mods"). Start anyway?',
        zh: '服务器所需模组未全部就绪（缺失 {count} 个；请先在 Steam 下载，再到模组页点「同步模组」）。仍要启动吗？'
    },
    mismatchedSavesOnStartMessage: {
        en: '{count} of the saves exist on only one shard (Master or Caves) and cannot be loaded. Start anyway?',
        zh: '存在 {count} 个只在地面或洞穴单侧的存档，这类存档无法加载。仍要启动吗？'
    },
    continueStart: {en: 'Start Anyway', zh: '仍要启动'},
    gotIt: {en: 'Got it', zh: '知道了'},
    refresh: {en: 'Refresh', zh: '刷新'},
    confirm: {en: 'OK', zh: '确定'},
    openInFolder: {en: 'Open in Folder', zh: '在文件夹中打开'},

    // 列表空状态
    noClusters: {en: 'No local saves found. Check path settings.', zh: '未找到本地存档，请检查路径配置。'},
    noSaves: {en: 'No save files found.', zh: '未找到存档文件。'},

    // 清理不一致存档
    repairSaves: {en: 'Clean Mismatched Saves', zh: '清理不一致存档'},
    repairSavesConfirm: {
        en: 'Delete saves that exist on only one shard (Master or Caves)? These saves cannot be loaded.',
        zh: '将删除只存在于地面或洞穴单侧的存档（这类存档无法加载），确定继续吗？'
    },
    repairSavesSuccess: {en: 'Mismatched saves cleaned!', zh: '不一致存档已清理！'},
    mismatched: {en: 'Mismatched', zh: '不一致'},
    mismatchedSavesHint: {
        en: 'Greyed-out saves exist on only one shard and cannot be loaded.',
        zh: '标灰的存档只存在于地面或洞穴单侧，无法加载。'
    },
    mismatchedSaveTip: {
        en: 'This save exists on only one shard (Master or Caves).',
        zh: '该存档只存在于地面或洞穴单侧。'
    },
};