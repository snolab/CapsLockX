if (!CapsLockX){
    MsgBox, % "本模块只在 CapsLockX 下工作"
    ExitApp
}
CapsLockX_AppendHelp( CapsLockX_LoadHelpFrom(CapsLockX_THIS_MODULE_HELP_FILE_PATH))
Return

#If !!(CapsLockXMode & CM_FN) || !!(CapsLockXMode & CM_CapsLockX)

F1:: Launch_App1 ; 打开我的电脑
F2:: Launch_App2 ; 计算器
F3:: Browser_Home ; 
F4:: Launch_Media ; 默认是 Windows Media Player

F5:: Send {Media_Play_Pause}
F6:: Send {Media_Prev}
F7:: Send {Media_Next}
F8:: Send {Media_Stop}

F9:: Send {Volume_Up}
F10:: Send {Volume_Down}
F11:: Send {Volume_Mute}
F12:: Send {Launch_App2}

; 关掉屏幕显示
Pause:: SendMessage,0x112,0xF170,2,,Program Manager