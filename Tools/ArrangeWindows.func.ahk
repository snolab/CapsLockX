; ========== CapsLockX ==========
; Note：Save as UTF-8 with BOM please
; 名称：自动排列窗口
; 作者：snomiao (snomiao@gmail.com)
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.07.04
; 版权：Copyright © 2020 Snowstar Laboratory. All Rights Reserved.
; 许可证 LICENCE: GNU GPLv3 ( https://www.gnu.org/licenses/gpl-3.0.html )
; ========== CapsLockX ==========
；
ArrangeWindows(A_Args[1])
Return

ArrangeWindows(arrangeFlags = "0"){
    arrangeFlags += 0 ; string to number

    ARRANGE_MAXWINDOW := 1
    ARRANGE_STACKED := 2 ; if not then arrange SIDE_BY_SIDE

    ; 常量定义
    WS_EX_TOOLWINDOW  := 0x00000080
    WS_CAPTION        := 0x00C00000
    WS_EX_NOANIMATION := 0x04000000
    WS_EX_NOACTIVATE  := 0x08000000
    WS_POPUP          := 0x80000000

    n:=0
    listOfWindow := ""
    WinGet, id, List,,,
    Loop, %id%
    {
        this_id := id%A_Index%
        WinGet, this_pid, PID, ahk_id %this_id%
        WinGet, style, style, ahk_id %this_id%
        WinGetTitle, this_title, ahk_id %this_id%
        WinGetClass, this_class, ahk_id %this_id%
        ; Process, , PID-or-Name [, Param3]
        If (this_class == "TXGuiFoundation"){  ;  && this_process=="QQ.exe"
            ; 白名单
        }else{
            ; 黑名单
            ; ; 跳过无标题窗口
            ; if !(style & WS_CAPTION)
            ;     Continue
            ; ; 跳过工具窗口
            ; if (style & WS_EX_TOOLWINDOW)
            ;     Continue 
            ; 跳过弹出窗口
            if (style & WS_POPUP)
                Continue 
            ; 排除空标题窗口
            ; If (!RegExMatch(this_title, ".+"))
            ; Continue 
            ; If (this_class == "Progman")
            ; Continue ; 排除 Win10 的常驻窗口管理器
        }

        ; 跳过最小化的窗口
        WinGet, minmax, minmax, ahk_id %this_id%
        if(minmax == -1)
            continue
        if (minmax == 1){
            if(!(arrangeFlags & ARRANGE_MAXWINDOW)){
                continue
            }
        }
        listOfWindow .= "ahk_pid " this_pid " ahk_id " this_id "`n" ; . "`t" . this_title . "`n"
        n += 1
        
        ; debug
        ; WinActivate, ahk_id %this_id%
        ; WinGetClass, this_class, ahk_id %this_id%
        ; WinGetPos, X, Y, Width, Height, ahk_id %this_id%
        ; MsgBox, 4, , Visiting All Windows`n%A_Index% of %id%`nahk_id %this_id%`n%X% %Y% %Width% %Height%`nahk_class %this_class%`n%this_title%`n`nContinue?
        ; IfMsgBox, NO, break
    }
    ; 按 pid 和 hwnd 排列，所以这样排出来的窗口的顺序是稳定的
    Sort listOfWindow
    if(arrangeFlags & ARRANGE_STACKED){
        ArrangeWindowsStacked(listOfWindow, arrangeFlags)
    }else{
        ArrangeWindowsSideBySide(listOfWindow, arrangeFlags)
    }
}
FastResizeWindow(hWnd, x, y, w, h){
    ; 如有必要则还原最大化的窗口
    WinGet, minmax, minmax, ahk_id %hWnd%
    if (minmax == 1){
        WinRestore, ahk_id %hWnd%
    }
    
    ; ref: [SetWindowPos function (winuser.h) - Win32 apps | Microsoft Docs]( https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos )
    SWP_NOACTIVATE := 0x0010
    SWP_ASYNCWINDOWPOS:= 0x4000
    HWND_TOPMOST := -1
    HWND_BOTTOM := 1
    HWND_TOP := 0
    HWND_NOTOPMOST := -2
    SWP_NOMOVE := 0x0002
    SWP_NOSIE := 0x0001
    flags := SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS
    ; 先置顶
    DllCall("SetWindowPos"
        , "UInt", hWnd ;handle
        , "UInt", HWND_TOPMOST ; z-index
        , "Int", 0 ; x
        , "Int", 0 ; y
        , "Int", 0 ; width
        , "Int", 0 ; height
        , "UInt", flags | SWP_NOSIZE | SWP_NOMOVE ) ; SWP_ASYNCWINDOWPOS
    ; 再排到正确的位置上
    DllCall("SetWindowPos"
        , "UInt", hWnd ;handle
        , "UInt", HWND_NOTOPMOST ; z-index
        , "Int", x
        , "Int", y
        , "Int", w
        , "Int", h
        , "UInt", flags ) ; SWP_ASYNCWINDOWPOS
}
ArrangeWindowsSideBySide(listOfWindow, arrangeFlags = "0"){
    arrangeFlags += 0 ; string to number
    n := StrSplit(listOfWindow, "`n", "`r").Count() - 1
    ; calc rows and cols
    ; shorten edge first
    if (A_ScreenWidth <= A_ScreenHeight){
        ; row more than cols
        col := Sqrt(n) | 0
        row := Ceil(n / col)
    }else{
        ; col more than rows
        row := Sqrt(n) | 0
        col := Ceil(n / row)
    }
    
    k:=0
    Loop, Parse, listOfWindow, `n
    {
        this_id := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")

        ; shorten edge first
        if (A_ScreenWidth <= A_ScreenHeight){
            ; row first
            nx := Mod(k, col)
            ny := k / col | 0
            size_x := A_ScreenWidth / col
            size_y := A_ScreenHeight / row
        }else{
            ; col first
            nx := k / row | 0
            ny := Mod(k, row)
            size_x := A_ScreenWidth / col
            size_y := A_ScreenHeight / row
        }
        x := nx * size_x
        y := ny * size_y
        
        rx:=x-8, ry:=y, rsize_x:=size_x+16, rsize_y:=size_y+8 ; 填满边界不留缝隙
        FastResizeWindow(this_id, rx, ry, rsize_x, rsize_y)
        k+=1
    }
}

ArrangeWindowsStacked(listOfWindow, arrangeFlags = "0"){
    arrangeFlags += 0 ; string to number
    n := StrSplit(listOfWindow, "`n", "`r").Count() - 1
    
    k := 1
    dx := 64
    dy := 64
    w := A_ScreenWidth - 2 * dx - n * dx + dx
    h := A_ScreenHeight - 2 * dy - n * dy + dy
    Loop, Parse, listOfWindow, `n
    {
        this_id := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")

        x := k * dx
        y := k * dy
        
        FastResizeWindow(this_id, x, y, w, h)
        k+=1
    }
}
