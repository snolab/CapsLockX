; save with UTF8 with DOM

if(!!CapslockXConfigPath){
    ; 基本设定
    ; [Core]
    global T_AskRunAsAdmin
    IniRead, T_AskRunAsAdmin, %CapslockXConfigPath%, Core, T_AskRunAsAdmin, 1
    IniWrite, %T_AskRunAsAdmin%, %CapslockXConfigPath%, Core, T_AskRunAsAdmin
    
    ; 修改CapsLockX触发键
    global T_CapsLockXKey
    IniRead, T_CapsLockXKey, %CapslockXConfigPath%, Core, T_CapsLockXKey, CapsLock
    IniWrite, %T_CapsLockXKey%, %CapslockXConfigPath%, Core, T_CapsLockXKey
    
    ; 是否使用 ScrollLock 灯来显示 CapsLockX 状态
    global T_UseScrollLockLight
    IniRead, T_UseScrollLockLight, %CapslockXConfigPath%, Core, T_UseScrollLockLight, 0
    IniWrite, %T_UseScrollLockLight%, %CapslockXConfigPath%, Core, T_UseScrollLockLight
    global T_UseScrollLockAsCapsLock
    IniRead, T_UseScrollLockAsCapsLock, %CapslockXConfigPath%, Core, T_UseScrollLockAsCapsLock, 0
    IniWrite, %T_UseScrollLockAsCapsLock%, %CapslockXConfigPath%, Core, T_UseScrollLockAsCapsLock
    
    ; 是否开启声音提示
    global T_SwitchSound
    IniRead, T_SwitchSound, %CapslockXConfigPath%, Core, T_SwitchSound, 0
    IniWrite, %T_SwitchSound%, %CapslockXConfigPath%, Core, T_SwitchSound
    global T_SwitchSoundOn
    IniRead, T_SwitchSoundOn, %CapslockXConfigPath%, Core, T_SwitchSoundOn, ./Data/NoteG.mp3
    IniWrite, %T_SwitchSoundOn%, %CapslockXConfigPath%, Core, T_SwitchSoundOn
    global T_SwitchSoundOff
    IniRead, T_SwitchSoundOff, %CapslockXConfigPath%, Core, T_SwitchSoundOff, ./Data/NoteC.mp3
    IniWrite, %T_SwitchSoundOff%, %CapslockXConfigPath%, Core, T_SwitchSoundOff
    
    ; 不同模式下的拖盘图标
    global T_SwitchTrayIconDefault
    IniRead, T_SwitchTrayIconDefault, %CapslockXConfigPath%, Core, T_SwitchTrayIconDefault, ./Data/XIconWhite.ico
    IniWrite, %T_SwitchTrayIconDefault%, %CapslockXConfigPath%, Core, T_SwitchTrayIconDefault
    global T_SwitchTrayIconOn
    IniRead, T_SwitchTrayIconOn, %CapslockXConfigPath%, Core, T_SwitchTrayIconOn, ./Data/XIconBlue.ico
    IniWrite, %T_SwitchTrayIconOn%, %CapslockXConfigPath%, Core, T_SwitchTrayIconOn
    global T_SwitchTrayIconOff
    IniRead, T_SwitchTrayIconOff, %CapslockXConfigPath%, Core, T_SwitchTrayIconOff, ./Data/XIconWhite.ico
    IniWrite, %T_SwitchTrayIconOff%, %CapslockXConfigPath%, Core, T_SwitchTrayIconOff
    
    ; 禁用模块
    global TMouse_Disabled
    IniRead, TMouse_Disabled, %CapslockXConfigPath%, TMouse, TMouse_Disabled, 0
    IniWrite, %TMouse_Disabled%, %CapslockXConfigPath%, TMouse, TMouse_Disabled
    
    ; 使用 SendInput 方法提高模拟鼠标点击、移动性能
    global TMouse_SendInput
    IniRead, TMouse_SendInput, %CapslockXConfigPath%, TMouse, TMouse_SendInput, 1
    IniWrite, %TMouse_SendInput%, %CapslockXConfigPath%, TMouse, TMouse_SendInput
    
    ; 使用 Windows API 强势提升模拟鼠标移动性能
    global TMouse_SendInputAPI
    IniRead, TMouse_SendInputAPI, %CapslockXConfigPath%, TMouse, TMouse_SendInputAPI, 1
    IniWrite, %TMouse_SendInputAPI%, %CapslockXConfigPath%, TMouse, TMouse_SendInputAPI
    
    ; 启用用自动粘附各种按钮，编辑框
    global TMouse_StickyCursor
    IniRead, TMouse_StickyCursor, %CapslockXConfigPath%, TMouse, TMouse_StickyCursor, 1
    IniWrite, %TMouse_StickyCursor%, %CapslockXConfigPath%, TMouse, TMouse_StickyCursor
    
    ; 撞上屏幕边界后停止加速
    global TMouse_StopAtScreenEdge
    IniRead, TMouse_StopAtScreenEdge, %CapslockXConfigPath%, TMouse, TMouse_StopAtScreenEdge, 1
    IniWrite, %TMouse_StopAtScreenEdge%, %CapslockXConfigPath%, TMouse, TMouse_StopAtScreenEdge
    
    ; 屏幕 DPI 比率，自动计算得出，如果数值不对，才需要纠正
    global TMouse_DPIRatio := A_ScreenDPI / 96
    
    ; 鼠标加速度比率, 一般就改那个1，你想慢点就改成 0.8
    global TMouse_MouseSpeedRatio := TMouse_DPIRatio * 1
    
    ; 滚轮加速度比率, 一般就改那个1，你想慢点就改成 0.8
    global TMouse_WheelSpeedRatio := TMouse_DPIRatio * 0.8
}