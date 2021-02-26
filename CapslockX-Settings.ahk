; save with UTF8 with DOM

ReadUpdateConfig(field, varName, defaultValue := 0, comment = "# "){
    IniRead, %varName%, %CapslockXConfigPath%, %field%, %varName%, %defaultValue%
    content := %varName% ; 千层套路
    IniWrite, %content%, %CapslockXConfigPath%, %field%, %varName%
    return content
}

if (!!CapslockXConfigPath) {
    ; 基本设定
    ; [Core]
    ; 是否请求提升权限
    global T_AskRunAsAdmin := ReadUpdateConfig("Core", "T_AskRunAsAdmin", 1)
    ; 使用 Insert、CapsLock、Space、ScrollLock 作为引导键的开关，默认只启用 Space 和 CapsLock
    global T_SpaceAsXKey := ReadUpdateConfig("Core", "T_SpaceAsXKey", 1)
    global T_CapsLockAsXKey := ReadUpdateConfig("Core", "T_CapsLockAsXKey", 1)
    global T_InsertAsXKey := ReadUpdateConfig("Core", "T_InsertAsXKey", 0)
    global T_ScrollLockAsXKey := ReadUpdateConfig("Core", "T_ScrollLockAsXKey", 0)
    ; 是否使用 ScrollLock 灯来显示 CapsLockX 状态（不建议
    global T_UseScrollLockLight := ReadUpdateConfig("Core", "T_UseScrollLockLight", 0)
    ; 是否使用 CapsLockX 灯来显示 CapsLockX 状态（强烈不建议 
    global T_UseCapsLockLight := ReadUpdateConfig("Core", "T_UseCapsLockLight", 0)
    ; 是否开启声音提示（默认不开）
    global T_SwitchSound := ReadUpdateConfig("Core", "T_SwitchSound", 0)
    global T_SwitchSoundOn := ReadUpdateConfig("Core", "T_SwitchSoundOn", "./Data/NoteG.mp3")
    global T_SwitchSoundOff := ReadUpdateConfig("Core", "T_SwitchSoundOff", "./Data/NoteC.mp3")
    ; 不同模式下的拖盘图标
    global T_SwitchTrayIconDefault := ReadUpdateConfig("Core", "T_SwitchTrayIconDefault", "./Data/XIconWhite.ico")
    global T_SwitchTrayIconOn := ReadUpdateConfig("Core", "T_SwitchTrayIconOn", "./Data/XIconBlue.ico")
    global T_SwitchTrayIconOff := ReadUpdateConfig("Core", "T_SwitchTrayIconOff", "./Data/XIconWhite.ico")
    
    ; 禁用模块
    ; [T_Mouse] 鼠标模拟模块
    global TMouse_Disabled := ReadUpdateConfig("TMouse", "TMouse_Disabled", 0)    
    ; 使用 SendInput 方法提高模拟鼠标点击、移动性能
    global TMouse_SendInput := ReadUpdateConfig("TMouse", "TMouse_SendInput", 1)    
    ; 使用 Windows API 强势提升模拟鼠标移动性能
    global TMouse_SendInputAPI := ReadUpdateConfig("TMouse", "TMouse_SendInputAPI", 1)
    ; 启用用自动粘附各种按钮，编辑框
    global TMouse_StickyCursor := ReadUpdateConfig("TMouse", "TMouse_StickyCursor", 1)
    ; 撞上屏幕边界后停止加速
    global TMouse_StopAtScreenEdge := ReadUpdateConfig("TMouse", "TMouse_StopAtScreenEdge", 1)
    ; 屏幕 DPI 比率，自动计算得出，如果数值不对，才需要纠正
    global TMouse_DPIRatio := A_ScreenDPI / 96 / 3
    ; 鼠标加速度比率, 一般就改那个1，你想慢点就改成 0.8
    global TMouse_MouseSpeedRatio := TMouse_DPIRatio * 0.8
    ; 滚轮加速度比率, 一般就改那个1，你想慢点就改成 0.8
    global TMouse_WheelSpeedRatio := TMouse_DPIRatio * 0.8
}