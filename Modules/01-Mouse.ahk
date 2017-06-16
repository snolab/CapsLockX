CoordMode, Mouse, Screen

; 鼠标加速度微分对称模型，每秒误差 2.5ms 以内
global mtl := 0, mtr := 0, mtu := 0, tmd := 0, mvx := 0, mvy := 0

; 滚轮加速度微分对称模型（不要在意这中二的名字hhhh
global stu := 0, std := 0, stl := 0, str := 0, svx := 0, svy := 0

If(TMouse_SendInput)
    SendMode Input

; 解决多屏 DPI 问题
DllCall("Shcore.dll\SetProcessDpiAwareness", "UInt", 2)
Return

; 解决DPI比率问题
;msgbox % DllCall("MonitorFromPoint", "UInt", 0, "UInt", 0, "UInt", 0)





GetCursorHandle(){
    VarSetCapacity( PCURSORINFO, 20, 0) ;为鼠标信息 结构 设置出20字节空间
    NumPut(20, PCURSORINFO, 0, "UInt")  ;*声明出 结构 的大小cbSize = 20字节
    DllCall("GetCursorInfo", "Ptr", &PCURSORINFO) ;获取 结构-光标信息
    If(NumGet( PCURSORINFO, 4, "UInt") == 0  ) ;当光标隐藏时，直接输出特征码为0
        Return 0
    Return NumGet(PCURSORINFO, 8)
}

