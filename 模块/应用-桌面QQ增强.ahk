If(!CapslockX)
    ExitApp
Return

#IfWinActive ahk_class TXGuiFoundation ahk_exe QQ.exe
    ; mute
    !m::
        Send {RButton}{Down 2}{Right}{Up}{Enter}
        MouseMove, 0, -86, 0, R
        Return
    