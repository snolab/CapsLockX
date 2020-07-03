; Save as UTF-8 with BOM please
; 自动排列窗口
; author: snomiao@gmail.com
; Copyright © 2020
; LICENCE: GNU GPLv3
;
ArrangeWindows(A_Args[1])
Return
ArrangeWindows(arrangeFlags = "0"){
    arrangeFlags += 0 ; string to number

    ARRANGE_MAXWINDOW := 1
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
        this_id := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")
        rx:=x-8, ry:=y-8, rsize_x:=size_x+16+8, rsize_y:=size_y+16 ; 填满边界不留缝隙
        
        ; 如有必要则还原最大化的窗口
        WinGet, minmax, minmax, ahk_id %this_id%
        if (minmax == 1){
            WinRestore, ahk_id %this_id%
        }
        

        hWnd := this_id ; WinExist("ahk_id %this_id%") ;get handle
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
            , "UInt", -1   ; HWND_TOPMOST
            , "Int", rx  ;x
            , "Int", ry ;y
            , "Int", rsize_x ;width
            , "Int", rsize_y ;height
            , "UInt", flags | SWP_NOSIZE | SWP_NOMOVE ) ; SWP_ASYNCWINDOWPOS
        ; 再排到正确的位置上
        DllCall("SetWindowPos"
            , "UInt", hWnd ;handle
            , "UInt", -2 
            , "Int", rx  ;x
            , "Int", ry  ;y
            , "Int", rsize_x ;width
            , "Int", rsize_y ;height
            , "UInt", flags ) ; SWP_ASYNCWINDOWPOS
        k+=1
    }
}
