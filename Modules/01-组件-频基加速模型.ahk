; ========== CapsLockX ==========
; 名称：频基加速度物理模型
; 描述：频基加速度物理模型（适用于FPS各种游戏）
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：0.0.1(20200606)
; 版权：Copyright © 2018-2020 Snowstar Laboratory. All Rights Reserved.
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

global FM_Test := FM_Create()
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
FM_Create1(){
    Return {频率列: []}
}
FM_Down1(this){
    按否 := this.放刻 < this.按刻
    if(!按否){
        旧 := this.按刻
        新 := this.按刻 := FM_QTick()
        时差:= 新 - 旧

    }
}
FM_Up1(this){
    this.放刻 := FM_QTick()
}
FM_Out1(this){

}
FM_Create(){
    Return {左按刻: 0, 右按刻: 0, 上按刻: 0, 下按刻: 0, 左放刻: 0, 右放刻: 0, 上放刻: 0, 下放刻: 0}
    ; Push
}
FM_Ticker(this, 事件 := ""){

    ; 左按否 := this.左放刻 < this.左按刻
    ; 右按否 := this.右放刻 < this.右按刻
    ; 上按否 := this.上放刻 < this.上按刻
    ; 下按否 := this.下放刻 < this.下按刻
    ; ToolTip 左按否 右按否 上按否 下按否
    ToolTip % 事件 this[事件 + "刻"]
    Return 
}

#if

$!j:: FM_Ticker(FM_Test, FM_左 | FM_按)
$!j Up:: FM_Ticker(FM_Test, FM_左 | FM_放)
$!l:: FM_Ticker(FM_Test, FM_右 | FM_按)
$!l Up:: FM_Ticker(FM_Test, FM_右 | FM_放)
$!i:: FM_Ticker(FM_Test, FM_上 | FM_按)
$!i Up:: FM_Ticker(FM_Test, FM_上 | FM_放)
$!k:: FM_Ticker(FM_Test, FM_下 | FM_按)
$!k Up:: FM_Ticker(FM_Test, FM_下 | FM_放)
