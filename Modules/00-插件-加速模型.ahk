; @CapslockX    v1
; @name         加速模型
; @author       snomiao@gmail.com
; 
; 
Return
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
    ; 二次函数运动模型
    ; Return ma2(t) * TMouse_DPIRatio
    
    ; 三次函数运动模型
    ; Return ma3(t) * TMouse_DPIRatio
    
    ; 指数函数运动模型
    Return maPower(t) * TMouse_DPIRatio
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
    If(t > 0)
        Return 1 +( Exp( t) - 1 ) * 8
    Else
        Return -1 -( Exp(-t) - 1 ) * 8
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