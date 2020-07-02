#SingleInstance, Force
CoordMode, Mouse, Screen
CoordMode, Pixel, Screen
SetTitleMatchMode RegEx

; 解决多屏 DPI 问题
DllCall("Shcore.dll\SetProcessDpiAwareness", "UInt", 2)
global wx, wy, w, h


MouseGetPos, mx, my, win, fControl
WinGetTitle, title, ahk_id %win%
TrayTip  已对 %title% 启用自动抢红包, 按F12结束。

WinGetPos, wx, wy, w, h, ahk_id %win%

SetTimer, qianghongbao, 16

Return

F12::
    TrayTip  已结束对 %title% 的自动抢红包。
    ExitApp
    Return

Pause::
    TrayTip  己刷新
    WinGetPos, wx, wy, w, h, ahk_id %win%
    Return

Pos2Long(x, y){
    Return x | (y << 16)
}
qianghongbao:
    ActionColor  := 0xD13D4B
    Action2Color := 0xFFEDBF
    cx := 93, cy := h - 415 - 40
    sx := wx + cx, sy := wy + cy
    PixelGetColor, clr, %sx%, %sy%, RGB
    if (ActionColor == clr) {
        ;Sleep, 200
        wParam := 0x0000
        lParam := Pos2Long(cx, cy)
        SendMessage 0x0201, %wParam%, %lParam%, %fControl%, ahk_id %win%
        SendMessage 0x0202, %wParam%, %lParam%, %fControl%, ahk_id %win%

        ; SendMessage 0x0201, %wParam%, %lParam%, %fControl%, ahk_id %win%
        ; SendMessage 0x0202, %wParam%, %lParam%, %fControl%, ahk_id %win%

        ; SendMessage 0x0201, %wParam%, %lParam%, %fControl%, ahk_id %win%
        ; SendMessage 0x0202, %wParam%, %lParam%, %fControl%, ahk_id %win%

        TrayTip , %clr% 发现目标!
    }Else if (Action2Color == clr){
        ;Sleep, 200
        wParam := 0x0000
        lParam := Pos2Long(cx, cy)

        SendMessage 0x0201, %wParam%, %lParam%, %fControl%, ahk_id %win%
        SendMessage 0x0202, %wParam%, %lParam%, %fControl%, ahk_id %win%

        TrayTip , %clr% 发现高级目标!

        Send !s
    }Else{
        ; TrayTip , %clr%
    }
    Return