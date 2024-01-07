; ========== CapsLockX ==========
; 注：Save as UTF-8 with BOM please
; 名称：摸拟鼠标
; 描述：WASD鼠标移动，QE 左右键 RF滚轮
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.06.27
; 版权：Copyright © 2017-2022 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========

; will include once
#Include, Modules/AccModel/AccModel.ahk

if (!CapsLockX) {
    MsgBox, % "本模块只为 CapsLockX 工作"
    ExitApp
}

global TMouse_Disabled := CapsLockX_Config("TMouse", "Disabled", 0, "禁用模拟鼠标模块")
global TMouse_SendInput := CapsLockX_Config("TMouse", "SendInput", 1, "使用 SendInput 方法提高模拟鼠标点击、移动性能")
global TMouse_SendInputAPI := CapsLockX_Config("TMouse", "SendInputAPI", 1, "使用 Windows API 强势提升模拟鼠标移动性能")
global TMouse_SendInputScroll := CapsLockX_Config("TMouse", "TMouse_SendInputScroll", 0, "使用 Windows API 强势提升模拟鼠标滚轮性能（目前有bug不建议启用）")

global TMouse_StickyCursor := CapsLockX_Config("TMouse", "StickyCursor", 1, "启用自动粘附各种按钮，编辑框")
global TMouse_StopAtScreenEdge := CapsLockX_Config("TMouse", "StopAtScreenEdge", 1, "撞上屏幕边界后停止加速")

; 根据屏幕 DPI 比率，自动计算，得出，如果数值不对，才需要纠正
global TMouse_UseDPIRatio := CapsLockX_Config("TMouse", "UseDPIRatio", 1, "是否根据屏幕 DPI 比率缩放鼠标速度")
global TMouse_MouseSpeedRatio := CapsLockX_Config("TMouse", "MouseSpeedRatio", 1, "鼠标加速度比率, 默认为 1, 你想慢点就改成 0.5 之类")
global TMouse_WheelSpeedRatio := CapsLockX_Config("TMouse", "WheelSpeedRatio", 1, "滚轮加速度比率, 默认为 1, 你想慢点就改成 0.5 之类")
global TMouse_DPIRatio := TMouse_UseDPIRatio ? A_ScreenDPI / 96 : 1
global CapsLockX_HJKL_Scroll := CapsLockX_Config("TMouse", "CapsLockX_HJKL_Scroll", 0, "使用IJKL滚轮移动滚轮，比RF多一个横轴。")

CapsLockX_AppendHelp( CapsLockX_LoadHelpFrom("Modules/01.1-插件-鼠标模拟.md" ))
; global debug_fps := new FPS_Debugger()
global 鼠标模拟 := new AccModel2D(Func("鼠标模拟"), 0.1, TMouse_DPIRatio * 120 * 2 * TMouse_MouseSpeedRatio)
global ScrollSimulator := new AccModel2D(Func("ScrollSimulator"), 0.1, TMouse_DPIRatio * 120 * 4 * TMouse_WheelSpeedRatio)
global DragSimulator := new AccModel2D(Func("DragSimulator"), 0.1, TMouse_DPIRatidweo * 120 * 16 * TMouse_WheelSpeedRatio)
global ZoomSimu := new AccModel2D(Func("ZoomSimu"), 0.1, TMouse_DPIRatio * 120 * 4 * TMouse_WheelSpeedRatio)
global 滚轮自动控制 := new AccModel2D(Func("滚轮自动控制"), 0.1, 10)
global 滚轮自动 := new AccModel2D(Func("滚轮自动"), 0, 1)

if (TMouse_SendInput) {
    SendMode Input
}
; 解决多屏 DPI 问题
DllCall("Shcore.dll\SetProcessDpiAwareness", "UInt", 2)

