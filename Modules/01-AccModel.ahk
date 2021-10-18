; ========== CapsLockX ==========
; 名称：时基加速模型
; 描述：用来模拟按键和鼠标，计算一个虚拟的光标运动物理模型。
; 版本：v2020.06.27
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版权：Copyright © 2017-2021 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========
; 光标加速度微分对称模型（不要在意这中二的名字hhhh

class AccModel2D
{
    __New(实动函数, 衰减率 := 1, 加速比率 := 1, 纵加速比率 := 0)
    {
        this.动刻 := 0, 
        this.左刻 := 0, this.右刻 := 0
        this.上刻 := 0, this.下刻 := 0
        this.横速 := 0, this.横移 := 0
        this.纵速 := 0, this.纵移 := 0
        this.间隔 := 0
        this.时钟 := ObjBindMethod(this, "_ticker")
        this.实动函数 := 实动函数
        this.横加速比率 := 加速比率
        this.纵加速比率 := 纵加速比率 == 0 ? this.横加速比率 : 纵加速比率
        this.a2 := 1
        this.a3 := 3
        this.aP := 9
        this.衰减率 := 衰减率
        this.最大速度 := 2147483.647
        this.中键间隔 := 0.1 ; 100ms
    }
    _dt(t, 现刻)
    {
        Return t ? (现刻 - t) / this._QPF() : 0
    }
    _ma(_dt)
    {
        a := 0
        a += this.a2 == 0 ? 0 : this.a2 * this._ma2(_dt)
        a += this.a3 == 0 ? 0 : this.a3 * this._ma3(_dt)
        a += this.aP == 0 ? 0 : this.aP * this._maPower(_dt)
        return a
    }
    _ma2(_dt)
    {
        ; x-_dt 二次曲线加速运动模型
        ; 跟现实世界的运动一个感觉，一般都加
        Return this._sign(_dt)
    }
    _ma3(_dt)
    {
        ; x-_dt 三次曲线函数运动模型
        ; 与现实世界不同，
        ; 这个模型会让人感觉鼠标比较“重”
        Return _dt
    }
    _maPower(_dt)
    {
        ; x-_dt 指数曲线运动的简化模型
        ; 这个模型可以满足精确定位需求，也不会感到鼠标“重”
        ; 但是因为跟现实世界的运动曲线不一样，凭直觉比较难判断落点，需要一定练习才能掌握。
        Return this._sign(_dt) * ( Exp(Abs(_dt)) - 1 )
    }
    _sign(x)
    {
        return x == 0 ? 0 : (x > 0 ? 1 : -1)
    }
    _damping(v, a)
    {
        ; 限制最大速度
        maxs := this.最大速度
        v := v < -maxs ? -maxs : v
        v := v > maxs ? maxs : v
        
        ; 摩擦力不阻碍用户意志，加速度存在时不使用摩擦力
        if ((a > 0 And v > 0) Or (a < 0 And v < 0)) {
            Return v
        }
        
        ; 简单粗暴倍数降速
        v *= 1 - this.衰减率
        ; 线性降速
        v -= !this.衰减率 ? 0 : v > 1 ? 1 : (v < -1 ? -1 : 0)
        ; 零点吸附
        v:= Abs(v) < 1 ? 0 : v
        Return v
    }
    _ticker(现刻:=0)
    {
        loop, 2 {
            ; 系统默认时钟频率大概64hz，对于刷
            ; 所以这里套一层 Looper 来提高 FPS，提升操作手感
            this._tickerLooper(现刻)
            ; Sleep, 1
        }
    }
    _tickerLooper(现刻:=0)
    {
        现刻 := 现刻==0 ?  this._QPC():现刻
        ; 计算 dt
        dt := this.动刻 == 0 ? 0 : ((现刻 - this.动刻) / this._QPF())
        this.动刻 := 现刻
        
        ; 计算用户操作总时间
        左时 := this._dt(this.左刻, 现刻), 右时 := this._dt(this.右刻, 现刻)
        上时 := this._dt(this.上刻, 现刻), 下时 := this._dt(this.下刻, 现刻)
        
        ; 同时按下相当于中键（同时也会取消自动）
        if (this.左刻 && this.右刻 && Abs(右时-左时) < this.中键间隔) {
            this.止动()
            return this.实动函数.Call(this._sign(右时-左时), 0, "横中键")
        }
        if (this.上刻 && this.下刻 && Abs(下时-上时) < this.中键间隔) {
            this.止动()
            return this.实动函数.Call(0, this._sign(下时-上时), "纵中键")
        }
        ; 处理移动
        横加速 := this._ma(- 左时 + 右时) * this.横加速比率
        纵加速 := this._ma(- 上时 + 下时) * this.纵加速比率
        this.横速 := this._damping(this.横速 + 横加速 * dt, 横加速)
        this.纵速 := this._damping(this.纵速 + 纵加速 * dt, 纵加速)
        
        ; if (abs(v) >= maxs) {
        ;     this.加速比率 *= 1.1
        ;     TrayTip, CapsLockX, 达到速度上限，加速模型加力百分之十！
        ;     return 0
        ; }
        
        ; 快速启动
        if (!dt) {
            this.启动中 := 1
            this.实动函数.Call(0, 0, "启动")
            this.启动中 := 0

            this.横移 := this._sign(横加速)
            this.纵移 := this._sign(纵加速)
        }
        this.横移 += this.横速 * dt
        this.纵移 += this.纵速 * dt
        横输出 := this.横移 | 0  ; 取整输出
        纵输出 := this.纵移 | 0  ; 取整输出
        this.横移 -= 横输出      ; 收回零头攒起来
        this.纵移 -= 纵输出      ; 收回零头攒起来
        
        ; debug
        msg := dt "`n" 现刻 "`n" this.动刻 "`n" 横加速 "`n" this.横速 "`n" this.横移 "`n" this.横输出
        ; tooltip %msg%
        
        if (横输出 || 纵输出) {
            this.实动函数.Call(横输出, 纵输出, "移动")
        }
        ; 速度归 0，结束定时器
        if ( !this.横速 && !this.纵速 && !(横输出 || 纵输出)) {
            this.止动()
            Return
        }
    }
    始动() {
        this.动刻 := 0
        this.ticker()
        时钟 := this.时钟
        SetTimer % 时钟, % this.间隔
    }
    止动(){
        this.动刻 := 0, this.动中 := 0
        this.左刻 := 0, this.右刻 := 0
        this.上刻 := 0, this.下刻 := 0
        this.横速 := 0, this.横移 := 0
        this.纵速 := 0, this.纵移 := 0
        时钟 := this.时钟
        SetTimer % 时钟, Off
        this.实动函数.Call(0, 0, "止动")
    }
    冲突止动(){
        在动 := this.动刻 != 0
        启动中 := this.启动中
        if(在动 && !启动中){
            this.止动()
        }
    }
    左按(){
        this.左刻 := this.左刻 ? this.左刻 : this._QPC()
        this.始动()
    }
    左放(){
        this.左刻 := 0
    }
    右按(){
        this.右刻 := this.右刻 ? this.右刻 : this._QPC()
        this.始动()
    }
    右放(){
        this.右刻 := 0
    }
    上按(){
        this.上刻 := this.上刻 ? this.上刻 : this._QPC()
        this.始动()
    }
    上放(){
        this.上刻 := 0
    }
    下按(){
        this.下刻 := this.下刻 ? this.下刻 : this._QPC()
        this.始动()
    }
    下放(){
        this.下刻 := 0
    }
    ; 高性能计时器，精度能够达到微秒级，相比之下 A_Tick 的精度大概只有10几ms。
    _QPF()
    {
        DllCall("QueryPerformanceFrequency", "Int64*", QuadPart)
        Return QuadPart
    }
    _QPC()
    {
        DllCall("QueryPerformanceCounter", "Int64*", Counter)
        Return Counter
    }
}
class FPS_Debugger
{
    __New()
    {
        this.interval := 1000
        this.count := 0
        this.timer := ObjBindMethod(this, "Tick")
        timer := this.timer
        SetTimer % timer, % this.interval
    }
    inc()
    {
        this.count := this.count + 1
        ; ToolTip % "FPS:" this.count
    }
    ; In this example, the timer calls this method:
    Tick()
    {
        ToolTip % "FPS:" this.count
        this.count := 0
    }
}
