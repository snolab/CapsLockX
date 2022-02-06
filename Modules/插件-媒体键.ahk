if (!CapsLockX) {
    MsgBox, % "本模块只在 CapsLockX 下工作"
    ExitApp
}
CapsLockX_AppendHelp( CapsLockX_LoadHelpFrom(CapsLockX_THIS_MODULE_HELP_FILE_PATH))
Return

#if !!(CapsLockXMode & CM_FN) || !!(CapsLockXMode & CM_CapsLockX)

F1:: Launch_App1 ; 打开我的电脑
F2:: Launch_App2 ; 计算器
F3:: Browser_Home ; 主页
F4:: Launch_Media ; 启动播放器

F5:: Send {Media_Play_Pause} ; 暂停
F6:: Send {Media_Prev} ; 上一首
F7:: Send {Media_Next} ; 下一首
F8:: Send {Media_Stop} ; 停止

F9:: Send {Volume_Up}    ; 音量-
F10:: Send {Volume_Down} ; 音量+
F11:: Send {Volume_Mute} ; 音量0
F12:: Send {Launch_App2} ; 启动计算器

; 关掉屏幕显示
Pause:: SendMessage, 0x112, 0xF170, 2, , Program Manager