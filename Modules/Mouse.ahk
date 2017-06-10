CoordMode, Mouse, Screen
CoordMode, Pixel, Screen


; 调试用
^F12:: ExitApp

; wasd 控制鼠标
; e左键
; q右键



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
    Return maPower(t) ; 指数函数运动模型
}
ma2(t){
    ; x-t 二次曲线加速运动模型
    ; 跟现实世界的运动一个感觉
    If(0 == t)
        Return 0
    If(t > 0)
        Return  6
    Else
        Return -6
}

ma3(t){
    ; x-t 三次曲线函数运动模型
    ; 与现实世界不同，
    ; 这个模型会让人感觉鼠标比较“重”
    ;
    If(0 == t)
        Return 0
    If(t > 0)
        Return t * 12
    Else
        Return t * 12
}

maPower(t){
    ; x-t 指数曲线运动的简化模型
    ; 这个模型可以满足精确定位需求，也不会感到鼠标“重”
    ; 但是因为跟现实世界的运动曲线不一样，凭直觉比较难判断落点，需要一定练习才能掌握。
    ;
    If(0 == t)
        Return 0
    If(t > 0)
        Return  ( Exp( t) - 0.95 ) * 16
    Else
        Return -( Exp(-t) - 0.95 ) * 16
}

; 时间计算
dt(t, tNow){
    Return t ? (tNow - t) / QPF() : 0
}



GetCursorShape(){   ;获取光标特征码 by nnrxin  
    VarSetCapacity( PCURSORINFO, 20, 0) ;为鼠标信息 结构 设置出20字节空间
    NumPut(20, PCURSORINFO, 0, "UInt")  ;*声明出 结构 的大小cbSize = 20字节
    DllCall("GetCursorInfo", "Ptr", &PCURSORINFO) ;获取 结构-光标信息
    if ( NumGet( PCURSORINFO, 4, "UInt")="0" ) ;当光标隐藏时，直接输出特征码为0
        return, 0
    VarSetCapacity( ICONINFO, 20, 0) ;创建 结构-图标信息
    DllCall("GetIconInfo", "Ptr", NumGet(PCURSORINFO, 8), "Ptr", &ICONINFO)  ;获取 结构-图标信息
    VarSetCapacity( lpvMaskBits, 128, 0) ;创造 数组-掩图信息（128字节）
    DllCall("GetBitmapBits", "Ptr", NumGet( ICONINFO, 12), "UInt", 128, "UInt", &lpvMaskBits)  ;读取 数组-掩图信息
    loop, 128{ ;掩图码
        MaskCode += NumGet( lpvMaskBits, A_Index, "UChar")  ;累加拼合
    }
    if (NumGet( ICONINFO, 16, "UInt")<>"0"){ ;颜色图不为空时（彩色图标时）
        VarSetCapacity( lpvColorBits, 4096, 0)  ;创造 数组-色图信息（4096字节）
        DllCall("GetBitmapBits", "Ptr", NumGet( ICONINFO, 16), "UInt", 4096, "UInt", &lpvColorBits)  ;读取 数组-色图信息
        loop, 256{ ;色图码
            ColorCode += NumGet( lpvColorBits, A_Index*16-3, "UChar")  ;累加拼合
        }  
    } else
        ColorCode := "0"
    DllCall("DeleteObject", "Ptr", NumGet( ICONINFO, 12))  ; *清理掩图
    DllCall("DeleteObject", "Ptr", NumGet( ICONINFO, 16))  ; *清理色图
    VarSetCapacity( PCURSORINFO, 0) ;清空 结构-光标信息
    VarSetCapacity( ICONINFO, 0) ;清空 结构-图标信息
    VarSetCapacity( lpvMaskBits, 0)  ;清空 数组-掩图
    VarSetCapacity( lpvColorBits, 0)  ;清空 数组-色图
    return, % MaskCode//2 . ColorCode  ;输出特征码
}



MoCaLi(v, a){ ; 摩擦力
    ; 限制最大速度
    ; maxSpeed := 80
    ; If(v   < -maxSpeed)
    ;     v := -maxSpeed
    ; If(v   >  maxSpeed)
    ;     v :=  maxSpeed
    ; 摩擦力不阻碍用户意志
    If((a > 0 And v > 0) Or (a < 0 And v < 0))
        Return v
    ; 简单粗暴倍数降速
    v *= 0.8
    If(v > 0)
        v -= 1
    If(v < 0)
        v += 1
    v //= 1
    Return v
}


; 鼠标加速度微分对称模型，每秒误差 2.5ms 以内
Global mtl := 0, mtr := 0, mtu := 0, tmd := 0, mvx := 0, mvy := 0

; 滚轮加速度微分对称模型（不要在意这中二的名字hhhh
Global stu := 0, std := 0, stl := 0, str := 0, svx := 0, svy := 0


; 鼠标运动处理
mm:
    tNow := QPC()
    ; 计算用户操作时间
    tda := dt(mtl, tNow),          tdd := dt(mtr, tNow)
    tdw := dt(mtu, tNow),          tds := dt(tmd, tNow)

    ; 计算加速度
    max := ma(tdd - tda),          may := ma(tds - tdw)

    ; 摩擦力不阻碍用户意志
    mvx := MoCaLi(mvx + max, max), mvy := MoCaLi(mvy + may, may)

    If(mvx Or mvy){
        MouseMove, %mvx%, %mvy%, 0, R
    }Else{
        SetTimer, mm, Off
    }
    Return

; 时间处理
mTick(){
    SetTimer, mm, 1
}

Pos2Long(x, y){
    Return x | (y << 16)
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
    sax := ma(tdc - tdz)
    svx := MoCaLi(svx + sax, sax)


    If(svx){
        MouseGetPos, mouseX, mouseY, wid, fcontrol
        wParam := svx << 16 ;zDelta
        lParam := Pos2Long(mouseX, mouseY)
        PostMessage, 0x20E, %wParam%, %lParam%, %fcontrol%, ahk_id %wid%
    }Else{
        SetTimer, msx, Off
    }
    Return

msy:
    tNow := QPC()
    ; 计算用户操作时间
    tdr := dt(stu, tNow), tdf := dt(std, tNow)
    ; 计算加速度
    say := ma(tdr - tdf)
    svy := MoCaLi(svy + say, say)

    If(svy){
        MouseGetPos, mouseX, mouseY, id, fcontrol
        wParam := svy << 16 ;zDelta
        lParam := Pos2Long(mouseX, mouseY)
        PostMessage, 0x20A, %wParam%, %lParam%, %fcontrol%, ahk_id %id%
    ;    ScrollMsg(0x20A, svy)
    }Else{
        SetTimer, msy, Off
    }
    
    Return

; 时间处理
sTickx(){
    SetTimer, msx, 1
}
sTicky(){
    SetTimer, msy, 1
}



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
+r:: stl := (stl ? stl : QPC()), sTickx()
+f:: str := (str ? str : QPC()), sTickx()

r Up:: stu := 0, sTicky()
f Up:: std := 0, sTicky()
+r Up:: stl := 0, sTickx()
+f Up:: str := 0, sTickx()