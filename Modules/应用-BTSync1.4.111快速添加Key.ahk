; save with utf8-with-bom
; MsgBox, 适用于 BTSync 1.4.111 Windows 版本，`键触发功能，F12退出
    ; MouseClick, WhichButton , X, Y, ClickCount, Speed, D|U

Return
#IfWinActive, BitTorrent Sync Beta ahk_exe BTSync.exe

`::
    TrayTip 添加一条目录（如未成功，请把鼠标指向设置图标）
    Click Left
    Sleep 100
    Send {Tab}{Tab}{Enter}
    Sleep 600
    Send ^v^{Left}+{Home}{Del}{Tab}{Enter}
    Sleep 400
    Send ^v{Enter}{Enter}
    Return

F12:: ExitApp