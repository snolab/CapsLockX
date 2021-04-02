
if(!CapsLockX)
    ExitApp
Return

TIM_MouseShift(times){
    dy := 300 / 8
    changey := dy * times
    MouseMove 0, changey, 1, R
}

#If WinActive("ahk_class TXGuiFoundation ahk_exe TIM.exe")

^f::
    CoordMode, Mouse, Client
    x := 117 * TMouse_DPIRatio
    y := 30 * TMouse_DPIRatio
    Click %x%, %y%
Return
!q::
    CoordMode, Mouse, Relative
    WinGetPos, X, Y, W, H, A
    x := W - 20
    y := 70
    Click %x%, %y%
    TIM_MouseShift(1)
Return
!b::
    CoordMode, Mouse, Relative
    MouseClick, Right
    Sleep 50
    Send {Up 2}
    Sleep 50
    Send {Right}
    Sleep 50
    Send {Up}
    Sleep 50
    Send {Enter}
Return
!Up::
    TIM_MouseShift(-1)
Return
!Down::
    TIM_MouseShift(1)
Return
!Enter::
    Click
Return
^PgDn:: Send ^{Tab}
^PgUp:: Send +^{Tab}
