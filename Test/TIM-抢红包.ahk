CoordMode, Mouse, Relative
CoordMode, Pixel, Relative
SetTitleMatchMode RegEx

; 解决多屏 DPI 问题
DllCall("Shcore.dll\SetProcessDpiAwareness", "UInt", 2)
global x, y, w, h

F12:: ExitApp


`::
    WinGetPos, x, y, w, h, 20146
    cx := 93, cy := h - 405
    ; x := 61, y := h - 329
    MouseMove % cx, cy

    ; ToolTip switched_on
    
    SetTimer, qianghongbao, 16
    Return
    ; Click
    ; Return

qianghongbao:
    dclr := 0xD13D4B
    PixelGetColor, clr, %cx%, %cy%, RGB
    ToolTip, %clr%
    if (clr == dclr){
        MouseMove % cx, cy
        Click
        Sleep, 32
        Click
        Sleep, 32
        Send ^{Enter}
        SetTimer, qianghongbao, Off
        ToolTip focus!!
    }
    Return