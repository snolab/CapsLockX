; save with utf8-with-bom
; MsgBox, 适用于 BTSync 1.4.111 Windows 版本，`键触发功能，F12退出
; MouseClick, WhichButton, X, Y, ClickCount, Speed, D|U

Return
#IfWinActive, BitTorrent Sync Beta ahk_exe BTSync.exe

!d::
    ; 添加一条带Key目录
    ; 例如 C:\Users\snomiao\同步\神key#BCWHZRSLANR64CGPTXRE54ENNSIUE5SMO
    ; 这样格式命名的文件夹会自动拆出Key来填到软件里

    ; CoordMode, Mouse, Screen
    ; TrayTip 添加一条带Key目录（如未成功，请把鼠标指向设置图标）
    WinGetPos, X, Y, W, H, A
    ClickX := X + W - 50
    ClickY := Y + 80
    ; MouseMove, %ClickX%, %ClickY%
    MouseClick, Left, %ClickX%, %ClickY%
    Sleep 100
    Send {Tab}{Tab}{Enter}
    Sleep 600
    Send ^v^{Left}+{Home}{Del}{Tab}{Enter}
    Sleep 400
    Send ^v{Enter}{Enter}
Return
