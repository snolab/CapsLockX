; 排列窗口
; Save as UTF-8 with BOM please
; Copyright © 2017-2020 snomiao@gmail.com
; LICENCE: GNU GPLv3
; 
;
FileEncoding, utf-8
ArrangeWindows(resotreMaxWindows = 0){
    WS_CAPTION := 0x00C00000
    WS_EX_TOOLWINDOW := 0x00000080
    ; FileDelete, listOfWindow.txt
    WinGet, id, List,,,
    listOfWindow := ""
    n:=0
    Loop, %id%
    {
        this_id := id%A_Index%
        WinGet, style, style, ahk_id %this_id%
        ; if !(style & WS_CAPTION) ; if the window doesn't have a title bar
        ; Continue
        if (style & WS_EX_TOOLWINDOW) ; if the window doesn't have a title bar
            Continue ; 跳过工具窗口嗯，
        ; if (style & WS_EX_TOOLWINDOW) ; if the window doesn't have a title bar
        ;     Continue ; 跳过工具窗口
        WinGetTitle, this_title, ahk_id %this_id%
        If (!RegExMatch(this_title, ".+"))
            Continue ; 排除空标题窗口
        WinGetClass, this_class, ahk_id %this_id%
        If (this_class == "Progman")
            Continue ; 排除 Win10 的常驻窗口管理器
        
        listOfWindow .= this_id . "|" . this_title . "`n"
        
        WinGet, minmax, minmax, ahk_id %this_id%
        if(minmax == -1)
            continue
        if (minmax == 1){
            if(!resotreMaxWindows){
                continue
            }
        }
        listOfWindow .= this_id . "|" . this_title . "`n"
        n += 1
        
        ; debug
        ; WinActivate, ahk_id %this_id%
        ; WinGetClass, this_class, ahk_id %this_id%
        ; WinGetPos, X, Y, Width, Height, ahk_id %this_id%
        ; MsgBox, 4, , Visiting All Windows`n%A_Index% of %id%`nahk_id %this_id%`n%X% %Y% %Width% %Height%`nahk_class %this_class%`n%this_title%`n`nContinue?
        ; IfMsgBox, NO, break
    }
    
    
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
    size_x := A_ScreenWidth / col
    size_y := A_ScreenHeight / row

    k:=0
    Loop, %id%
    {
        this_id := id%A_Index%
        WinGet, style, style, ahk_id %this_id%
        ; if !(style & WS_CAPTION) ; if the window doesn't have a title bar
        ; Continue
        if (style & WS_EX_TOOLWINDOW) ; if the window doesn't have a title bar
            Continue ; 跳过工具窗口
        ; if (style & WS_EX_TOOLWINDOW) ; if the window doesn't have a title bar
        ;     Continue ; 跳过工具窗口
        WinGetTitle, this_title, ahk_id %this_id%
        If (!RegExMatch(this_title, ".+"))
            Continue ; 排除空标题窗口
        WinGetClass, this_class, ahk_id %this_id%
        If (this_class == "Progman")
            Continue ; 排除 Win10 的常驻窗口管理器
        
        listOfWindow .= this_id . "|" . this_title . "`n"
        
        ; shorten edge first
        if (A_ScreenWidth <= A_ScreenHeight){
            ; row first
            nx := Mod(k, col)
            ny := k / col | 0
        }else{
            ; col first
            nx := k / row | 0
            ny := Mod(k, row)
        }
        x := nx * size_x
        y := ny * size_y
        
        WinGet, minmax, minmax, ahk_id %this_id%
        if (minmax == -1)
            continue
        if (minmax == 1){
            if (! resotreMaxWindows){
                continue
            }else{
                WinRestore, ahk_id %this_id%
            }
        }
        
        ; WinMove, ahk_id %this_id%, , %x%, %y%, %size_x%, %size_y%
        rx:=x-8, ry:=y-8, rsize_x:=size_x+16+8, rsize_y:=size_y+16 ; 填满边界不留缝隙
        WinMove, ahk_id %this_id%, , %rx%, %ry%, %rsize_x%, %rsize_y%
        k+=1
        ; WinActivate, ahk_id %this_id%
        ; MsgBox, 4, , %x% %y% %size_x% %size_y%
        ; IfMsgBox, NO, break
    }
    ; 调试用
    ; FileAppend, %listOfWindow%, listOfWindow.txt
}
; ToolTip, A_Args[1]
ArrangeWindows(A_Args[1])