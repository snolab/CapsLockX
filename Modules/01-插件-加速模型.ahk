; ========== CapsLockX ==========
; 名称：操作加速度物理模型
; 描述：加速度微分对称模型（不要在意这中二的名字hhhh
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：0.0.1(20200606)
; 版权：Copyright © 2018-2020 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========
;

If(!CapsLockX){
    MsgBox, % "本模块只在 CapsLockX 下工作"
    ExitApp
}


; 高性能计时器，精度能够达打微秒级，相比之下 A_Tick 的精度大概只有10几ms。
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
    ; 二次函数运动模型
    ; Return ma2(t)  ; * TMouse_DPIRatio
    
    ; 三次函数运动模型
    ; Return ma3(t)
    
    ; 指数函数运动模型
    Return maPower(t)
}
ma2(t){
    ; x-t 二次曲线加速运动模型
    ; 跟现实世界的运动一个感觉
    If(0 == t)
        Return 0
    If(t > 0)
        Return 1
    Else
        Return -1
}

ma3(t){
    ; x-t 三次曲线函数运动模型
    ; 与现实世界不同，
    ; 这个模型会让人感觉鼠标比较“重”
    ;
    If(0 == t)
        Return 0
    If(t > 0)
        Return 1 + t * 6
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
    Return (t > 0 ? 1 : -1) * ( 1 + ( Exp(Abs(t)) - 1 ) * 2)
}

; 时间计算
dt(t, tNow){
    Return t ? (tNow - t) / QPF() : 0
}

Friction(v, a){ ; 摩擦力
    ; 限制最大速度
    maxSpeed := 1000000
    If(v < -maxSpeed)
        v := -maxSpeed
    If(v > maxSpeed)
        v := maxSpeed
    
    ; 摩擦力不阻碍用户意志
    If((a > 0 And v > 0) Or (a < 0 And v < 0)){
        Return v
    }
    
    ; ; 刹车
    ; If((a < 0 And v > 0) Or (a > 0 And v < 0)){
    ; Return 0
    ; }
    
    ; 简单粗暴倍数降速
    v *= 0.9
    ; 线性
    If (v > 1)
        v -= 1
    If (v < -1)
        v += 1
    if (Abs(v)<=0.01)
        v:=0
    Return v
}
Return