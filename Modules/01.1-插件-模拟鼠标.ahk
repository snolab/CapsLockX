; ========== CapsLockX ==========
; 注：Save as UTF-8 with BOM please
; 名称：摸拟鼠标
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.06.27
; 版权：Copyright © 2018-2020 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========
;
; CoordMode, Mouse, Screen

AppendHelp( "
(
模拟鼠标
| CapsLockX + w a s d | 鼠标移动（上下左右） |
| CapsLockX + r f | 垂直滚轮（上下） |
| CapsLockX + R F | 水平滚轮（左右） |
| CapsLockX + rf | rf 同时按相当于鼠标中键 |
| CapsLockX + e | 鼠标左键 |
| CapsLockX + q | 鼠标右键 |
)")

; 鼠标加速度微分对称模型，每秒误差 2.5ms 以内
global 鼠刻左 := 0, 鼠刻右 := 0, 鼠刻上 := 0, 鼠刻下 := 0
global 鼠速横 := 0, 鼠速纵 := 0, 鼠差横 := 0, 鼠差纵 := 0

; 滚轮加速度微分对称模型（不要在意这中二的名字hhhh
global 轮动中 := 0
global 轮刻上 := 0, 轮刻下 := 0, 轮刻左 := 0, 轮刻右 := 0
global 轮速横 := 0, 轮速纵 := 0, 轮差横 := 0, 轮差纵 := 0
If(TMouse_SendInput)
    SendMode Input

; 解决多屏 DPI 问题
DllCall("Shcore.dll\SetProcessDpiAwareness", "UInt", 2)
; msgbox % say "_" sax "`n" 轮速纵 "_" 轮速横 "`n" lastsvy "_" lastsvx

Return

; 解决DPI比率问题
;msgbox % DllCall("MonitorFromPoint", "UInt", 0, "UInt", 0, "UInt", 0)

GetCursorHandle(){
    VarSetCapacity( PCURSORINFO, 20, 0) ;为鼠标信息 结构 设置出20字节空间
    NumPut(20, PCURSORINFO, 0, "UInt") ;*声明出 结构 的大小cbSize = 20字节
    DllCall("GetCursorInfo", "Ptr", &PCURSORINFO) ;获取 结构-光标信息
    If(NumGet( PCURSORINFO, 4, "UInt") == 0 ) ;当光标隐藏时，直接输出特征码为0
        Return 0
    Return NumGet(PCURSORINFO, 8)
}

CursorShapeChangedQ(){
    static lA_Cursor := GetCursorHandle()
    If(lA_Cursor == GetCursorHandle())
        Return 0
    lA_Cursor := GetCursorHandle()
    Return 1
}

sign(v){
    Return v == 0 ? 0 : (v > 0 ? 1 : -1)
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

; tooltip % A_ScreenDPI " " hMonitor " " dpiX " " dpiY
; wasd 控制鼠标
; e左键
; q右键

; ref: https://msdn.microsoft.com/en-us/library/windows/desktop/ms646273(v=vs.85).aspx
SendInput_MouseMsg32(dwFlag, mouseData = 0){
    VarSetCapacity(sendData, 28, 0)
    NumPut(0, sendData, 0, "UInt")
    NumPut(0, sendData, 4, "Int") 
    NumPut(0, sendData, 8, "Int") 
    NumPut(mouseData, sendData, 12, "UInt")
    NumPut(dwFlag, sendData, 16, "UInt")
    DllCall("SendInput", "UInt", 1, "Str", sendData, "UInt", 28)
}

SendInput_MouseMoveR32(x, y){
    VarSetCapacity(sendData, 28, 0)
    NumPut(0, sendData, 0, "UInt")
    NumPut(x, sendData, 4, "Int")
    NumPut(y, sendData, 8, "Int")
    NumPut(0, sendData, 12, "UInt")
    NumPut(1, sendData, 16, "UInt")
    DllCall("SendInput", "UInt", 1, "Str", sendData, "UInt", 28)
}

; TODO: 这个64位的函数不知道为啥用不了。。。
; ref: https://msdn.microsoft.com/en-us/library/windows/desktop/ms646270(v=vs.85).aspx
SendInput_MouseMoveR64(x, y){
    /*
    VarSetCapacity(sendData, 28, 0)
    NumPut(0 , sendData, 0, "UShort")
    NumPut(鼠速横, sendData, 4, "Short")
    NumPut(鼠速纵, sendData, 8, "Short")
    NumPut(0 , sendData, 12, "UShort")
    NumPut(1 , sendData, 16, "UShort")
    DllCall("SendInput", "UShort", 1, "Point", sendData, "UShort", 28)
    */

    cbSize := 24 + A_PtrSize
    VarSetCapacity(sendData, cbSize, 0) ; INPUT OBJECT
    NumPut(0, sendData, 0, "UInt")
    NumPut(x, sendData, 4, "Int")
    NumPut(y, sendData, 8, "Int")
    NumPut(0, sendData, 12, "UInt")
    NumPut(1, sendData, 16, "UInt")
    NumPut(0, sendData, 20, "UInt")
    NumPut(0, sendData, 24, "UInt")
    ; SendInput
    test := &sendData
    a0 := NumGet(sendData, 0, "UInt")
    a1 := NumGet(sendData, 4, "UInt")
    a2 := NumGet(sendData, 8, "UInt")
    a3 := NumGet(sendData, 12, "UInt")
    a4 := NumGet(sendData, 16, "UInt")
    a5 := NumGet(sendData, 20, "UInt")
    a6 := NumGet(sendData, 24, "UInt")

    ret := DllCall("SendInput", "UInt", 1, "Int", 0, "Int", cbSize, "Int")
    ToolTip, %test% %cbSize% %ErrorLevel% %A_LastError% %ret% %a0% %a1% %a2% %a3% %a4% %a5% %a6%
}

; 鼠标运动处理
; mouseTicker_dev:
;     ; 在非 CapsLockX 模式下直接停止
;     If (!(CapsLockXMode == CM_CapsLockX || CapsLockXMode == CM_FN)){
;         鼠刻左 := 0, 鼠刻右 := 0, 鼠刻上 := 0, 鼠刻下 := 0, 鼠速横 := 0, 鼠速纵 := 0, 鼠差横 := 0, 鼠差纵 := 0
;         max := 0, may := 0
;     }else{
;         tNow := QPC()
;         ; 计算用户操作时间, 计算 ADWS 键按下的时长
;         tda := dt(鼠刻左, tNow), tdd := dt(鼠刻右, tNow)
;         tdw := dt(鼠刻上, tNow), tds := dt(鼠刻下, tNow)
;         tdx := tdd - tda, tdy := tds - tdw
;     }
;     sign(tdx) + tdx * tdx
;     tax := tda * tay
;     If (TMouse_SendInputAPI && A_PtrSize == 4) ; 这只能32位用
;     {
;         SendInput_MouseMoveR32(鼠差横, 鼠差纵)
;         鼠差横 -= 鼠差横 | 0, 鼠差纵 -= 鼠差纵 | 0
;     }Else{
;         MouseMove, %鼠差横%, %鼠差纵%, 0, R
;         鼠差横 -= 鼠差横 | 0, 鼠差纵 -= 鼠差纵 | 0
;     }
; Return

; 鼠标模拟
MouseTickerStart(){
    if(!鼠动中) {
        SetTimer, MouseTicker, 1
        鼠动中 := 1
    }
}
MouseTickerStop(){
    鼠动中 := 0, 鼠刻左 := 0, 鼠刻右 := 0, 鼠刻上 := 0, 鼠刻下 := 0, 鼠速横 := 0, 鼠速纵 := 0, 鼠差横 := 0, 鼠差纵 := 0
    SetTimer, MouseTicker, Off
}
MouseTicker:
    MouseTicker()
Return
MouseTicker(){
    ; 在非 CapsLockX 模式下直接停止
    If (!(CapsLockXMode == CM_CapsLockX || CapsLockXMode == CM_FN)){
        鼠刻左 := 0, 鼠刻右 := 0, 鼠刻上 := 0, 鼠刻下 := 0, 鼠速横 := 0, 鼠速纵 := 0, 鼠差横 := 0, 鼠差纵 := 0
        max := 0, may := 0
    }else{
        tNow := QPC()
        ; 计算用户操作时间, 计算 ADWS 键按下的时长
        tda := dt(鼠刻左, tNow), tdd := dt(鼠刻右, tNow)
        tdw := dt(鼠刻上, tNow), tds := dt(鼠刻下, tNow)
        ; 计算这段时长的加速度
        ; tooltip % TMouse_MouseSpeedRatio
        max := ma(tdd - tda) * 0.5 ; * TMouse_MouseSpeedRatio
        may := ma(tds - tdw) * 0.5 ; * TMouse_MouseSpeedRatio
    }

    ; ; 摩擦力不阻碍用户意志
    鼠速横 := Friction(鼠速横 + max, max), 鼠速纵 := Friction(鼠速纵 + may, may)

    ; 实际移动需要约化
    鼠差横 += 鼠速横, 鼠差纵 += 鼠速纵

    if ( 鼠速横 == 0 && 鼠速纵 == 0){
        ; 完成移动，退出定时
        MouseTickerStop()
        Return
    }

    ; 撞到屏幕边角就停下来
    If (TMouse_StopAtScreenEdge) {
        MouseGetPos, xa, ya
    }

    If (TMouse_SendInputAPI && A_PtrSize == 4) ; 这只能32位用
    {
        SendInput_MouseMoveR32(鼠差横, 鼠差纵)
        鼠差横 -= 鼠差横 | 0, 鼠差纵 -= 鼠差纵 | 0
    }Else{
        MouseMove, %鼠差横%, %鼠差纵%, 0, R
        鼠差横 -= 鼠差横 | 0, 鼠差纵 -= 鼠差纵 | 0
    }

    ; 撞到屏幕边角就停下来
    If (TMouse_StopAtScreenEdge){
        If(xa == xb)
            鼠速横 := 0
        If(ya == yb)
            鼠速纵 := 0
        xb := xa, yb := ya
    }

    ; 对区域切换粘附
    ; 放在 MouseMove 后面是粘附的感觉
    ; 放在它前面是撞到东西的感觉
    If(TMouse_StickyCursor And CursorShapeChangedQ()){
        鼠速横 := 0, 鼠速纵 := 0
    }

    ; 对屏幕边角用力穿透，并粘附( 必须放 MouseMove 下面 )
    ; 此设定与StopAtScreenEdge不兼容
    ; If(max And Abs(鼠速横) > 80 And xa == xb){
    ; If(xStop){
    ; MouseMove, (鼠速横 < 0 ? 3 : -3) * A_ScreenWidth, 0, 0, R
    ; throughedScreen = 1
    ; xStop = 0
    ; }Else{
    ; xStop = 1
    ; }
    ; 鼠速横 := 0, 鼠速纵 := 0
    ; }
    ; If(may And Abs(鼠速纵) > 80 And ya == yb){
    ; If(yStop){
    ; MouseMove, 0, (鼠速纵 < 0 ? 3 : -3) * A_ScreenHeight, 0, R
    ; throughedScreen = 1
    ; yStop = 0
    ; }Else{
    ; yStop = 1
    ; }
    ; 鼠速横 := 0, 鼠速纵 := 0
    ; }
}

Pos2Long(x, y) {
Return x | (y << 16)
}

ScrollMsg2(msg, zDelta){
    MouseGetPos, mouseX, mouseY, wid, fcontrol
    wParam := zDelta << 16 ;zDelta
    lParam := Pos2Long(mouseX, mouseY)

    If(GetKeyState("Shift","p"))
        wParam := wParam | 0x4
    If(GetKeyState("Ctrl","p"))
        wParam := wParam | 0x8

    PostMessage, msg, %wParam%, %lParam%, %fcontrol%, ahk_id %wid%
}

ScrollMsg(msg, zDelta){
    wParam := zDelta << 16

    MouseGetPos,,,, ControlClass2, 2
    MouseGetPos,,,, ControlClass3, 3
    if (A_Is64bitOS) {
        ControlClass1 := DllCall( "WindowFromPoint", "int64", m_x | (m_y << 32), "Ptr")
    } else{
        ControlClass1 := DllCall("WindowFromPoint", "int", m_x, "int", m_y)
    }

    ;Detect modifer keys held down (only Shift and Control work)
    if GetKeyState("Shift", "p"){
        wParam := wParam | 0x4
    }
    if GetKeyState("Ctrl", "p") {
        wParam := wParam | 0x8
    }

    if (ControlClass2 == "") {
        PostMessage, msg, wParam, lParam, %fcontrol%, ahk_id %ControlClass1%
    } else {
        PostMessage, msg, wParam, lParam, %fcontrol%, ahk_id %ControlClass2%
        If(ControlClass2 != ControlClass3){
            PostMessage, msg, wParam, lParam, %fcontrol%, ahk_id %ControlClass3%
        }
    }
}
; 滚轮运动处理
ScrollTickerStart(){
    if(!轮动中){
        SetTimer, ScrollTicker, 1
        轮动中 := 1
    }
}
ScrollTicker:
    ScrollTicker()
Return
ScrollTicker(){
    ; RF同时按下相当于中键
    If(GetKeyState("MButton", "P")){
        If(轮刻上 == 0 && 轮刻下 == 0){
            Send {MButton Up}
            轮刻上 := 0, 轮刻下 := 0
            Return
        }
    }
    If(轮刻上 && 轮刻下 && Abs(tdr - tdf) < 1){
        If(!GetKeyState("MButton", "P")){
            Send {MButton Down}
            轮刻上 := 1, 轮刻下 := 1
        }
        Return
    }

    ; 在非CapsLockX模式下停止
    If (!(CapsLockXMode == CM_CapsLockX || CapsLockXMode == CM_FN)){
        轮刻上 := 0, 轮刻下 := 0, 轮刻左 := 0, 轮刻右 := 0
        轮速横 := 0, 轮速纵 := 0, 轮差横 := 0, 轮差纵 := 0
        sax := 0, say := 0
    }else{
        tNow := QPC()
        ; 计算用户操作时间
        tdz := dt(轮刻左, tNow), tdc := dt(轮刻右, tNow)
        tdr := dt(轮刻上, tNow), tdf := dt(轮刻下, tNow)
        ; 计算加速度
        say := ma(tdr - tdf) * TMouse_WheelSpeedRatio
        sax := ma(tdc - tdz) * TMouse_WheelSpeedRatio
        ; tooltip % say "_" sax
    }
    ; 计算速度
    lastsvx := 轮速横
    lastsvy := 轮速纵
    ; _ := "_", 换行 := "`n"
    ; tooltip  % tdr _ tdf 换行 say _ sax 换行 轮速纵 _ 轮速横 换行 lastsvy _ lastsvx

    轮速纵 := Friction(轮速纵 + say, say), 轮速横 := Friction(轮速横 + sax, sax)

    ; tooltip %轮刻上%`n%轮刻下%`n%轮刻左%`n%轮刻右%`n%轮速横%`n%轮速纵%`n%轮差横%`n%轮差纵%

    if ( 轮速纵 == 0 && 轮速横 == 0){
        ; 完成滚动，退出定时
        ; tooltip Done
        轮刻上 := 0, 轮刻下 := 0, 轮刻左 := 0, 轮刻右 := 0, 轮速横 := 0, 轮速纵 := 0, 轮差横 := 0, 轮差纵 := 0
        SetTimer, ScrollTicker, Off
        轮动中 := 0
        Return
    }

    轮差纵 += 轮速纵, 轮差横 += 轮速横
    ; 处理移动
    If ((轮差纵 | 0) != 0) {
        If (TMouse_SendInputAPI && A_PtrSize == 4) ; 这API只能32位环境下用
            SendInput_MouseMsg32(0x0800, 轮差纵) ; 0x0800/*MOUSEEVENTF_WHEEL*/
        Else
            ScrollMsg2(0x20A, 轮差纵)
        轮差纵 -= 轮差纵 | 0
    }
    If ((轮差横 | 0) != 0) {
        If (TMouse_SendInputAPI && A_PtrSize == 4) ; 这API只能32位环境下用
            SendInput_MouseMsg32(0x1000, 轮差横) ; 0x1000/*MOUSEEVENTF_HWHEEL*/
        Else
            ScrollMsg2(0x20E, 轮差横) ; 在64位下用的是低性能的……
        轮差横 -= 轮差横 | 0
    }
}

#If CapsLockXMode

; 鼠标按键处理
$*q:: RButton
$*e:: LButton

; 鼠标运动处理
$*a:: 鼠刻左 := (鼠刻左 ? 鼠刻左 : QPC()), MouseTickerStart()
$*d:: 鼠刻右 := (鼠刻右 ? 鼠刻右 : QPC()), MouseTickerStart()
$*w:: 鼠刻上 := (鼠刻上 ? 鼠刻上 : QPC()), MouseTickerStart()
$*s:: 鼠刻下 := (鼠刻下 ? 鼠刻下 : QPC()), MouseTickerStart()
$*a Up:: 鼠刻左 := 0, MouseTickerStart()
$*d Up:: 鼠刻右 := 0, MouseTickerStart()
$*w Up:: 鼠刻上 := 0, MouseTickerStart()
$*s Up:: 鼠刻下 := 0, MouseTickerStart()

; 鼠标滚轮处理
$*r:: 轮刻上 := (轮刻上 ? 轮刻上 : QPC()), ScrollTickerStart()
$*f:: 轮刻下 := (轮刻下 ? 轮刻下 : QPC()), ScrollTickerStart()
$*r Up:: 轮刻上 := 0, ScrollTickerStart()
$*f Up:: 轮刻下 := 0, ScrollTickerStart()

; Alt 单格滚动
$!r:: Send {WheelUp}
$!f:: Send {WheelDown}
$!^r:: Send ^{WheelUp}
$!^f:: Send ^{WheelDown}

; Ctrl 缩放
$^r:: Send ^{WheelUp}
$^f:: Send ^{WheelDown}

; Shift 横向滚动
$+r:: 轮刻左 := (轮刻左 ? 轮刻左 : QPC()), ScrollTickerStart()
$+f:: 轮刻右 := (轮刻右 ? 轮刻右 : QPC()), ScrollTickerStart()
$+r Up:: 轮刻左 := 0, ScrollTickerStart()
$+f Up:: 轮刻右 := 0, ScrollTickerStart()

