; ========== CapsLockX ==========
; Note：Save as UTF-8 with BOM please
; 名称：窗口增强模块
; 作者：snomiao (snomiao@gmail.com)
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.07.04
; 版权：Copyright © 2018-2020 Snowstar Laboratory. All Rights Reserved.
; 许可证 LICENCE: GNU GPLv3 ( https://www.gnu.org/licenses/gpl-3.0.html )
; ========== CapsLockX ==========
;
; Exit if running without CapsLockX
; 
If(!CapsLockX)
    ExitApp

AppendHelp("
(
窗口墙强
| CapsLockX + O               | 快速排列当前桌面的窗口
| CapsLockX + Shift + O       | 快速排列当前桌面的窗口（不包括最大化的窗口）
| CapsLockX + [ ]             | 切换到上一个/下一个桌面
| CapsLockX + =               | 新建桌面
| CapsLockX + -               | 删除当前桌面（会把所有窗口移到上一个桌面）
| CapsLockX + Alt + [ ]       | 把当前窗口移到上一个/下一个桌面
| CapsLockX + Alt + =         | 新建桌面，并把当前窗口移过去
| CapsLockX + 1 2 ... 9       | 切换到第 n 个桌面
| CapsLockX + Alt + 1 2 ... 9 | 把当前窗口移到第 n 个桌面(如果有的话)
)")
; setup done
Return


; 把当前窗口置顶

; #If !!(CapsLockXMode & CM_FN)

; 确保WinTab模块优先级比Mouse高，否则此处 WASD 无效
; Make sure WinTab module has higher priority than Mouse, otherwise WASD is invalid here

#If CapsLockXMode == CM_CAPSX || CapsLockXMode == CM_FN

; ; 自动排列窗口 
; o:: Run Tools\ArrangeWindows.func.ahk 1
; ; 自动排列窗口（不包括最大化的窗口）
; +o:: Run Tools\ArrangeWindows.func.ahk 0
; ; 自动堆叠窗口 
; !o:: Run Tools\ArrangeWindows.func.ahk 3
; ; 自动堆叠窗口（不包括最大化的窗口）
; !+o:: Run Tools\ArrangeWindows.func.ahk 2


; 自动排列窗口 
o:: ArrangeWindows(1)
; 自动排列窗口（不包括最大化的窗口）
+o:: ArrangeWindows(0)
; 自动堆叠窗口 
!o:: ArrangeWindows(3)
; 自动堆叠窗口（不包括最大化的窗口）
!+o:: ArrangeWindows(2)

; 使用 Windows 原生的方式自动排列窗口（和虚拟桌面不兼容）
; !o::
;     WinActivate ahk_class Shell_TrayWnd ahk_exe explorer.exe
;     ; side by side 排列
;     Send {AppsKey}i
Return

; Win + Tab
\:: Send #{Tab} 

; 切换当前窗口置顶并透明
'::
    ; WinGet, Var, Transparent, 150, A
    WinSet, Transparent, 200, A
    Winset, Alwaysontop, , A
Return

+'::
    ; WinGet, Var, Transparent, 150, A
    WinSet, Transparent, 255, A
    Winset, Alwaysontop, , A
Return

; 让当前窗口临时透明
`;::
WinSet, Transparent, 100, A
Return

`; Up::
WinSet, Transparent, 255, A
Return

; 关闭标签
$x:: Send ^w
; 关闭窗口
$Esc:: Send !{F4}
$!x:: Send !{F4}
; 杀死窗口
$^!x:: WinKill A

#If WinActive("ahk_class MultitaskingViewFrame")
    
; 在 Alt+Tab 下, WASD 模拟方向键 , 1803之后还可以用
!a:: Left
!d:: Right
!w:: Up
!s:: Down
; qe 切换桌面
!q::
    SendEvent {Blind}{Enter}
    Sleep 200
    SendEvent ^#{Left}
Return
!e::
    SendEvent {Blind}{Enter}
    Sleep 200
    SendEvent ^#{Right}
Return
!+q::
    SendEvent {Blind}{Enter}
    Sleep 200
    MoveActiveWindowWithAction("^#{Left}")
Return
!+e::
    SendEvent {Blind}{Enter}
    Sleep 200
    MoveActiveWindowWithAction("^#{Right}")
Return
; cx 关闭应用
!c:: SendEvent {Blind}{Delete}{Right}
!x:: SendEvent {Blind}{Delete}{Right}

; 新建桌面
!z::
    SendEvent {Blind}{Esc}
    Sleep 200
    Send ^#d
Return
; 新建桌面并移动窗口
!v::
    SendEvent {Blind}{Esc}
    Sleep 200
    MoveActiveWindowWithAction("^#d")
Return

; 模拟 Tab 键切换焦点
\:: Send {Tab}
; 在 Win10 下的 Win+Tab 界面，WASD 切换窗口焦点
; 以及在窗口贴边后，WASD 切换窗口焦点

; 模拟方向键
w:: Send {Up}
a:: Send {Left}
s:: Send {Down}
d:: Send {Right}

; 切换桌面概览
q:: Send ^#{Left}
e:: Send ^#{Right}
[:: Send ^#{Left}
]:: Send ^#{Right}

; 增删桌面
=:: Send ^#d
-:: Send ^#{F4}
z:: Send ^#{F4}

; 关掉窗口
x:: Send ^w{Right}
`;:: Send ^w{Right}

; 切换到第x个桌面
1::Send {AppsKey}m{Down 0}{Enter}
2::Send {AppsKey}m{Down 1}{Enter}
3::Send {AppsKey}m{Down 2}{Enter}
4::Send {AppsKey}m{Down 3}{Enter}
5::Send {AppsKey}m{Down 4}{Enter}
6::Send {AppsKey}m{Down 5}{Enter}
7::Send {AppsKey}m{Down 6}{Enter}
8::Send {AppsKey}m{Down 7}{Enter}
9::Send {AppsKey}m{Down 8}{Enter}

; ; 移到除了自己的最后一个桌面（或新建桌面）
; 0::Send {AppsKey}m{Up 2}{Enter}

; 移到新建桌面
v:: Send {AppsKey}mn{Sleep 16}+{Tab}
':: Send {AppsKey}mn{Sleep 16}+{Tab}

; 移到新建桌面，并激活窗口
c:: Send {AppsKey}mn{Enter}

; 新版

; ahk_class Windows.UI.Core.CoreWindowi
; ahk_exe explorer.exe

; #IfWinActive (?:Task View)|任务视图 ahk_class Windows.UI.Core.CoreWindow ; ahk_exe explorer.exe
#IfWinActive ahk_class Windows.UI.Core.CoreWindow ahk_exe explorer.exe
    
    ; 在 Alt+Tab 下, WASD 模拟方向键
    !a:: Left
    !d:: Right
    !w:: Up
    !s:: Down
    ; ; qe 切换桌面
    ; !q:: Send ^#{Left}
    ; !e:: Send ^#{Right}
    ; ; qe 切换桌面
    ; !c:: Delete
    
    ; 模拟 Tab 键切换焦点
    \:: Send {Tab}
    ; 在 Win10 下的 Win+Tab 界面，WASD 切换窗口焦点
    ; 以及在窗口贴边后，WASD 切换窗口焦点
    
    ; 模拟方向键
    w:: Send {Up}
    a:: Send {Left}
    s:: Send {Down}
    d:: Send {Right}
    
    ; 切换桌面概览
    q:: Send {Enter}; ^#{Left}
    e:: Send {Enter}; ^#{Right}
    [:: Send ^#{Left}
    ]:: Send ^#{Right}
    
    ; 增删桌面
    =:: Send ^#d
    -:: Send ^#{F4}
    z:: Send ^#{F4}
    
    ; 关掉窗口
    x:: Send ^w
    `;:: Send ^w
    
    ; 切换到第x个桌面
    1::Send {AppsKey}m{Down 0}{Enter}
    2::Send {AppsKey}m{Down 1}{Enter}
    3::Send {AppsKey}m{Down 2}{Enter}
    4::Send {AppsKey}m{Down 3}{Enter}
    5::Send {AppsKey}m{Down 4}{Enter}
    6::Send {AppsKey}m{Down 5}{Enter}
    7::Send {AppsKey}m{Down 6}{Enter}
    8::Send {AppsKey}m{Down 7}{Enter}
    9::Send {AppsKey}m{Down 8}{Enter}
    
    ; ; 移到除了自己的最后一个桌面（或新建桌面）
    ; 0::Send {AppsKey}m{Up 2}{Enter}
    
    ; 移到新建桌面
    v:: Send {AppsKey}mn{Sleep 16}+{Tab}
    ':: Send {AppsKey}mn{Sleep 16}+{Tab}
    
    ; 移到新建桌面，并激活窗口
    c:: Send {AppsKey}mn{Enter}


#If
; 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口
; 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口
; 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口
; 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口
; 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口
; 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口
; 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口
; 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口 快速排列窗口
; ArrangeWindows(arrangeFlags = "0"){
;     SysGet, MonitorCount, MonitorCount
;     Loop, %MonitorCount%
;     {
;         ArrangeWindowsInMonitor(arrangeFlags, A_Index)
;     }
; }
; this is improved method for stable
GetMonitorIndexFromWindowByWindowsCenterPoint(hWnd){
    WinGetPos, X, Y, W, H, ahk_id %hWnd%
    CX := X + W / 2
    CY := Y + H / 2 
    SysGet, monitorCount, MonitorCount
    monitorIndex := "" ; default
    Loop, %monitorCount%
    {
        SysGet, M, Monitor, %A_Index%
        ; Compare center position to determine the monitor index.
        if(( abs(min(max(MLeft, CX), MRight) - CX) <= 1) 
        && ( abs(min(max(MTop, CY), MBottom) - CY) <= 1)){
            msgbox,, %title%, %A_Index% %MLeft% %CX% %MRight% %EQ%
            monitorIndex := A_Index
            break
        }
    }
    return %monitorIndex%
}
; below function is modified from [How to determine a window is in which monitor? - Ask for Help - AutoHotkey Community]( https://autohotkey.com/board/topic/69464-how-to-determine-a-window-is-in-which-monitor/ )
GetMonitorIndexFromWindow(hWnd)
{
	; default is blank to prevent 
	monitorIndex := ""
    ; monitorIndex := GetMonitorIndexFromWindowByWindowsCenterPoint(hWnd)
    ; if(monitorIndex){
    ;     return %monitorIndex%
    ; }
	VarSetCapacity(monitorInfo, 40)
	NumPut(40, monitorInfo)
	
	if (monitorHandle := DllCall("MonitorFromWindow", "uint", hWnd, "uint", 0x2)) 
		&& DllCall("GetMonitorInfo", "uint", monitorHandle, "uint", &monitorInfo) 
	{
		monitorLeft   := NumGet(monitorInfo,  4, "Int")
		monitorTop    := NumGet(monitorInfo,  8, "Int")
		monitorRight  := NumGet(monitorInfo, 12, "Int")
		monitorBottom := NumGet(monitorInfo, 16, "Int")
		workLeft      := NumGet(monitorInfo, 20, "Int")
		workTop       := NumGet(monitorInfo, 24, "Int")
		workRight     := NumGet(monitorInfo, 28, "Int")
		workBottom    := NumGet(monitorInfo, 32, "Int")
		isPrimary     := NumGet(monitorInfo, 36, "Int") & 1

        ; msgbox , ,, workLeft%workLeft% workTop%workTop% workRight%workRight% workBottom%workBottom%
        

		SysGet, monitorCount, MonitorCount

		Loop, %monitorCount%
		{
			SysGet, tempMon, Monitor, %A_Index%

			; Compare location to determine the monitor index.
			if ((monitorLeft = tempMonLeft) and (monitorTop = tempMonTop)
				and (monitorRight = tempMonRight) and (monitorBottom = tempMonBottom))
			{
				monitorIndex := A_Index
				break
			}
		}
	}
	
	return %monitorIndex%
}
ArrangeWindows(arrangeFlags = "0"){
    arrangeFlags += 0 ; string to number
    ARRANGE_MAXWINDOW := 1
    ARRANGE_STACKED := 2 ; if not then arrange SIDE_BY_SIDE

    ; 常量定义
    WS_EX_TOOLWINDOW  := 0x00000080
    WS_EX_APPWINDOW   := 0x00040000
    WS_CAPTION        := 0x00C00000
    WS_EX_NOANIMATION := 0x04000000
    WS_EX_NOACTIVATE  := 0x08000000
    WS_POPUP          := 0x80000000

    SysGet, MonitorCount, MonitorCount
    Loop, %MonitorCount%
    {
        MonitorIndex := A_Index
        listOfWindow%MonitorIndex% := ""
    }
    
    DetectHiddenWindows, Off
    WinGet, id, List,,,
    Loop, %id%
    {
        hWnd := id%A_Index%

        WinGet, this_pid, PID, ahk_id %hWnd%
        WinGet, style, style, ahk_id %hWnd%
        WinGetTitle, this_title, ahk_id %hWnd%
        WinGetClass, this_class, ahk_id %hWnd%
        
        ; Process, , PID-or-Name [, Param3]
        ; 白名单
        ; If (this_class == "TXGuiFoundation"){  ;  && this_process=="QQ.exe"
        ;     style := style & ~WS_POPUP
        ; }
        If(1){
            ; 跳过最小化的窗口
            WinGet, minmax, minmax, ahk_id %hWnd%
            if(minmax == -1){
                continue
            }
            ; 跳过最大化窗口
            if(!(arrangeFlags & ARRANGE_MAXWINDOW)){
                if (minmax == 1){
                    continue
                }
            }
            ; 黑名单
            ; ; 跳过无标题窗口
            ; if !(style & WS_CAPTION)
            ;     Continue
            ; ; 跳过工具窗口
            ; if (style & WS_EX_TOOLWINDOW)
            ;     Continue
            if (!(style & WS_EX_APPWINDOW))
                Continue
            ; ; 跳过弹出窗口
            ; if (style & WS_POPUP)
            ;     Continue 
            ; 排除空标题窗口
            If (!RegExMatch(this_title, ".+"))
                Continue 
            ; If (this_class == "Progman")
            ; Continue ; 排除 Win10 的常驻窗口管理器
            ; 排除不归属于当前参数显示器的窗口
            ; if(!!MonitorIndex){
                ; this_monitor := GetMonitorIndexFromWindow(hWnd)
            ;     if(MonitorIndex != this_monitor){
            ;         continue
            ;     }
            ; }
            ; 跳过不在当前虚拟桌面的窗口
            if(!IsWindowOnCurrentVirtualDesktop(hWnd)){
                continue
            }
            ; MsgBox ,,,% style

            ; debug
            ; WinActivate, ahk_id %hWnd%
            ; WinGetClass, this_class, ahk_id %hWnd%
            ; WinGetTitle, this_title, ahk_id %hWnd%
            ; WinGetPos, X, Y, Width, Height, ahk_id %hWnd%
            ; MsgBox, 4, , Visiting All Windows`n%A_Index% of %id%`nahk_id %hWnd%`n%X% %Y% %Width% %Height%`nahk_class %this_class%`n%this_title%`n`nContinue?
            ; IfMsgBox, NO, break
        }
        this_monitor := GetMonitorIndexFromWindow(hWnd)
        listOfWindow%this_monitor% .= "ahk_pid " this_pid " ahk_id " hWnd "`n" ; . "`t" . this_title . "`n"
        
    }
    Loop, %MonitorCount%
    {
        ; 先按 pid 和 hwnd 排列，这样排出来的窗口的顺序就是稳定的了
        Sort listOfWindow%A_Index%
        ; MsgBox,, , % listOfWindow%A_Index%
        if(arrangeFlags & ARRANGE_STACKED){
            ArrangeWindowsStacked(listOfWindow%A_Index%, arrangeFlags, A_Index)
        }else{
            ArrangeWindowsSideBySide(listOfWindow%A_Index%, arrangeFlags, A_Index)
        }
    }
}
ArrangeWindowsSideBySide(listOfWindow, arrangeFlags = "0", MonitorIndex = ""){
    arrangeFlags += 0 ; string to number
    n := StrSplit(listOfWindow, "`n", "`r").Count() - 1
    ; try parse work rect from monitor
    if(!MonitorIndex){
        AreaX := 0 
        AreaY := 0 
        AreaW := A_ScreenWidth
        AreaH := A_ScreenHeight
    }else{
        SysGet, MonitorWorkArea, MonitorWorkArea, %MonitorIndex%
        ; SysGet, Monitor, Monitor, %MonitorIndex%
        AreaX := MonitorWorkAreaLeft
        AreaY := MonitorWorkAreaTop
        AreaW := MonitorWorkAreaRight - MonitorWorkAreaLeft
        AreaH := MonitorWorkAreaBottom - MonitorWorkAreaTop
    }
    ; calc rows and cols
    ; shorten edge first
    if (AreaW <= AreaH){
        ; row more than cols
        col := Sqrt(n) | 0
        row := Ceil(n / col)
    }else{
        ; col more than rows
        row := Sqrt(n) | 0
        col := Ceil(n / row)
    }
    size_x := AreaW / col
    size_y := AreaH / row
    
    k:=0
    Loop, Parse, listOfWindow, `n
    {
        hWnd := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")

        ; shorten edge first
        if (AreaW <= AreaH){
            ; row first
            nx := Mod(k, col)
            ny := k / col | 0
        }else{
            ; col first
            nx := k / row | 0
            ny := Mod(k, row)
        }
        x := AreaX + nx * size_x
        y := AreaY + ny * size_y
        
        ; 填满窗口间的缝隙
        x:=x-8, y:=y, w:=size_x+16, h:=size_y+8 

        if(n <= 4 ){
            ; msgbox ,,, x%x% y%y% w%w% h%h% `n X%AreaX% Y%AreaY% W%AreaW% H%AreaH%
        }
        ; 
        FastResizeWindow(hWnd, x, y, w, h)
        k+=1
    }
}

ArrangeWindowsStacked(listOfWindow, arrangeFlags = "0", MonitorIndex = ""){
    arrangeFlags += 0 ; string to number
    n := StrSplit(listOfWindow, "`n", "`r").Count() - 1
    ; try parse work rect from monitor
    if(!MonitorIndex){
        AreaX := 0 
        AreaY := 0 
        AreaW := A_ScreenWidth
        AreaH := A_ScreenHeight
    }else{
        SysGet, MonitorWorkArea, MonitorWorkArea, %MonitorIndex%
        AreaX := MonitorWorkAreaLeft
        AreaY := MonitorWorkAreaTop
        AreaW := MonitorWorkAreaRight - MonitorWorkAreaLeft
        AreaH := MonitorWorkAreaBottom - MonitorWorkAreaTop
    }
    ; 
    k := 1
    dx := 64
    dy := 64
    w := AreaW - 2 * dx - n * dx + dx
    h := AreaH - 2 * dy - n * dy + dy
    Loop, Parse, listOfWindow, `n
    {
        hWnd := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")

        x := AreaX + k * dx
        y := AreaY + k * dy
        FastResizeWindow(hWnd, x, y, w, h, 1)
        k+=1
    }
}

FastResizeWindow(hWnd, x, y, w, h, ForceTOP = ""){
    ; 如有必要则还原最大化的窗口
    WinGet, minmax, minmax, ahk_id %hWnd%
    if (minmax == 1){
        WinRestore, ahk_id %hWnd%
        needSetTOPMOST := 1
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
    flags := SWP_NOACTIVATE
    ; 先置顶（否则会显示在最大化窗口的后面 -- 被挡住）
    if(ForceTOP){
        ; WinActivate, ahk_id %hWnd%
        DllCall("SetWindowPos"
            , "UInt", hWnd ;handle
            , "UInt", HWND_TOPMOST ; z-index
            , "Int", 0 ; x
            , "Int", 0 ; y
            , "Int", 0 ; width
            , "Int", 0 ; height
            , "UInt", SWP_NOSIZE | SWP_NOMOVE | SWP_ASYNCWINDOWPOS ) ; SWP_ASYNCWINDOWPOS
    }
    ; 再排到正确的位置上
    DllCall("SetWindowPos"
        , "UInt", hWnd ;handle
        , "UInt", HWND_NOTOPMOST ; z-index
        , "Int", x
        , "Int", y
        , "Int", w
        , "Int", h
        , "UInt", SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS) ; SWP_ASYNCWINDOWPOS
}