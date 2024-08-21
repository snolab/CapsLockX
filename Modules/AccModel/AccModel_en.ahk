; ========== CapsLockX ==========
; Name: Time-Based Acceleration Model
; Description: Used to simulate keystrokes and mouse movements, calculating a virtual cursor motion physics model.
; Version: v2020.06.27
; Author: snomiao
; Contact: snomiao@gmail.com
; Support: https://github.com/snomiao/CapsLockX
; Copyright: Copyright © 2017-2024 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========
; Cursor acceleration differential symmetry model (don’t mind the quirky name, haha)

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
    __New(actionFunc, decayRate := 1, accelRate := 1, vertAccelRate := 0)
    {
        this.moveTick := 0, this.moving := 0
        this.leftTick := 0, this.rightTick := 0
        this.upTick := 0, this.downTick := 0
        this.horizontalSpeed := 0, this.horizontalMove := 0
        this.verticalSpeed := 0, this.verticalMove := 0
        this.leftHoldKey := "", this.rightHoldKey := ""
        this.upHoldKey := "", this.downHoldKey := ""
        this.interval := 0
        this.timerMethod := ObjBindMethod(this, "_ticker")
        this.actionFunc := actionFunc
        this.horizontalAccelRate := accelRate
        this.verticalAccelRate := vertAccelRate == 0 ? this.horizontalAccelRate : vertAccelRate
        this.decayRate := decayRate
        this.middleKeyInterval := 0.1 ; 100ms
    }

    _dt(t, currentTick)
    {
        Return t ? (currentTick - t) / this._QPF() : 0
    }

    _ma(_dt)
    {
        sgn := this._sign(_dt)
        abs := Abs(_dt)
        ; 1x exponential function + 1x 4th-degree polynomial
        a := 0
        a += 1 * sgn * ( Exp(abs) - 1 )
        a += 3 * sgn
        a += 4 * sgn * abs
        a += 9 * sgn * abs * abs
        a += 16 * sgn * abs * abs * abs
        return a
    }

    _sign(x)
    {
        return x == 0 ? : (x > 0 ? 1 : -1)
    }

    _damping(v, a, dt)
    {
        ; Limit maximum speed
        if (this.maxSpeed) {
            maxs := this.maxSpeed
            v := v < -maxs ? -maxs : v
            v := v > maxs ? maxs : v
            if (abs(v)==maxs) {
                ; tooltip Warning: reached maximum speed
            }
        }

        ; Friction does not hinder user intent; no friction is used if acceleration is directional
        if (a * v > 0) {
            Return v
        }

        ; Crude exponential dampening
        v *= Exp(-dt*20)
        v -= this._sign(v) * dt

        ; v *= 1 - this.decayRate
        ; Linear speed reduction
        ; v -= !this.decayRate ? 0 : v > 1 ? 1 : (v < -1 ? -1 : 0)
        ; Zero-point attraction
        v:= Abs(v) < 1 ? 0 : v
        Return v
    }

    _ticker()
    {
        re := this._tickerLooper()
    }

    _tickerLooper()
    {
        ; Calculate total user operation time
        currentTick := this._QPC()
        ; Compute dt
        dt := this.moveTick == 0 ? 0 : ((currentTick - this.moveTick) / this._QPF())
        this.moveTick := currentTick
        leftTime := this._dt(this.leftTick, currentTick), rightTime := this._dt(this.rightTick, currentTick)
        upTime := this._dt(this.upTick, currentTick), downTime := this._dt(this.downTick, currentTick)
        
        ; Simultaneous key presses act as a middle key press (also cancels auto)
        if (this.leftTick && this.rightTick && Abs(rightTime-leftTime) < this.middleKeyInterval) {
            this.actionFunc.Call(this._sign(rightTime-leftTime), 0, "horizontal middle key")
            this.StopMovement()
            return 1
        }
        if (this.upTick && this.downTick && Abs(downTime-upTime) < this.middleKeyInterval) {
            this.actionFunc.Call(0, this._sign(downTime-upTime), "vertical middle key")
            this.StopMovement()
            return 1
        }
        
        ; Handle movement
        horizontalAccel := this._ma(rightTime-leftTime) * this.horizontalAccelRate
        verticalAccel := this._ma(downTime-upTime) * this.verticalAccelRate
        this.horizontalSpeed := this._ADD(this.horizontalSpeed, horizontalAccel * dt)
        this.verticalSpeed := this._ADD(this.verticalSpeed, verticalAccel * dt)
        this.horizontalSpeed := this._damping(this.horizontalSpeed, horizontalAccel, dt)
        this.verticalSpeed := this._damping(this.verticalSpeed, verticalAccel, dt)

        ; perf_timing(1)
        ; Fast start
        if (!dt) {
            this.starting := 1
            this.actionFunc.Call(0, 0, "start")
            this.starting := 0

            this.horizontalMove := this._sign(horizontalAccel)
            this.verticalMove := this._sign(verticalAccel)
        }
        this.horizontalMove := this._ADD(this.horizontalMove, this.horizontalSpeed * dt)
        this.verticalMove := this._ADD(this.verticalMove, this.verticalSpeed * dt)
        horizontalOutput := this.horizontalMove | 0  ; Round off the output
        verticalOutput := this.verticalMove | 0  ; Round off the output
        this.horizontalMove -= horizontalOutput ; Accumulate the remainder
        this.verticalMove -= verticalOutput ; Accumulate the remainder

        ; Debug
        ; msg := dt "`n" currentTick "`n" this.moveTick "`n" horizontalAccel "`n" this.horizontalSpeed "`n" this.horizontalMove "`n" this.horizontalOutput
        ; tooltip %msg%

        if (horizontalOutput || verticalOutput) {
            ; tooltip %dt% %horizontalOutput% %verticalOutput% %horizontalAccel% %verticalAccel%
            this.actionFunc.Call(horizontalOutput, verticalOutput, "move")
        }

        ; If requested key is released, stop the timer
        if (this.leftHoldKey && !GetKeyState(this.leftHoldKey, "P")) {
            this.leftHoldKey := "", this.ReleaseLeftKey()
        }
        if (this.rightHoldKey && !GetKeyState(this.rightHoldKey, "P")) {
            this.rightHoldKey := "", this.ReleaseRightKey()
        }
        if (this.upHoldKey && !GetKeyState(this.upHoldKey, "P")) {
            this.upHoldKey := "", this.ReleaseUpKey()
        }
        if (this.downHoldKey && !GetKeyState(this.downHoldKey, "P")) {
            this.downHoldKey := "", this.ReleaseDownKey()
        }

        ; If speed is zero, stop the timer
        if ( !this.horizontalSpeed && !this.verticalSpeed && !(horizontalOutput || verticalOutput)) {
            this.StopMovement()
            return 0
        }
        return 1
    }

    StartMovement() {
        this.moveTick := 0
        this._ticker()
        timerMethod := this.timerMethod
        SetTimer % timerMethod, % 0
    }
    
    StopMovement(){
        timerMethod := this.timerMethod
        SetTimer % timerMethod, Off
        if (this.moveTick != 0) {
            this.moveTick := 0
            this.actionFunc.Call(0, 0, "stop")
        }
        this.moveTick := 0, this.moving := 0
        this.leftTick := 0, this.rightTick := 0
        this.upTick := 0, this.downTick := 0
        this.horizontalSpeed := 0, this.horizontalMove := 0
        this.verticalSpeed := 0, this.verticalMove := 0
        this.leftHoldKey := "", this.rightHoldKey := ""
        this.upHoldKey := "", this.downHoldKey := ""
    }

    PressLeftKey(leftHoldKey:=""){
        if (this.leftHoldKey) {
            return
        }
        this.leftHoldKey := leftHoldKey
        this.leftTick := this.leftTick ? this.leftTick : this._QPC()
        this.StartMovement()
    }

    ReleaseLeftKey(){
        this.leftTick := 0
    }

    PressRightKey(rightHoldKey:=""){
        if (this.rightHoldKey) {
            return
        }
        this.rightHoldKey := rightHoldKey
        this.rightTick := this.rightTick ? this.rightTick : this._QPC()
        this.StartMovement()
    }

    ReleaseRightKey(){
        this.rightTick := 0
    }

    PressUpKey(upHoldKey:=""){
        if (this.upHoldKey) {
            return
        }
        this.upHoldKey := upHoldKey
        this.upTick := this.upTick ? this.upTick : this._QPC()
        this.StartMovement()
    }

    ReleaseUpKey(){
        this.upTick := 0
    }

    PressDownKey(downHoldKey:=""){
        if (this.downHoldKey) {
            return
        }
        this.downHoldKey := downHoldKey
        this.downTick := this.downTick ? this.downTick : this._QPC()
        this.StartMovement()
    }

    ReleaseDownKey(){
        this.downTick := 0
    }

    ; High-performance timer with microsecond-level precision, while A_Tick has around 10ms precision.
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

    _ADD(acc, x){
        c := acc + x
        if (Abs(acc) > (2147483647/2) this._sign(acc) != this._sign(c)) {
           return acc
        }
        return c
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
