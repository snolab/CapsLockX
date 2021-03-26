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

global FM_Test := {timer: "方向"}

ToolTip, testing

Return

; 高性能计时器，精度能够达到微秒级，相比之下 A_Tick 的精度大概只有10几ms。
FM_QPF(){
    DllCall("QueryPerformanceFrequency", "Int64*", QuadPart)
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
    旧 := this.更新刻, 新 := this.更新刻 := FM_QTick()
    时差 := 新 - 旧
    this.速度 += this.加速 * 时差
    this.位移 += this.速度 * 时差
    离散位移 += this.位移 | 0
    this.位移 -= 离散位移
    ToolTip, % 离散位移

    if((加速 / 10) | 0 == 0){
        TIMER:= this.timer
        SetTimer, %TIMER%, Off
    }
}

FM_Acc(this, 加速度){
    this.加速度 := 加速度
}
FM_Down(this){
    旧 := this.按刻, 新 := this.按刻 := FM_QTick()
    频率 := 1 / (新 - 旧)
    this.频率[0] ||= 1
    TIMER:= this.timer
    SetTimer, %TIMER%, 1
}

FM_DownRaw(this){
    按否 := this.放刻 < this.按刻
    if(!按否)
        FM_Down(this)
}
FM_Up(this){
    this.放刻 := FM_QTick()
}
FM_Out1(this){

}
FM_Create1(){
    Return {按刻: 0, 放刻: 0, 频率列: [1,1,1,1]}
}
FM_Create4(){
    Return {左: FM_Create1(), 右: FM_Create1(), 上: FM_Create1(), 下: FM_Create1()}
}
#if

方向:
    FM_Ticker(FM_Test)
Return

$!h:: FM_DownRaw(FM_Test)
$!h Up:: FM_Up(FM_Test)
; $!j:: FM_Ticker(FM_Test, FM_左 | FM_按)
; $!j Up:: FM_Ticker(FM_Test, FM_左 | FM_放)
; $!l:: FM_Ticker(FM_Test, FM_右 | FM_按)
; $!l Up:: FM_Ticker(FM_Test, FM_右 | FM_放)
; $!i:: FM_Ticker(FM_Test, FM_上 | FM_按)
; $!i Up:: FM_Ticker(FM_Test, FM_上 | FM_放)
; $!k:: FM_Ticker(FM_Test, FM_下 | FM_按)
; $!k Up:: FM_Ticker(FM_Test, FM_下 | FM_放)
