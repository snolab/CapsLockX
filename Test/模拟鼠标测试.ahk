#SingleInstance, force
; 鼠标加速度微分对称模型，每秒误差 2.5ms 以内
global mtl := 0, mtr := 0, mtu := 0, mtd := 0
global mtx := 0, mty := 0, mdx := 0, mdy := 0

Return

; 移动鼠标
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

MouseMove(mdx, mdy)
{
    if (TMouse_SendInputAPI && A_PtrSize == 4) ; 这只能32位用
    {
        SendInput_MouseMoveR32(mdx, mdy)
    } else {
        MouseMove, %mdx%, %mdy%, 0, R
    }
}

; 高性能计时器，精度能够达到微秒级，相比之下 A_Tick 的精度大概只有10几ms。
QPC()
{
    DllCall("QueryPerformanceCounter", "Int64*", Counter)
    Return Counter
}
QPF()
{
    DllCall("QueryPerformanceFrequency", "Int64*", QuadPart)
    Return QuadPart
}
; 高性能计时器，精度能够达到微秒级，相比之下 A_Tick 的精度大概只有10几ms。
QuerySeconds()
{
    DllCall("QueryPerformanceCounter", "Int64*", Counter)
    DllCall("QueryPerformanceFrequency", "Int64*", QuadPart)
    Return Counter / QuadPart
}
; 时差计算
dt(t, t2)
{
    Return t ? (t2 - t) / QPF() : 0
}
sign(x)
{
    return x == 0 ? 0 : x > 0 ? 1 : -1
}
calcPos(t)
{
    Return sign( t  ) * Abs( 1 + t * t * A_ScreenWidth ) ; 2次
    ; Return sign( t ) * Abs( 1 + t * t * t * A_ScreenWidth ) ; 3次
    ; Return sign( t ) * ( 1 + ((Exp(Abs(t))-1)/(Exp(1)-1))  * A_ScreenWidth ) ; 指数
}
mTick:
    tNow := QPC()
    ; 计算用户操作时间, 计算 ADWS 键按下的时长及时长差 (秒)
    tda := dt(mtl, tNow), tdd := dt(mtr, tNow)
    tdw := dt(mtu, tNow), tds := dt(mtd, tNow)
    
    ; 目标为 1 秒划过屏幕
    ; tdx *= 2, tdy *= 2
    
    ; tdx := tdd - tda, tdy := tds - tdw
    
    _mtx := calcPos(tdd) - calcPos(tda)
    _mty := calcPos(tds) - calcPos(tdw)
    
    ; 计算新坐标
    ; _mtx := tdx == 0 ? mtx : sign( tdx ) * Abs( 1 + tdx * tdx* tdx * A_ScreenWidth )
    ; _mty := tdy == 0 ? mty : sign( tdy ) * Abs( 1 + tdy * tdy* tdy * A_ScreenHeight )
    ; if(tdx == 0) {
    ;     mtx := 0, _mtx :=0
    ; } else {
    ; }
    ; if(tdy == 0) {
    ;     mty := 0, _mty :=0
    ; } else {
    ;     ; _mty := sign( tdy ) * Abs( 1 + tdy * tdy * A_ScreenHeight ) ; 2次
    ;     _mty := sign( tdy ) * Abs( 1 + tdy * tdy * tdy * A_ScreenHeight ) ; 3次
    ;     ; _mty := sign( tdy ) * ( 1 + ((Exp(Abs(tdy))-1)/(Exp(1)-1)) * A_ScreenWidth ) ;
    ; }
    
    ; 计算坐标差值，取整
    mdx := (_mtx - mtx) | 0
    mdy := (_mty - mty) | 0
    ; 计算移动后的坐标
    mtx += mdx
    mty += mdy
    QSeconds := QuerySeconds()
    
    if(mdx || mdy ) {
        MouseMove(mdx, mdy)
        ; ToolTip, %tdx% %_mtx% %mtx% %mdx%
    }
Return

mTick()
{
    SetTimer mTick, 0
}
; 只有开启CapsLockX模式能触发
; #If CapsLockXMode == CM_CapsLockX
; 鼠标运动处理
*a:: mtl := (mtl ? mtl : QPC()), mTick()
*d:: mtr := (mtr ? mtr : QPC()), mTick()
*w:: mtu := (mtu ? mtu : QPC()), mTick()
*s:: mtd := (mtd ? mtd : QPC()), mTick()
*a Up:: mtl := 0, mTick()
*d Up:: mtr := 0, mTick()
*w Up:: mtu := 0, mTick()
*s Up:: mtd := 0, mTick()

; ; 鼠标滚轮处理
; r:: scroll_tu := (scroll_tu ? scroll_tu : QPC()), sTick()
; f:: scroll_td := (scroll_td ? scroll_td : QPC()), sTick()
; r Up:: scroll_tu := 0, sTick()
; f Up:: scroll_td := 0, sTick()

; ; 单格滚动
; !r:: Send {WheelUp}
; !f:: Send {WheelDown}
; !^r:: Send ^{WheelUp}
; !^f:: Send ^{WheelDown}

; ; 缩放
; ^r:: Send ^{WheelUp}
; ^f:: Send ^{WheelDown}

; ; ^r:: scroll_tu := (scroll_tu ? scroll_tu : QPC()), sTick()
; ; ^f:: scroll_td := (scroll_td ? scroll_td : QPC()), sTick()
; ; ^r Up:: scroll_tu := 0, sTick()
; ; ^f Up:: scroll_td := 0, sTick()

; ; 横向滚动
; +r:: scroll_tl := (scroll_tl ? scroll_tl : QPC()), sTick()
; +f:: scroll_tr := (scroll_tr ? scroll_tr : QPC()), sTick()
; +r Up:: scroll_tl := 0, sTick()
; +f Up:: scroll_tr := 0, sTick()

Esc:: ExitApp

