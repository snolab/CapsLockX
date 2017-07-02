SetDefaultMouseSpeed 0
Return
#IfWinActive ahk_class TXGuiFoundation ahk_exe TIM.exe
    ^!F12:: ExitApp ; 退出脚本
    ^f::
        CoordMode, Mouse, Relative
        Click 300, 60
        Return
    !Home:: Click 300, 180
    ^PgDn:: Send ^{Tab}
    ^PgUp:: Send +^{Tab}
    !w:: Send +{Tab}{AppsKey}{Down 2}{Enter}
    !x::
        WinGetPos, x, y, w, h
        cx := (w - 400 - 300) / 2 + 400
        Click %cx% 250
        Return
    F4::
        Click right
        Click Rel, 235, 120
        MouseMove, -235, -120, 0, R
        Return
    F6::
        Click right
        Click Rel, 235, 263
        Click Rel, 329, 132
        MouseMove, -564, -395, 0, R
        Return
