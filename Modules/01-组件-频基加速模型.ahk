; ========== CapsLockX ==========
; 名称：频基加速度物理模型
; 描述：频基加速度物理模型（适用于FPS各种游戏）
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：0.0.1(20200606)
; 版权：Copyright © 2017-2021 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========
;
; 时序捕获：（事件捕获  按下：t1  放开：t2  按下：t3）
; 时差：依次 时序捕获 （差 t3 t1）
; 频率：除 1 时差
; 基频：1
; 频率列：列 基频
; 入频：压入 频率列 频率
; 消除：
; 平均频率：平均 频率列
; 输出频率：最大值 1 （平均 频率列）
; 加速度：乘 5 输出频率
; 

; 左入 右入 左出 右出
; 插入 压入 移走 弹出

if (!CapsLockX){
    MsgBox, % "本模块只在 CapsLockX 下工作"
    ExitApp
}

global FM_Test := FM_Create4("myMouseMove")
global FM_Test1 := FM_Create1("myMouseMove1" )

ToolTip, testing

; msgbox % FM_Vec4Mul1([1,2,3,4],5)[2]

Return

FM_Vec4Mul1(vec, val){
    return [vec[1]*val,vec[2]*val,vec[3]*val, vec[4]*val]
}
FM_Vec4Plus(vec1, vec2){
    return [vec1[1]+vec2[1],vec1[2]+vec2[2],vec1[3]+vec2[3], vec1[4]+vec2[4]]
}
FM_Vec4Minus(vec1, vec2){
    return [vec1[1]-vec2[1],vec1[2]-vec2[2],vec1[3]-vec2[3], vec1[4]-vec2[4]]
}
FM_Vec4Mul(vec1, vec2){
    return [vec1[1]*vec2[1],vec1[2]*vec2[2],vec1[3]*vec2[3], vec1[4]*vec2[4]]
}
FM_Vec4Dot(vec1, vec2){
    return [vec1[1]*vec2[1]+vec1[2]*vec2[2]+vec1[3]*vec2[3]+vec1[4]*vec2[4]]
}
FM_Vec4Lt(vec1, vec2){
    return [vec1[1]<vec2[1], vec1[2]<vec2[2], vec1[3]<vec2[3], vec1[4]<vec2[4]]
}
FM_Vec4BitOr(vec, bit){
    return [vec[1]|bit, vec[2]|bit, vec[3]|bit, vec[4]|bit]
}
FM_Vec4BitNot(vec){
    return [!vec[1], !vec[2], !vec[3], !vec[4]]
}
FM_Vec4And(vec){
    return vec[1] && vec[2] && vec[3] && vec[4]
}
FM_Vec4Or(vec){
    return vec[1] || vec[2] || vec[3] || vec[4]
}
FM_Vec4MulRate(vec, r){
    return FM_Vec4Plus(FM_Vec4Mul1(vec, r), FM_Vec4Mul1(vec,1 - r))
}
FM_Vec4Zeros(){
    return [0,0,0,0]
}
FM_Vec4Ones(){
    return [1,1,1,1]
}
FM_Vec4Print(vec){
    return "["vec[1]","vec[2]","vec[3]","vec[4]"]"
}
; 高性能计时器，精度能够达到微秒级，相比之下 A_Tick 的精度大概只有10几ms。
FM_QPF(){
    DllCall("QueryPerformanceFrequency", "Int64*", QuadPart)f
    Return QuadPart
}
FM_QPC(){
    DllCall("QueryPerformanceCounter", "Int64*", Counter)
    Return Counter
}
FM_QTick(){
    Return FM_QPC() / TM_QPF()
}
FM_Ticker(this){
    旧 := this.更新刻, 新 := this.更新刻 := FM_QTick(), 时差 := 新 - 旧
    if(!时差)
        return [0, 0]
    this.位移 := FM_Vec4Plus(this.位移, FM_Vec4Mul1(this.速度, 时差))
    离散位移 := FM_Vec4BitOr(this.位移, 0)
    this.位移 := FM_Vec4Minus(this.位移, 离散位移)
    按否 := FM_Vec4Lt( this.放刻, this.按刻)
    ; this.速度 := FM_Vec4Mul(this.速度, FM_Vec4Mul1(按否,(1 - (0.1 * 时差 * 100)))) ; 阻力

    ; this.频率 := this.频率 * 时差 * 0.01 + 1 * (1-时差 * 0.01) ; 频率阻力
    _=_
    ToolTip, % 时差 _ FM_Vec4Print(按否) _ FM_Vec4Print(this.速度) _ FM_Vec4Print(this.位移) _ FM_Vec4Print(离散位移) _ FM_Vec4Print(this.频率)
    if(this.加速 < 0 )
        this.加速 := 0
    if(this.速度 | 0 == 0){
        this.更新刻 := ""
        timerLabel := this.timerLabel
        SetTimer, %timerLabel%, Off
    }
    out := [离散位移[2]-离散位移[1], 离散位移[4]-离散位移[3]]
    return out
}

