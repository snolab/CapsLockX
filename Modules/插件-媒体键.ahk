if (!CapsLockX){
    MsgBox, % "本模块只在 CapsLockX 下工作"
    ExitApp
}

CapsLockX_AppendHelp("
(
媒体键
| CapsLockX + F1 | 打开：我的电脑
| CapsLockX + F2 | 打开：计算器
| CapsLockX + F3 | 打开：浏览器主页
| CapsLockX + F4 | 打开：媒体库（默认是 Windows Media Player）
| CapsLockX + F5 | 播放：暂停/播放
| CapsLockX + F6 | 播放：上一首
| CapsLockX + F7 | 播放：下一首
| CapsLockX + F8 | 播放：停止
| CapsLockX + F9 | 音量加
| CapsLockX + F10 | 音量减
| CapsLockX + F11 | 静音
| CapsLockX + F12 | 打开：计算器
| CapsLockX + Pause |
)")

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