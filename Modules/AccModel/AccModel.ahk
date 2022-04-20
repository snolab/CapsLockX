; ========== CapsLockX ==========
; 名称：时基加速模型
; 描述：用来模拟按键和鼠标，计算一个虚拟的光标运动物理模型。
; 版本：v2020.06.27
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版权：Copyright © 2017-2022 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========
; 光标加速度微分对称模型（不要在意这中二的名字hhhh

perf_now()
{
    DllCall("QueryPerformanceCounter", "Int64*", Counter)
    DllCall("QueryPerformanceFrequency", "Int64*", QuadPart)
    return Counter/QuadPart
}
perf_timing(showq := 1)
{
    static last := perf_now()
    now := perf_now()
    d := now - last
    static d99 := 0
    static dl := 0
    d99 := d99*99/100 + d*1/100
    last := now
    fps := 1/d99
    if (showq) {
        tooltip perf %fps% %d% %d99% %dl%
    }
    dl := d
}
class AccModel2D
{
    __New(实动函数, 衰减率 := 1, 加速比率 := 1, 纵加速比率 := 0)
    {
        this.动刻 := 0, this.动中 := 0
        this.左刻 := 0, this.右刻 := 0
        this.上刻 := 0, this.下刻 := 0
        this.横速 := 0, this.横移 := 0
        this.纵速 := 0, this.纵移 := 0
        this.左等键 := "", this.右等键 := ""
        this.上等键 := "", this.下等键 := ""
        this.间隔 := 0
        this.时钟 := ObjBindMethod(this, "_ticker")
        this.实动函数 := 实动函数
        this.横加速比率 := 加速比率
        this.纵加速比率 := 纵加速比率 == 0 ? this.横加速比率 : 纵加速比率
        this.衰减率 := 衰减率
        this.中键间隔 := 0.1 ; 100ms
    }
    _dt(t, 现刻)
    {
        Return t ? (现刻 - t) / this._QPF() : 0
    }
    _ma(_dt)
    {
        sgn := this._sign(_dt)
        abs := Abs(_dt)
        ; 1x 指数函数 + 1x 4次函数
        a := 0
        a += 1 * sgn * ( Exp(abs) - 1 )
        a += 1 * sgn
        a += 4 * sgn * abs
        a += 9 * sgn * abs * abs
        a += 16 * sgn * abs * abs * abs
        return a
    }
    _sign(x)
    {
        return x == 0 ? 0 : (x > 0 ? 1 : -1)
    }
    _damping(v, a, dt)
    {
        ; 限制最大速度
        if (this.最大速度) {
            maxs := this.最大速度
            v := v < -maxs ? -maxs : v
            v := v > maxs ? maxs : v
            if (abs(v)==maxs) {
                ; tooltip 警告：达到最大速度
            }
        }
        ; 摩擦力不阻碍用户意志，加速度同向时不使用摩擦力
        if (a * v > 0) {
            Return v
        }
        
        ; 简单粗暴倍数降速
        v *= Exp(-dt*20)
        v -= this._sign(v) * dt
        ; v *= 1 - this.衰减率
        ; 线性降速
        ; v -= !this.衰减率 ? 0 : v > 1 ? 1 : (v < -1 ? -1 : 0)
        ; 零点吸附
        v:= Abs(v) < 1 ? 0 : v
        Return v
    }
    _ticker()
    {
        re := this._tickerLooper()
    }
    _tickerLooper()
    {
        ; 用户操作总时间计算
        现刻 := this._QPC()
        ; dt 计算
        dt := this.动刻 == 0 ? 0 : ((现刻 - this.动刻) / this._QPF())
        this.动刻 := 现刻
        左时 := this._dt(this.左刻, 现刻), 右时 := this._dt(this.右刻, 现刻)
        上时 := this._dt(this.上刻, 现刻), 下时 := this._dt(this.下刻, 现刻)
        ; 同时按下相当于中键（同时也会取消自动）
        if (this.左刻 && this.右刻 && Abs(右时-左时) < this.中键间隔) {
            this.实动函数.Call(this._sign(右时-左时), 0, "横中键")
            this.止动()
            return 1
        }
        if (this.上刻 && this.下刻 && Abs(下时-上时) < this.中键间隔) {
            this.实动函数.Call(0, this._sign(下时-上时), "纵中键")
            this.止动()
            return 1
        }
        ; 处理移动
        横加速 := this._ma(右时-左时) * this.横加速比率
        纵加速 := this._ma(下时-上时) * this.纵加速比率
        this.横速 += 横加速 * dt
        this.纵速 += 纵加速 * dt
        this.横速 := this._damping(this.横速, 横加速, dt)
        this.纵速 := this._damping(this.纵速, 纵加速, dt)
        
        ; perf_timing(1)
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
        ; msg := dt "`n" 现刻 "`n" this.动刻 "`n" 横加速 "`n" this.横速 "`n" this.横移 "`n" this.横输出
        ; tooltip %msg%
        
        if (横输出 || 纵输出) {
            ; tooltip %dt% %横输出% %纵输出% %横加速% %纵加速%
            this.实动函数.Call(横输出, 纵输出, "移动")
        }
        ; 要求的键弹起，结束定时器
        if (this.左等键 && !GetKeyState(this.左等键, "P")) {
            this.左等键 := "", this.左放()
        }
        if (this.右等键 && !GetKeyState(this.右等键, "P")) {
            this.右等键 := "", this.右放()
        }
        if (this.上等键 && !GetKeyState(this.上等键, "P")) {
            this.上等键 := "", this.上放()
        }
        if (this.下等键 && !GetKeyState(this.下等键, "P")) {
            this.下等键 := "", this.下放()
        }
        ; 速度归 0，结束定时器
        if ( !this.横速 && !this.纵速 && !(横输出 || 纵输出)) {
            this.止动()
            return 0
        }
        return 1
    }
    始动() {
        this.动刻 := 0
        this._ticker()
        时钟 := this.时钟
        SetTimer % 时钟, % 0
    }
    止动(){
        时钟 := this.时钟
        SetTimer % 时钟, Off
        if (this.动刻 != 0) {
            this.动刻 := 0
            this.实动函数.Call(0, 0, "止动")
        }
        this.动刻 := 0, this.动中 := 0
        this.左刻 := 0, this.右刻 := 0
        this.上刻 := 0, this.下刻 := 0
        this.横速 := 0, this.横移 := 0
        this.纵速 := 0, this.纵移 := 0
        this.左等键 := "", this.右等键 := ""
        this.上等键 := "", this.下等键 := ""
    }
    左按(左等键:=""){
        if (this.左等键) {
            return
        }
        this.左等键 := 左等键
        this.左刻 := this.左刻 ? this.左刻 : this._QPC()
        this.始动()
    }
    左放(){
        this.左刻 := 0
    }
    右按(右等键:=""){
        if (this.右等键) {
            return
        }
        this.右等键 := 右等键
        this.右刻 := this.右刻 ? this.右刻 : this._QPC()
        this.始动()
    }
    右放(){
        this.右刻 := 0
    }
    上按(上等键:=""){
        if (this.上等键) {
            return
        }
        this.上等键 := 上等键
        this.上刻 := this.上刻 ? this.上刻 : this._QPC()
        this.始动()
    }
    上放(){
        this.上刻 := 0
    }
    下按(下等键:=""){
        if (this.下等键) {
            return
        }
        this.下等键 := 下等键
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
    _QPS()
    {
        ; static _QPF
        return this._QPC() / this._QPF()
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
