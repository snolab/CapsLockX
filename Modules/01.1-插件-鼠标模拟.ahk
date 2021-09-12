; ========== CapsLockX ==========
; 注：Save as UTF-8 with BOM please
; 名称：摸拟鼠标
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.06.27
; 版权：Copyright © 2017-2021 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========

if (!CapsLockX) {
    MsgBox, % "本模块只为 CapsLockX 工作"
    ExitApp
}

global TMouse_Disabled := CapsLockX_Config("TMouse", "Disabled", 0, "禁用模拟鼠标模块")
global TMouse_SendInput := CapsLockX_Config("TMouse", "SendInput", 1, "使用 SendInput 方法提高模拟鼠标点击、移动性能")
global TMouse_SendInputAPI := CapsLockX_Config("TMouse", "SendInputAPI", 1, "使用 Windows API 强势提升模拟鼠标移动性能")
global TMouse_StickyCursor := CapsLockX_Config("TMouse", "StickyCursor", 1, "启用自动粘附各种按钮，编辑框")
global TMouse_StopAtScreenEdge := CapsLockX_Config("TMouse", "StopAtScreenEdge", 1, "撞上屏幕边界后停止加速")

; 根据屏幕 DPI 比率，自动计算，得出，如果数值不对，才需要纠正
global TMouse_UseDPIRatio := CapsLockX_Config("TMouse", "UseDPIRatio", 1, "是否根据屏幕 DPI 比率缩放鼠标速度")
global TMouse_MouseSpeedRatio := CapsLockX_Config("TMouse", "MouseSpeedRatio", 1, "鼠标加速度比率, 默认为 1, 你想慢点就改成 0.5 之类")
global TMouse_WheelSpeedRatio := CapsLockX_Config("TMouse", "WheelSpeedRatio", 1, "滚轮加速度比率, 默认为 1, 你想慢点就改成 0.5 之类")
global TMouse_DPIRatio := TMouse_UseDPIRatio ? A_ScreenDPI / 96 : 1

