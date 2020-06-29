If(!CapsLockX)
    ExitApp
Return

#IfWinActive ahk_class Qt5QWindowIcon ahk_exe Telegram.exe
    !a::
        MouseClick, Right
        Sleep, 40
        Send, {Down 1}{Enter}
        Return
    !m::
        MouseClick, Right
        Sleep, 40
        Send, {Down 4}{Enter}
        sleep, 40
        send, {Enter}
        Sleep, 300
        MouseClick, Right
        Sleep, 40
        Send, {Down 1}{Enter}
        Return