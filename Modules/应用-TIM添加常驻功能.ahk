
if (!CapsLockX)
    ExitApp
Return

TIM_MouseShift(times){
    dy := 300 / 8
    changey := dy * times
    MouseMove 0, changey, 1, R
}
#IfWinActive ahk_class TXGuiFoundation ahk_exe TIM.exe
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
        ; Send {Space}
        ; MouseMove 20, 180, 0, R
        ; Sleep 200
        ; MouseClick, Left
        ; MouseMove 200, 60, 0, R
        Return
    !Up::
        TIM_MouseShift(-1)
        Return
    !y::
        WinGetPos, X, Y, W, H, A
        X1 := X, Y1 := Y, X2 := X + W, Y2 := Y + H
        CoordMode, Pixel, Screen
        CoordMode, Mouse, Screen
        Loop {
            FILE := "./Modules/TIM同意按钮.png"
            If ( !FileExist(FILE) ){
                Msgbox 文件不存在：%FILE%
                Return
            }

            ImageSearch, OX, OY, X1, Y1, X2, Y2, %FILE%
            finded := OX || OY
            If (finded){
                Y1 := OY + 1
                Click %OX%, %OY%
            }
        } Until !finded
        Return
    !Down::
        TIM_MouseShift(1)
        Return
    !Enter::
        Click
        Return
    !h::
        CoordMode, ToolTip, Relative

        WinGetPos, X, Y, W, H, A

        x := 117 + 1
        y := 30 + 1
        ToolTip, f, x, y, 2
        
        x := W -20 + 1
        y := 70 + 1
        ToolTip, q, x, y, 3
        Return
    !h Up::
        ToolTip
        Return
    ^PgDn:: Send ^{Tab}
    ^PgUp:: Send +^{Tab}
    ; !w:: Send +{Tab}{AppsKey}{Down 2}{Enter}
    ; !x::
    ;     WinGetPos, x, y, w, h
    ;     cx := (w - 400 - 300) / 2 + 400
    ;     Click %cx% 250
    ;     Return