If(!CapsLockX){
    MsgBox, % "本模块只在 CapsLockX 下工作"
    ExitApp
}

CapslockXAddHelp("
(
媒体键
| CapslockX + F1    | 打开：我的电脑
| CapslockX + F2    | 打开：计算器
| CapslockX + F3    | 打开：浏览器主页
| CapslockX + F4    | 打开：媒体库（默认是 Windows Media Player）
| CapslockX + F5    | 播放：暂停/播放
| CapslockX + F6    | 播放：上一首
| CapslockX + F7    | 播放：下一首
| CapslockX + F8    | 播放：停止
| CapslockX + F9    | 音量加
| CapslockX + F10   | 音量减
| CapslockX + F11   | 静音
| CapslockX + F12   | 打开：计算器
| CapslockX + Pause |
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