NewCursorShapeQ(){
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

; tooltip % A_ScreenDPI " " hMonitor " " dpiX  " "  dpiY
; wasd 控制鼠标
; e左键
; q右键


; 鼠标运动处理
mm:
    tNow := QPC()
    ; 计算用户操作时间
    tda := dt(mtl, tNow),          tdd := dt(mtr, tNow)
    tdw := dt(mtu, tNow),          tds := dt(tmd, tNow)

    ; 计算加速度
    max := ma(tdd - tda) * TMouse_MouseSpeedRatio
    may := ma(tds - tdw) * TMouse_MouseSpeedRatio

    ; 摩擦力不阻碍用户意志
    mvx := Friction(mvx + max, max), mvy := Friction(mvy + may, may)
    ;mvx //= 1, mvy //= 1
    ;mvx //= 1, mvy //= 1
    If(Abs(mvx) < 0.5)
        mvx := 0
    If(Abs(mvy) < 0.5)
        mvy := 0

    ; TODO: 输出速度时间曲线，用于DEBUG

    If(mvx Or mvy){

        ;
        If(TMouse_StopAtScreenEdge){
            MouseGetPos, xa, ya
        }

        If(TMouse_SendInputAPI){
            VarSetCapacity(sendData, 28, 0) ;为鼠标信息 结构 设置出20字节空间
            NumPut(0, sendData,  0, "UInt") ; INPUT.type
            NumPut(mvx, sendData,  4, "Int") ; INPUT.mouse_event.dx
            NumPut(mvy, sendData,  8, "Int") ; INPUT.mouse_event.dy
            NumPut(1, sendData, 16, "UInt") ; INPUT.mouse_event.dwFlags = 1/*_MOUSEEVENTF_MOVE*/
            DllCall("SendInput", "UInt", 1, "Str", sendData, "UInt", 28)
        }Else{
            MouseMove, %mvx%, %mvy%, 0, R   
        }

        ; 撞到屏幕边角就停下来
        If(TMouse_StopAtScreenEdge){
            ; 修复拖动窗口加速不对的 BUG
            If(GetKeyState("e", "P"))
                Return

            If(xa == xb)
                mvx := 0
            If(ya == yb)
                mvy := 0
            xb := xa, yb := ya
        }
        
        ; 对区域切换粘附
        ; 放在 MouseMove 后面是粘附的感觉
        ; 放在它前面是撞到东西的感觉
        If(TMouse_StickyCursor And NewCursorShapeQ()){
            mvx := 0, mvy := 0
        }

        ; ; 对屏幕边角用力穿透，并粘附( 必须放 MouseMove 下面 )
        ; If(max And Abs(mvx) > 80 And xa == xb){
        ;     If(xStop){
        ;         MouseMove, (mvx < 0 ? 3 : -3) * A_ScreenWidth, 0, 0, R
        ;         throughedScreen = 1
        ;         xStop = 0
        ;     }Else{
        ;         xStop = 1
        ;     }
        ;     mvx := 0, mvy := 0
        ; }
        ; If(may And Abs(mvy) > 80 And ya == yb){
        ;     If(yStop){
        ;         MouseMove, 0, (mvy < 0 ? 3 : -3) * A_ScreenHeight, 0, R
        ;         throughedScreen = 1
        ;         yStop = 0
        ;     }Else{
        ;         yStop = 1
        ;     }
        ;     mvx := 0, mvy := 0
        ; }

    }Else{
        SetTimer, mm, Off
    }
    Return

; 时间处理
mTick(){
    SetTimer, mm, 0
}

Pos2Long(x, y){
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


    if(A_Is64bitOS)
        ControlClass1 := DllCall( "WindowFromPoint", "int64", m_x | (m_y << 32), "Ptr")
    Else
        ControlClass1 := DllCall("WindowFromPoint", "int", m_x, "int", m_y)

    ;Detect modifer keys held down (only Shift and Control work)
    If(GetKeyState("Shift","p"))
        wParam := wParam | 0x4
    If(GetKeyState("Ctrl","p"))
        wParam := wParam | 0x8

    ; MsgBox, %ControlClass1% "\" %ControlClass2% "\" %ControlClass3%

    If(ControlClass2 == "")
    {
        PostMessage, msg, wParam, lParam, %fcontrol%, ahk_id %ControlClass1%
    }Else{
        PostMessage, msg, wParam, lParam, %fcontrol%, ahk_id %ControlClass2%
        If(ControlClass2 != ControlClass3)
            PostMessage, msg, wParam, lParam, %fcontrol%, ahk_id %ControlClass3%
    }

}
; 滚轮运动处理
msx:
    tNow := QPC()
    ; 计算用户操作时间
    tdz := dt(stl, tNow), tdc := dt(str, tNow)
    ; 计算加速度
    sax := ma(tdc - tdz) * TMouse_WheelSpeedRatio
    svx := Friction(svx + sax, sax)
    If(Abs(svx) < 0.5)
        svx := 0
    If(svx){
        ; MouseGetPos, mouseX, mouseY, wid, fcontrol
        ; wParam := svx << 16 ;zDelta
        ; lParam := Pos2Long(mouseX, mouseY)
        ; PostMessage, 0x20E, %wParam%, %lParam%, %fcontrol%, ahk_id %wid%
        ScrollMsg2(0x20E, svx)
    }Else{
        SetTimer, msx, Off
    }
    Return

msy:

    tNow := QPC()
    ; 计算用户操作时间
    tdr := dt(stu, tNow), tdf := dt(std, tNow)

    ; RF同时按下相当于中键
    If(stu And std And Abs(tdr - tdf) < 1){
        If(!GetKeyState("MButton", "P")){
            Send {MButton Down}
            stu := 0, std := 0
        }
    }Else{
        If(GetKeyState("MButton", "P") And stu == 0 And std == 0)
            Send {MButton Up}
    }
    ; 计算加速度
    say := ma(tdr - tdf) * TMouse_WheelSpeedRatio
    

    svy := Friction(svy + say, say)
    If(Abs(svy) < 0.5)
        svy := 0
    If(svy){
        ScrollMsg2(0x20A, svy)
    }Else{
        SetTimer, msy, Off
    }
    
    Return

; 时间处理
sTickx(){
    SetTimer, msx, 0
}
sTicky(){
    SetTimer, msy, 0
}

#If CapsXMode == CM_CAPSX Or !CapsX
    a:: mtl := (mtl ? mtl : QPC()), mTick()
    d:: mtr := (mtr ? mtr : QPC()), mTick()
    w:: mtu := (mtu ? mtu : QPC()), mTick()
    s:: tmd := (tmd ? tmd : QPC()), mTick()
    a Up:: mtl := 0, mTick()
    d Up:: mtr := 0, mTick()
    w Up:: mtu := 0, mTick()
    s Up:: tmd := 0, mTick()
    
    e:: LButton
    q:: RButton

    r:: stu := (stu ? stu : QPC()), sTicky()
    f:: std := (std ? std : QPC()), sTicky()
    ; z:: stl := (stl ? stl : QPC()), sTickx()
    ; c:: str := (str ? str : QPC()), sTickx()
    !r:: Send {WheelUp}
    !f:: Send {WheelDown}
    ^r:: Send ^{WheelUp}
    ^f:: Send ^{WheelDown}

    r Up:: stu := 0, sTicky()
    f Up:: std := 0, sTicky()
    ; z Up:: stl := 0, sTickx()
    ; c Up:: str := 0, sTickx()