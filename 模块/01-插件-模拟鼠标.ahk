CoordMode, Mouse, Screen

; 鼠标加速度微分对称模型，每秒误差 2.5ms 以内
global mtl := 0, mtr := 0, mtu := 0, mtd := 0, mvx := 0, mvy := 0, mdx := 0, mdy := 0

; 滚轮加速度微分对称模型（不要在意这中二的名字hhhh
global stu := 0, std := 0, stl := 0, str := 0, svx := 0, svy := 0, sdx := 0, sdy := 0

If(TMouse_SendInput)
    SendMode Input

; 解决多屏 DPI 问题
DllCall("Shcore.dll\SetProcessDpiAwareness", "UInt", 2)

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
    NumPut(mvx, sendData, 4, "Short")
    NumPut(mvy, sendData, 8, "Short")
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

; `\::
; SendInput_MouseMoveR64(1111, 1111)
; Return

; 鼠标运动处理
mTicker:
    ; 在非 CapslockX 模式下直接停止
    If (!(CapslockXMode == CM_CapslockX || CapslockXMode == CM_FN)){
        max := 0
        may := 0
    }else{
        tNow := QPC()
        ; 计算用户操作时间, 计算 ADWS 键按下的时长
        tda := dt(mtl, tNow), tdd := dt(mtr, tNow)
        tdw := dt(mtu, tNow), tds := dt(mtd, tNow)
        ; 计算这段时长的加速度
        max := ma(tdd - tda) * TMouse_MouseSpeedRatio
        may := ma(tds - tdw) * TMouse_MouseSpeedRatio
    }
    
    ; ; 摩擦力不阻碍用户意志
    mvx := Friction(mvx + max, max), mvy := Friction(mvy + may, may)
    
    ; 实际移动需要约化
    mdx += mvx, mdy += mvy
    
    if ( mvx == 0 && mvy == 0){
        ; 完成移动，退出定时
        SetTimer, mTicker, Off
        Return
    }
    
    If (TMouse_StopAtScreenEdge) {
        MouseGetPos, xa, ya
    }
    
    If (TMouse_SendInputAPI && A_PtrSize == 4) ; 这只能32位用
    {
        SendInput_MouseMoveR32(mdx, mdy)
        mdx -= mdx | 0, mdy -= mdy | 0
    }Else{
        MouseMove, %mdx%, %mdy%, 0, R 
        mdx -= mdx | 0, mdy -= mdy | 0
    }
    
    ; 撞到屏幕边角就停下来
    If (TMouse_StopAtScreenEdge){
        If(xa == xb)
            mvx := 0
        If(ya == yb)
            mvy := 0
        xb := xa, yb := ya
    }
    
    ; 对区域切换粘附
    ; 放在 MouseMove 后面是粘附的感觉
    ; 放在它前面是撞到东西的感觉
    If(TMouse_StickyCursor And CursorShapeChangedQ()){
        mvx := 0, mvy := 0
    }
    
    ; 对屏幕边角用力穿透，并粘附( 必须放 MouseMove 下面 )
    ; 此设定与StopAtScreenEdge不兼容
    ; If(max And Abs(mvx) > 80 And xa == xb){
    ; If(xStop){
    ; MouseMove, (mvx < 0 ? 3 : -3) * A_ScreenWidth, 0, 0, R
    ; throughedScreen = 1
    ; xStop = 0
    ; }Else{
    ; xStop = 1
    ; }
    ; mvx := 0, mvy := 0
    ; }
    ; If(may And Abs(mvy) > 80 And ya == yb){
    ; If(yStop){
    ; MouseMove, 0, (mvy < 0 ? 3 : -3) * A_ScreenHeight, 0, R
    ; throughedScreen = 1
    ; yStop = 0
    ; }Else{
    ; yStop = 1
    ; }
    ; mvx := 0, mvy := 0
    ; }
    
Return

; 时间处理
mTick(){
    SetTimer, mTicker, 0
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
    
    if (A_Is64bitOS)
        ControlClass1 := DllCall( "WindowFromPoint", "int64", m_x | (m_y << 32), "Ptr")
    Else
        ControlClass1 := DllCall("WindowFromPoint", "int", m_x, "int", m_y)
    
    ;Detect modifer keys held down (only Shift and Control work)
    If(GetKeyState("Shift", "p"))
        wParam := wParam | 0x4
    If(GetKeyState("Ctrl", "p"))
        wParam := wParam | 0x8
    
    ; MsgBox, %ControlClass1% "\" %ControlClass2% "\" %ControlClass3%
    
    If(ControlClass2 == ""){
        PostMessage, msg, wParam, lParam, %fcontrol%, ahk_id %ControlClass1%
    }Else{
        PostMessage, msg, wParam, lParam, %fcontrol%, ahk_id %ControlClass2%
        If(ControlClass2 != ControlClass3)
            PostMessage, msg, wParam, lParam, %fcontrol%, ahk_id %ControlClass3%
    }
}
; 滚轮运动处理
sTicker:
    ; RF同时按下相当于中键
    If(GetKeyState("MButton", "P")){
        If(stu == 0 And std == 0){
            Send {MButton Up}
            stu := 0, std := 0
            Return
        }
    }
    If(stu And std And Abs(tdr - tdf) < 1){
        If(!GetKeyState("MButton", "P")){
            Send {MButton Down}
            stu := 1, std := 1
        }
        Return
    }
    
    ; 在非CapslockX模式下停止
    If (!(CapslockXMode == CM_CapslockX || CapslockXMode == CM_FN)){
        sax := 0
        say := 0
    }else{
        tNow := QPC()
        ; 计算用户操作时间
        tdz := dt(stl, tNow), tdc := dt(str, tNow)
        tdr := dt(stu, tNow), tdf := dt(std, tNow)
        ; 计算加速度
        say := ma(tdr - tdf) * TMouse_WheelSpeedRatio
        sax := ma(tdc - tdz) * TMouse_WheelSpeedRatio
    }
    ; 计算速度
    lastsvy := svy
    lastsvx := svx
    
    svy := Friction(svy + say, say), svx := Friction(svx + sax, sax)
    
    ; ?
    ; If ( (lastsvx > 0 && svx < 0) || (lastsvx < 0 && svx > 0) || (lastsvy > 0 && svy < 0) || (lastsvy < 0 && svy > 0) ){
    ; svx := 0
    ; svy := 0
    ; }
    
    ; ; 稳定化
    ; If(Abs(svx) < 0.02)
    ; svx := 0
    ; If(Abs(svy) < 0.02)
    ; svy := 0
    ; ; ToolTip, %sax% %say% %svx% %svy%
    
    if ( svy == 0 && svx == 0){
        ; 完成滚动，退出定时
        ; tooltip Done
        SetTimer, sTicker, Off
        Return
    }
    
    sdy+= svy, sdx += svx
    ; 处理移动
    If ((sdy|0) != 0) {
        If (TMouse_SendInputAPI && A_PtrSize == 4) ; 这API只能32位环境下用
            SendInput_MouseMsg32(0x0800, sdy) ; 0x0800/*MOUSEEVENTF_WHEEL*/
        Else
            ScrollMsg2(0x20A, sdy)
        sdy -= sdy | 0
    }
    If ((sdx|0) != 0) {
        If (TMouse_SendInputAPI && A_PtrSize == 4) ; 这API只能32位环境下用
            SendInput_MouseMsg32(0x1000, sdx) ; 0x1000/*MOUSEEVENTF_HWHEEL*/
        Else
            ScrollMsg2(0x20E, sdx) ; 在64位下用的是低性能的……
        sdx -= sdx | 0
    }
Return

; 时间处理
sTick(){
    SetTimer, sTicker, 0
}

; CapslockX和fn模式都能触发
#If CapslockXMode == CM_CapslockX || CapslockXMode == CM_FN
    ; 鼠标按键处理
*e:: Send {Blind}{LButton Down}
*e up:: Send {Blind}{LButton Up}
*q::
    If(TMouse_SendInputAPI && A_PtrSize == 4) ; 这API只能32位用
        SendInput_MouseMsg32(8) ; 8/*_MOUSEEVENTF_RIGHTDOWN*/
    Else
        Send {Blind}{RButton Down}
Return
*q up::
    If(TMouse_SendInputAPI && A_PtrSize == 4) ; 这API只能32位用
        SendInput_MouseMsg32(16) ; 16/*_MOUSEEVENTF_RIGHTUP*/
    Else
        Send {Blind}{RButton Up}
Return

; 只有开启CapslockX模式能触发
; #If CapslockXMode == CM_CapslockX
; 鼠标运动处理
a:: mtl := (mtl ? mtl : QPC()), mTick()
d:: mtr := (mtr ? mtr : QPC()), mTick()
w:: mtu := (mtu ? mtu : QPC()), mTick()
s:: mtd := (mtd ? mtd : QPC()), mTick()
a Up:: mtl := 0, mTick()
d Up:: mtr := 0, mTick()
w Up:: mtu := 0, mTick()
s Up:: mtd := 0, mTick()

; 鼠标滚轮处理
r:: stu := (stu ? stu : QPC()), sTick()
f:: std := (std ? std : QPC()), sTick()
+r:: stl := (stl ? stl : QPC()), sTick()
+f:: str := (str ? str : QPC()), sTick()
; +z:: stl := (stl ? stl : QPC()), sTickx()
; +c:: str := (str ? str : QPC()), sTickx()
; !r:: Send {WheelUp}
; !f:: Send {WheelDown}
; ^r:: Send ^{WheelUp}
; ^f:: Send ^{WheelDown}

r Up:: stu := 0, sTick()
f Up:: std := 0, sTick()
+r Up:: stl := 0, sTick()
+f Up:: str := 0, sTick()
; z Up:: stl := 0, sTickx()
; c Up:: str := 0, sTickx()