CapsLockX_AppendHelp( CapsLockX_LoadHelpFrom("Modules/01.1-插件-鼠标模拟.md" ))
; global debug_fps := new FPS_Debugger()
global 鼠标模拟 := new AccModel2D(Func("鼠标模拟"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global 滚轮模拟 := new AccModel2D(Func("滚轮模拟"), 0.1, TMouse_DPIRatio * 360 * TMouse_WheelSpeedRatio)
global 滚轮自控 := new AccModel2D(Func("滚轮自控"), 0.1, 10)
global 滚轮自动 := new AccModel2D(Func("滚轮自动"), 0, 1)

if (TMouse_SendInput)
    SendMode Input

; 解决多屏 DPI 问题
DllCall("Shcore.dll\SetProcessDpiAwareness", "UInt", 2)

Return

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

; TODO: 这个的64位的函数不知道为啥用不了。。。
; ref: https://msdn.microsoft.com/en-us/library/windows/desktop/ms646270(v=vs.85).aspx
SendInput_MouseMoveR32(x, y)
{
    VarSetCapacity(sendData, 28, 0)
    NumPut(0, sendData, 0, "UInt")
    NumPut(x, sendData, 4, "Int")
    NumPut(y, sendData, 8, "Int")
    NumPut(0, sendData, 12, "UInt")
    NumPut(1, sendData, 16, "UInt")
    DllCall("SendInput", "UInt", 1, "Str", sendData, "UInt", 28)
}

; 鼠标模拟
鼠标模拟(dx, dy, 状态)
{
    if (状态 == "横中键") {
        SendEvent {Click 2}
        return
    }
    if (状态 == "纵中键") {
        SendEvent {Click 3}
        return
    }
    
    if (TMouse_SendInputAPI && A_PtrSize == 4) {
        ; 这只能32位用
        SendInput_MouseMoveR32(dx, dy)
    } else {
        MouseMove, %dx%, %dy%, 0, R
    }
    
    ; TODO: 撞到屏幕边角就停下来
    ; if(TMouse_StopAtScreenEdge )
    ; MouseGetPos, xb, yb
    ; 鼠标模拟.横速 *= dx && xa == xb ? 0 : 1
    ; 鼠标模拟.纵速 *= dy && ya == yb ? 0 : 1
    
    
    ; 在各种按钮上减速
    if (TMouse_StickyCursor && CursorShapeChangedQ()) {
        鼠标模拟.横速 *= 0.5
        鼠标模拟.纵速 *= 0.5
    }
}

滚轮自动(dx, dy, 状态){
    WM_MOUSEWHEEL := 0x020A
    WM_MOUSEWHEELH := 0x020E
    _:= dy &&  滚轮消息发送(WM_MOUSEWHEEL, -dy)
    _:= dx &&  滚轮消息发送(WM_MOUSEWHEELH, dx)
}
滚轮自控(dx, dy, 状态)
{
    滚轮自动.横速 += dx, 滚轮自动.纵速 += dy, 滚轮自动.始动()
    msg := "【雪星滚轮自动v2】`n"
    msg .= "横：" (滚轮自动.横速|0) "px/s`n纵：" (滚轮自动.纵速|0)  "px/s`n"
    msg .= "CapsLockX + Ctrl + Alt + RF 调整纵向自动滚轮`n"
    msg .= "CapsLockX + Ctrl + Alt + Shift + RF 调整横向自动滚轮`n"
    鼠标模拟_ToolTip(msg)
}
滚轮模拟(dx, dy, 状态)
{
    if (状态 != "移动") {
        SendEvent {Blind}{MButton Down}
        KeyWait r
        KeyWait f
        SendEvent {Blind}{MButton Up}
        ; 关闭滚轮自动
        滚轮自横:=0, 滚轮自纵:=0
        return
    }
    WM_MOUSEWHEEL := 0x020A
    WM_MOUSEWHEELH := 0x020E
    _:= dy &&  滚轮消息发送(WM_MOUSEWHEEL, -dy)
    _:= dx &&  滚轮消息发送(WM_MOUSEWHEELH, dx)
}

滚轮消息发送(msg, zDelta)
{
    ; 目前还不支持UWP
    CoordMode, Mouse, Screen
    MouseGetPos, x, y, wid, fcontrol
    
    wParam := zDelta << 16 ;zDelta
    lParam := x | (y << 16) ; pos2long
    
    MouseGetPos, , , , ControlClass2, 2
    MouseGetPos, , , , , ControlClass3, 3
    if (A_Is64bitOS) {
        ControlClass1 := DllCall("WindowFromPoint", "int64", x | (y << 32), "Ptr")|0x0
    } else {
        ControlClass1 := DllCall("WindowFromPoint", "int", x, "int", y) |0x0
    }
    ;Detect modifer keys held down (only Shift and Control work)
    wParam |= GetKeyState("Shift", "p") ? 0x4 : 0
    wParam |= GetKeyState("Ctrl", "p")  ? 0x8 : 0
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

CapsLockX_鼠标左键按下(){
    SendEvent {Blind}{LButton Down}
    KeyWait, e, ; wait forever 防止重复触发
}
CapsLockX_鼠标右键按下(){
    SendEvent {Blind}{RButton Down}
    KeyWait, q, ; wait forever 防止重复触发
}

鼠标模拟_ToolTip(tips){
    ToolTip %tips%
    SetTimer 鼠标模拟_ToolTipRemove, -3000
}
鼠标模拟_ToolTipRemove(){
    ToolTip
}

#if CapsLockXMode
    
; 鼠标按键处理
$*e::CapsLockX_鼠标左键按下()
$*q:: CapsLockX_鼠标右键按下()
$*e Up:: SendEvent {Blind}{LButton Up}
$*q Up:: SendEvent {Blind}{RButton Up}
; 鼠标运动处理
$*a:: 鼠标模拟.左按()
$*d:: 鼠标模拟.右按()
$*w:: 鼠标模拟.上按()
$*s:: 鼠标模拟.下按()
$*a Up:: 鼠标模拟.左放()
$*d Up:: 鼠标模拟.右放()
$*w Up:: 鼠标模拟.上放()
$*s Up:: 鼠标模拟.下放()
; 滚轮运动处理
$*+^!r:: 滚轮自控.左按()
$*+^!f:: 滚轮自控.右按()
$*^!r:: 滚轮自控.上按()
$*^!f:: 滚轮自控.下按()
$*+r:: 滚轮模拟.左按()
$*+f:: 滚轮模拟.右按()
$*r:: 滚轮模拟.上按()
$*f:: 滚轮模拟.下按()
$*r Up:: 滚轮模拟.上放(), 滚轮模拟.左放(), 滚轮自控.上放(), 滚轮自控.左放()
$*f Up:: 滚轮模拟.下放(), 滚轮模拟.右放(), 滚轮自控.下放(), 滚轮自控.右放()
