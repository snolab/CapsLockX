
If(!CapslockX)
    ExitApp
Return

#IfWinActive 个会话 ahk_class TXGuiFoundation ahk_exe QQ.exe
    !f::
        ; CoordMode, Mouse, Client
        CoordMode, Mouse, Relative
        Click 24, 24
        return
    ; 接收文件
    !r::
        Send 1!s+{Tab 9}{Space}!s
        Return

#IfWinActive ahk_class TXGuiFoundation ahk_exe QQ.exe
    ; mute
    !m::
        Send {RButton}{Down 2}{Right}{Up}{Enter}
        MouseMove, 0, -86, 0, R
        Return
    ; 接收文件
    !r::
        Send 1!s+{Tab 8}{Space}!s
        Return