FM_Ticker1(this){
    旧 := this.更新刻, 新 := this.更新刻 := FM_QTick()
    按否 := this.放刻 < this.按刻
    时差 := 新 - 旧
    if(!时差)
        return 0
    this.位移 += this.速度 * 时差
    离散位移 := this.位移 | 0
    ; MouseMove, -离散位移, 0, 0, R
    this.位移 -= 离散位移r

    if(!按否)
        this.速度 := this.速度 * (1 - (0.1 * 时差 * 100)) ; 阻力
    this.频率 := this.频率 * 时差 * 0.01 + 1 * (1-时差 * 0.01) ; 频率阻力
    if(this.加速 < 0 )
        this.加速 := 0
    if(this.速度 | 0 == 0){
        this.更新刻 := ""
        timerLabel := this.timerLabel
        SetTimer, %timerLabel%, Off
    }
    return 离散位移
}

FM_按下1(this){
    if(this.放刻 < this.按刻)
        return
    旧 := this.按刻, 新 := this.按刻 := FM_QTick()
    频率 := 1 / (新 - 旧)
    this.频率 := this.频率 * 0.9 + 频率 * 0.1
    if(!this.频率 || this.频率 <1)
        this.频率 := 1
    this.速度 += this.频率 ** 1.5 * 100
    timerLabel := this.timerLabel
    SetTimer, %timerLabel%, 1
}
FM_弹起1(this){
    this.放刻 := FM_QTick()
}
FM_按下(this,dir){
    if(this.放刻[dir] < this.按刻[dir])
        return
    旧 := this.按刻[dir], 新 := this.按刻[dir] := FM_QTick()
    频率 := (1 / (新 - 旧)) || 1
    this.频率[dir] := this.频率[dir] * 0.8 + 频率[dir] * 0.2
    if(!this.频率[dir] || this.频率[dir] <1)
        this.频率[dir] := 1
    this.速度[dir] += this.频率[dir] ** 1.5 * 200
    timerLabel := this.timerLabel
    SetTimer, %timerLabel%, 1
}
FM_弹起(this, dir){
    this.放刻[dir] := FM_QTick()
}
FM_Create1(timerLabel){
    Return {按刻: 0, 放刻: 0, 频率: 0, 加速: 0, 速度:0, 位移: 0, timerLabel: timerLabel}
}
FM_Create4(timerLabel){
    o := {timerLabel: timerLabel}
    o.按刻:= FM_Vec4Zeros(), o.放刻:= FM_Vec4Zeros(),
    o.频率:= FM_Vec4Zeros(), o.加速:= FM_Vec4Zeros(),
    o.速度:= FM_Vec4Zeros(), o.位移:= FM_Vec4Zeros(),
    return o
}
#if

myMouseMove1:
    vec := FM_Ticker1(FM_Test1)
    myMouseMove1(vec)
return
myMouseMove1(x){
    MouseMove, %x%,0,0,R
}

myMouseMove:
    vec := FM_Ticker(FM_Test)
    myMouseMove(vec)
return
myMouseMove(vec){
    x := vec[1], y := vec[2]
    MouseMove, %x%, %y%, 0, R
}
; FM_Ticker:
;     FM_Ticker(FM_Test)
; Return

; $!h:: FM_按下1(FM_Test1)
; $!h Up:: FM_弹起1(FM_Test1)
; $!j:: FM_按下(FM_Test, 1) ; FM_左)
; $!j Up:: FM_弹起(FM_Test, 1) ; FM_左)
; $!l:: FM_按下(FM_Test, 2) ; FM_右)
; $!l Up:: FM_弹起(FM_Test, 2) ; FM_右)
; $!i:: FM_按下(FM_Test, 3) ; FM_上)
; $!i Up:: FM_弹起(FM_Test, 3) ; FM_上)
; $!k:: FM_按下(FM_Test, 4) ; FM_下)
; $!k Up:: FM_弹起(FM_Test, 4) ; FM_下)
