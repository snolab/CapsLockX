; save with UTF8 with DOM
CapsLockX_ConfigSet(field, varName, setValue, comment := ""){
    content := setValue
    ; 对配置自动重新排序
    IniDelete, %CapsLockXConfigPath%, %field%, %varName%
    if(comment){
        IniDelete, %CapsLockXConfigPath%, %field%, %varName%#注释
        IniWrite, %comment%, %CapsLockXConfigPath%, %field%, %varName%#注释
    }
    IniWrite, %content%, %CapsLockXConfigPath%, %field%, %varName%
    return content
}

CapsLockX_Config(field, varName, defaultValue, comment := ""){
    IniRead, %varName%, %CapsLockXConfigPath%, %field%, %varName%, %defaultValue%
    content := %varName% ; 千层套路XD
    ; 对配置自动重新排序
    IniDelete, %CapsLockXConfigPath%, %field%, %varName%
    if(comment){
        IniDelete, %CapsLockXConfigPath%, %field%, %varName%#注释
        IniWrite, %comment%, %CapsLockXConfigPath%, %field%, %varName%#注释
    }
    IniWrite, %content%, %CapsLockXConfigPath%, %field%, %varName%
    return content
}

if (!!CapsLockXConfigPath){
    ; 基本设定
    ; [Core]
    global T_AskRunAsAdmin := CapsLockX_Config("Core", "T_AskRunAsAdmin", 1, "是否请求提升权限（例如模拟鼠标等功能需要管理员权限才能正常运行，如果不需要可以关掉）")
    global T_XKeyAs := CapsLockX_Config("Core", "T_XKeyAs", 1, "使用 Insert、CapsLock、Space、ScrollLock 作为引导键的开关，默认只启用 Space 和 CapsLock")
    global T_XKeyAsSpace := CapsLockX_Config("Core", "T_XKeyAsSpace", 1, "使用 Space 作为引导键")
    global T_XKeyAsCapsLock := CapsLockX_Config("Core", "T_XKeyAsCapsLock", 1, "使用 CapsLock 作为引导键")
    global T_XKeyAsInsert := CapsLockX_Config("Core", "T_XKeyAsInsert", 0, "使用 Insert 作为引导键")
    global T_XKeyAsScrollLock := CapsLockX_Config("Core", "T_XKeyAsScrollLock", 0, "使用 ScrollLock 作为引导键")
    global T_XKeyAsRAlt := CapsLockX_Config("Core", "T_XKeyAsRAlt", 0, "使用 右 Alt 作为引导键")
    global T_UseScrollLockLight := CapsLockX_Config("Core", "T_UseScrollLockLight", 0, "是否使用 ScrollLock 灯来显示 CapsLockX 状态（不建议")
    global T_UseCapsLockLight := CapsLockX_Config("Core", "T_UseCapsLockLight", 0, "是否使用 CapsLockX 灯来显示 CapsLockX 状态（强烈不建议")
    global T_SwitchSound := CapsLockX_Config("Core", "T_SwitchSound", 0, "是否开启声音提示（默认不开）")
    global T_SwitchSoundOn := CapsLockX_Config("Core", "T_SwitchSoundOn", "./Data/NoteG.mp3", "CapsLockX按下声音提示路径")
    global T_SwitchSoundOff := CapsLockX_Config("Core", "T_SwitchSoundOff", "./Data/NoteC.mp3", "CapsLockX弹起声音提示路径")
    ; 不同模式下的拖盘图标
    ; global T_SwitchTrayIconDefault := CapsLockX_Config("Core", "T_SwitchTrayIconDefault", "./Data/XIconWhite.ico", "CapsLockX默认托盘显示图标，默认" "./Data/XIconWhite.ico")
    global T_SwitchTrayIconOff := CapsLockX_Config("Core", "T_SwitchTrayIconOff", "./Data/XIconWhite.ico", "CapsLockX弹起托盘显示图标，默认" "./Data/XIconWhite.ico")
    global T_SwitchTrayIconOn := CapsLockX_Config("Core", "T_SwitchTrayIconOn", "./Data/XIconBlue.ico", "CapsLockX按下托盘显示图标，默认" "./Data/XIconBlue.ico")
}