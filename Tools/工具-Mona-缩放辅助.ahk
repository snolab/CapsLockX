CoordMode, Mouse, Screen
SetTitleMatchMode RegEx
SendMode Input

global TMouse_WheelSpeedRatio
global TMouse_DPIRatio
TMouse_WheelSpeedRatio := TMouse_WheelSpeedRatio ? TMouse_WheelSpeedRatio : 1
TMouse_DPIRatio := TMouse_DPIRatio ? TMouse_DPIRatio : A_ScreenDPI / 96

; 滚轮加速度微分对称模型（不要在意这中二的名字hhhh
global stu := 0, std := 0, stl := 0, str := 0, svx := 0, svy := 0

; 解决多屏 DPI 问题
DllCall("Shcore.dll\SetProcessDpiAwareness", "UInt", 2)

; 高性能计时
QPF(){
    DllCall("QueryPerformanceFrequency", "Int64*", QuadPart)
    Return QuadPart
}
QPC(){
    DllCall("QueryPerformanceCounter", "Int64*", Counter)
    Return Counter
}


; 构造加速模型相关函数
ma(t){
    ; Return ma2(t) ; 二次函数运动模型
    ; Return ma3(t) ; 三次函数运动模型
    Return maPower(t) * TMouse_DPIRatio ; 指数函数运动模型
}
ma2(t){
    ; x-t 二次曲线加速运动模型
    ; 跟现实世界的运动一个感觉
    If(0 == t)
        Return 0
    If(t > 0)
        Return  3
    Else
        Return -3
}

ma3(t){
    ; x-t 三次曲线函数运动模型
    ; 与现实世界不同，
    ; 这个模型会让人感觉鼠标比较“重”
    ;
    If(0 == t)
        Return 0
    If(t > 0)
        Return  1 + t * 6
    Else
        Return -1 + t * 6
}

maPower(t){
    ; x-t 指数曲线运动的简化模型
    ; 这个模型可以满足精确定位需求，也不会感到鼠标“重”
    ; 但是因为跟现实世界的运动曲线不一样，凭直觉比较难判断落点，需要一定练习才能掌握。
    ;
    If(0 == t)
        Return 0
    If(t > 0)
        Return  1 +( Exp( t) - 1 ) * 8
    Else
        Return -1 -( Exp(-t) - 1 ) * 8
}

; 时间计算
dt(t, tNow){
    Return t ? (tNow - t) / QPF() : 0
}

Friction(v, a){ ; 摩擦力
    ; 限制最大速度
    ; maxSpeed := 80
    ; If(v   < -maxSpeed)
    ;     v := -maxSpeed
    ; If(v   >  maxSpeed)
    ;     v :=  maxSpeed

    ; 摩擦力不阻碍用户意志
    If((a > 0 And v > 0) Or (a < 0 And v < 0)){
        Return v
    }
    ; 摩擦系数无限大
    Return 0

    ; ; 简单粗暴倍数降速
    ; v *= 0.9
    ; If(v > 0)
    ;     v -= 1
    ; If(v < 0)
    ;     v += 1
    ; Return v
}

; ref: https://msdn.microsoft.com/en-us/library/windows/desktop/ms646273(v=vs.85).aspx
SendInput_MouseMsg(dwFlag, mouseData = 0){
    VarSetCapacity(sendData, 28, 0)
    NumPut(0, sendData,  0, "UInt")
    NumPut(0, sendData,  4, "Int") 
    NumPut(0, sendData,  8, "Int") 
    NumPut(mouseData, sendData, 12, "UInt")
    NumPut(dwFlag, sendData, 16, "UInt")
    DllCall("SendInput", "UInt", 1, "Str", sendData, "UInt", 28)
}
SendInput_MouseMoveR(x, y){
    VarSetCapacity(sendData, 28, 0)
    NumPut(0, sendData,  0, "UInt")
    NumPut(mvx, sendData,  4, "Int")
    NumPut(mvy, sendData,  8, "Int")
    NumPut(1, sendData, 16, "UInt")
    DllCall("SendInput", "UInt", 1, "Str", sendData, "UInt", 28)
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
        SendInput_MouseMsg(0x01000, svy) ; 0x01000/*MOUSEEVENTF_HWHEEL*/
    }Else{
        SetTimer, msx, Off
    }
    Return

msy:
    tNow := QPC()

    ; 计算用户操作时间
    tdu := dt(stu, tNow), tdd := dt(std, tNow)

    ; RF同时按下相当于中键
    If(GetKeyState("MButton", "P")){
        If(stu == 0 Or std == 0){
            Send {MButton Up}
            stu := 0, std := 0
        }
        Return
    }
    If(stu And std And Abs(tdu - tdd) < 1){
        If(!GetKeyState("MButton", "P")){
            Send {MButton Down}
        }
        Return
    }

    ; 计算加速度
    say := ma(tdu - tdd) * TMouse_WheelSpeedRatio
    svy := Friction(svy + say, say)

    If(Abs(svy) < 0.5)
        svy := 0
    If(svy){
        SendInput_MouseMsg(0x0800, svy) ; 0x0800/*MOUSEEVENTF_WHEEL*/
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


#IfWinActive Mona_.* ahk_class Qt5QWindowIcon ahk_exe Mona.exe
    ; *e::    Send {Blind}{LButton Down}
    ; *e up:: Send {Blind}{LButton Up}
    ; *q::
    ;     If(TMouse_SendInputAPI)
    ;         SendInput_MouseMsg(8) ; 8/*_MOUSEEVENTF_RIGHTDOWN*/
    ;     Else
    ;         Send {Blind}{RButton Down}
    ;     Return
    ; *q up::
    ;     If(TMouse_SendInputAPI)
    ;         SendInput_MouseMsg(16) ; 16/*_MOUSEEVENTF_RIGHTUP*/
    ;     Else
    ;         Send {Blind}{RButton Up}
    ;     Return

    x:: RButton

    c:: stu := (stu ? stu : QPC()), sTicky()
    v:: std := (std ? std : QPC()), sTicky()
    ; z:: stl := (stl ? stl : QPC()), sTickx()
    ; c:: str := (str ? str : QPC()), sTickx()
    ; !r:: Send {WheelUp}
    ; !f:: Send {WheelDown}
    ; ^r:: Send ^{WheelUp}
    ; ^f:: Send ^{WheelDown}

    c Up:: stu := 0, sTicky()
    v Up:: std := 0, sTicky()
    ; z Up:: stl := 0, sTickx()
    ; c Up:: str := 0, sTickx()