; gestures supports
; [[AHK_H+Hv2] - WinApi() - Page 2 - Scripts and Functions - AutoHotkey Community]( https://www.autohotkey.com/board/topic/51243-ahk-hhv2-winapi/page-2 )
global WM_GESTURE:=281
global WM_GESTURENOTIFY:=282

global GID_BEGIN:=1
global GID_END:=2
global GID_ZOOM:=3
global GID_PAN:=4
global GID_ROTATE:=5
global GID_TWOFINGERTAP:=6
global GID_PRESSANDTAP:=7

Return

#if CapsLockXMode && !CapsLockX_MouseButtonSwitched

; 鼠标按键处理
*e:: CapsLockX_LMouseButtonDown("e")
*q:: CapsLockX_RMouseButtonDown("q")
*e Up::CapsLockX_LMouseButtonUp()k
*q Up:: CapsLockX_RMouseButtonUp()

#if CapsLockXMode && CapsLockX_MouseButtonSwitched

; 鼠标按键处理
*e:: CapsLockX_RMouseButtonDown("e")
*q:: CapsLockX_LMouseButtonDown("q")
*e Up::CapsLockX_RMouseButtonUp()
*q Up:: CapsLockX_LMouseButtonUp()

#if CapsLockXMode

; 鼠标运动处理
*a:: 鼠标模拟.左按("a")
*d:: 鼠标模拟.右按("d")
*w:: 鼠标模拟.上按("w")
*s:: 鼠标模拟.下按("s")

#if CapsLockXMode

; hold right shift key to simulate horizonal scrolling
*>+r:: ScrollSimulator.左按("r")
*>+f:: ScrollSimulator.右按("f")
*r:: ScrollSimulator.上按("r")
*f:: ScrollSimulator.下按("f")

#if

CursorHandleGet()
{
    VarSetCapacity(PCURSORINFO, 20, 0) ;为鼠标信息 结构 设置出20字节空间
    NumPut(20, PCURSORINFO, 0, "UInt") ;*声明出 结构 的大小cbSize = 20字节
    DllCall("GetCursorInfo", "Ptr", &PCURSORINFO) ;获取 结构-光标信息
    if (NumGet(PCURSORINFO, 4, "UInt") == 0 ) ;当光标隐藏时，直接输出特征码为0
    Return 0
    Return NumGet(PCURSORINFO, 8)
}

CursorShapeChangedQ()
{
    static lA_Cursor := CursorHandleGet()
    if (lA_Cursor == CursorHandleGet()) {
        Return 0
    }
    lA_Cursor := CursorHandleGet()
    Return 1
}

sign(v)
{
    Return v == 0 ? 0 : (v > 0 ? 1 : -1)
}

Pos2Long(x, y)
{
    Return x | (y << 16)
}

;global dpiX := 0, dpiY := 0
;DllCall("User32.dll\MonitorFromPoint", "UInt", x, "UInt", y, "UInt", 0/*MONITOR_DEFAULTTONEAREST*/)
; VarSetCapacity(point, 8, 0)
; MouseGetPos, x, y
; hMonitor := DllCall("User32.dll\MonitorFromPoint", "UInt", x, "UInt", y, "UInt", 0)
; ;DllCall("Shcore.dll\GetDpiForMonitor", "Ptr", hMonitor, "UInt", 2, "UInt*", dpiX, "UInt*", dpiY)
; DllCall("Shcore.dll\GetDpiForMonitor", "Ptr", hMonitor, "UInt", 0, "UInt*", dpiX, "UInt*", dpiY)
; ;A_ScreenDPI
; DllCall("GetDeviceCaps", "UInt", DllCall("GetDC", "UInt", 0), "UInt", 0)

; ref: https://msdn.microsoft.com/en-us/library/windows/desktop/ms646273(v=vs.85).aspx
SendInput_MouseMsg32(dwFlag, mouseData := 0)
{
    VarSetCapacity(sendData, 28, 0)
    NumPut(0, sendData, 0, "UInt")
    NumPut(0, sendData, 4, "Int")
    NumPut(0, sendData, 8, "Int")
    NumPut(mouseData, sendData, 12, "UInt")
    NumPut(dwFlag, sendData, 16, "UInt")
    DllCall("SendInput", "UInt", 1, "Str", sendData, "UInt", 28)
}
PostMessage_ScrollMouse(dx, dy)
{
    WM_MOUSEWHEEL := 0x020A
    WM_MOUSEWHEELH := 0x020E
    _:= dy && PostMessageForScroll(WM_MOUSEWHEEL, -dy)
    _:= dx && PostMessageForScroll(WM_MOUSEWHEELH, dx)
}
ScrollMouse(dx, dy)
{
    global TMouse_SendInputScroll
    if (TMouse_SendInputScroll) {
        SendInput_ScrollMouse(dx, dy)
    } else {
        PostMessage_ScrollMouse(dx, dy)
    }
}
SendInput_ScrollMouse(dx, dy)
{
    ; get cursor pos
    VarSetCapacity(POINT, 8, 0)
    DllCall("GetCursorPos", "Ptr", &POINT)
    x := NumGet(POINT, 0, "Int")
    y := NumGet(POINT, 4, "Int")
    ; scroll by system input message
    MOUSEEVENTF_WHEEL := 0x0800
    MOUSEEVENTF_HWHEEL := 0x1000
    if (dy) {
        size := A_PtrSize+4*4+A_PtrSize*2
        VarSetCapacity(mi, size, 0)
        NumPut(x, mi, A_PtrSize, "Int") ; LONG dx
        NumPut(y, mi, A_PtrSize+4, "Int") ; LONG dy
        NumPut(-dy, mi, A_PtrSize+4+4, "Int") ; DWORD mouseData
        NumPut(MOUSEEVENTF_WHEEL, mi, A_PtrSize+4+4+4, "UInt") ; DWORD dwFlags
        DllCall("SendInput", "UInt", 1, "Ptr", &mi, "Int", size )
        ; perf_timing()
    }
    if (dx) {
        ; todo fix sendinput
        PostMessage_ScrollMouse(dx, 0)
        ; size := A_PtrSize+4*4+A_PtrSize
        ; VarSetCapacity(mi, size, 0)
        ; NumPut(x, mi, A_PtrSize, "Int")         ; LONG dx
        ; NumPut(y, mi, A_PtrSize+4, "Int")       ; LONG dy
        ; NumPut(dx * 120, mi, A_PtrSize+4+4, "Int")    ; DWORD mouseData
        ; NumPut(MOUSEEVENTF_HWHEEL, mi, A_PtrSize+4+4+4, "UInt")   ; DWORD dwFlags
        ; DllCall("SendInput", "UInt", 1, "Ptr", &mi, "Int", size )
    }
}

SendInput_MouseMove(x, y)
{
    ; (20211105)终于支持64位了
    ; [SendInput function (winuser.h) - Win32 apps | Microsoft Docs]( https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput )
    ; [INPUT (winuser.h) - Win32 apps | Microsoft Docs]( https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-input )
    ; [MOUSEINPUT (winuser.h) - Win32 apps | Microsoft Docs]( https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-mouseinput )
    size := A_PtrSize+4*4+A_PtrSize*2
    VarSetCapacity(mi, size, 0)
    NumPut(x, mi, A_PtrSize, "Int") ; int dx
    NumPut(y, mi, A_PtrSize+4, "Int") ; int dy
    NumPut(0x0001, mi, A_PtrSize+4+4+4, "UInt") ; DWORD dwFlags MOUSEEVENTF_MOVE
    DllCall("SendInput", "UInt", 1, "Ptr", &mi, "Int", size )
}
鼠标模拟2(dx, dy){
    SendInput_MouseMove(dx, dy)

}

; void 鼠标模拟
鼠标模拟(dx, dy, 状态){
    if (!CapsLockXMode) {
        鼠标模拟.止动()
        return
    }
    if (状态 == "横中键") {
        ; TODO: better ScrollModeToggle()
        鼠标模拟.止动()
        return
    }
    if (状态 == "纵中键") {
        ; TODO: better ScrollModeToggle()
        鼠标模拟.止动()
        return
    }
    if (状态 != "移动") {
        return
    }
    ; Shift 减速 =1
    if (GetKeyState("Shift", "P")) {
        sleep 100
        dx := dx == 0 ? 0 : (dx > 0 ? 1 : -1 )
        dy := dy == 0 ? 0 : (dy > 0 ? 1 : -1 )
    }
    if (TMouse_SendInputAPI) {
        ; 支持64位AHK！
        SendInput_MouseMove(dx, dy)
    } else {
        MouseMove, %dx%, %dy%, 0, R
    }

    ; TODO: 撞到屏幕边角就停下来
    ; if(TMouse_StopAtScreenEdge )
    ; MouseGetPos, xb, yb
    ; 鼠标模拟.横速 *= dx && xa == xb ? 0 : 1
    ; 鼠标模拟.纵速 *= dy && ya == yb ? 0 : 1

    ; 在各种按钮上减速，进出按钮时减速80%
    if (TMouse_StickyCursor && CursorShapeChangedQ()) {
        鼠标模拟.横速 *= 0.2
        鼠标模拟.纵速 *= 0.2
    }
}

滚轮自动(dx, dy, 状态){
    if (状态 != "移动") {
        return
    }
    ScrollMouse(dx, dy)
}
滚轮自动控制(dx, dy, 状态){
    if (状态 != "移动") {
        return
    }
    滚轮自动.横速 += dx, 滚轮自动.纵速 += dy, 滚轮自动.始动()
    msg := "【雪星滚轮自动v2】`n"
    msg .= "横：" (滚轮自动.横速|0) "px/s`n纵：" (滚轮自动.纵速|0) "px/s`n"
    msg .= "CapsLockX + Ctrl + Alt + RF 调整纵向自动滚轮`n"
    msg .= "CapsLockX + Ctrl + Alt + Shift + RF 调整横向自动滚轮`n"
    鼠标模拟_ToolTip(msg)
}
ScrollSimulator(dx, dy, 状态){
    if (!CapsLockXMode) {
        return ScrollSimulator.止动()
    }
    if ( 状态 == "横中键" || 状态 == "纵中键") {

        SendEvent {Blind}{MButton Down}
        KeyWait r
        KeyWait f
        SendEvent {Blind}{MButton Up}
        ; 关闭滚轮自动
        if (滚轮自动.横速 || 滚轮自动.纵速) {
            滚轮自动.止动()
            滚轮自动控制(0, 0, "止动")
        }
        return
    }
    if (状态 != "移动") {
        return
    }
    ScrollMouse(dx, dy)
}
DragSimulator(dx, dy, 状态){
    if (!CapsLockXMode) {
        return DragSimulator.止动()
    }
    if ( 状态 == "横中键" || 状态 == "纵中键") {
        SendEvent {Blind}{MButton Down}
        KeyWait r
        KeyWait f
        SendEvent {Blind}{MButton Up}
        ; 关闭滚轮自动
        if (滚轮自动.横速 || 滚轮自动.纵速) {
            滚轮自动.止动()
            滚轮自动控制(0, 0, "止动")
        }
        return
    }
    if (状态 != "移动") {
        return
    }
    ScrollMouse(dx, dy)
}
ZoomSimu(dx, dy, action)
{
    if (!CapsLockXMode) {
        return ZoomSimu.止动()
    }
    if (action != "移动") {
        return
    }
    SendEvent, {CtrlDown}
    ScrollMouse(dx, dy)
    SendEvent, {CtrlUp}
}
PostMessageForScroll(msg, zDelta)
{
    ; 目前还不支持 UWP which should use WM_TOUCH
    ; Currently, UWP does not support this; you should use WM_TOUCH instead.
    CoordMode, Mouse, Screen
    MouseGetPos, x, y, wid, fcontrol
    wParam := zDelta << 16 ;zDelta
    lParam := x | (y << 16) ; pos2long
    MouseGetPos, , , , ControlClass2, 2
    MouseGetPos, , , , , ControlClass3, 3
    if (A_Is64bitOS) {
        ControlClass1 := DllCall("WindowFromPoint", "int64", x | (y << 32), "Ptr") | 0x0
    } else {
        ControlClass1 := DllCall("WindowFromPoint", "int", x, "int", y) | 0x0
    }
    ;Detect modifer keys held down (only Shift and Control work)
    wParam |= GetKeyState("Shift", "p") ? 0x4 : 0
    wParam |= GetKeyState("Ctrl", "p") ? 0x8 : 0
    if (ControlClass2 == "") {
        ; PostMessage, %msg%, %wParam%, %lParam%, %fcontrol%, ahk_id %ControlClass1%
        DllCall("PostMessage", "UInt", ControlClass1, "UInt", msg, "UInt", wParam, "UInt", lParam, "UInt")
    } else {
        ; PostMessage, %msg%, %wParam%, %lParam%, %fcontrol%, ahk_id %ControlClass2%
        DllCall("PostMessage", "UInt", ControlClass2, "UInt", msg, "UInt", wParam, "UInt", lParam, "UInt")
        if (ControlClass2 != ControlClass3) {
            ; PostMessage, %msg%, %wParam%, %lParam%, %fcontrol%, ahk_id %ControlClass3%
            DllCall("PostMessage", "UInt", ControlClass3, "UInt", msg, "UInt", wParam, "UInt", lParam, "UInt")
        }
    }
    if (wid) {
        DllCall("PostMessage", "UInt", wid, "UInt", msg, "UInt", wParam, "UInt", lParam, "UInt")
    }
    ; tooltip % x " " y "`n" ControlClass1  "`n"  ControlClass2 "`n" ControlClass3 "`n" wid
}

CapsLockX_LMouseButtonDown(wait){
    global CapsLockX_鼠标左键等待
    if (CapsLockX_鼠标左键等待) {
        return
    }
    CapsLockX_鼠标左键等待 := wait
    SendEvent {Blind}{LButton Down}
    KeyWait %wait%
    ; Hotkey, %wait% Up, CapsLockX_LMouseButtonUp
}
CapsLockX_LMouseButtonUp(){
    global CapsLockX_鼠标左键等待
    SendEvent {Blind}{LButton Up}
    CapsLockX_鼠标左键等待 := ""

}
CapsLockX_RMouseButtonDown(wait){
    global CapsLockX_RMouseButtonWait
    if (CapsLockX_RMouseButtonWait) {
        return
    }
    CapsLockX_RMouseButtonWait := wait
    SendEvent {Blind}{RButton Down}
    KeyWait %wait%
    ; Hotkey, %wait% Up, CapsLockX_RMouseButtonUp
}
CapsLockX_RMouseButtonUp(){
    global CapsLockX_RMouseButtonWait
    SendEvent {Blind}{RButton Up}
    CapsLockX_RMouseButtonWait := ""
}
鼠标模拟_ToolTip(tips){
    ToolTip %tips%
    SetTimer ScrollSimulator_ToolTipRemove, -3000
}
ScrollSimulator_ToolTipRemove(){
    ToolTip
}
