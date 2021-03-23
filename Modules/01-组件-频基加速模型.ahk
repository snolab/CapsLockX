; ========== CapsLockX ==========
; 名称：频基加速度物理模型
; 描述：频基加速度物理模型
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：0.0.1(20200606)
; 版权：Copyright © 2018-2020 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========
;

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
FM_Create(){
    Return {左按刻: 0, 右按刻: 0, 上按刻: 0, 下按刻: 0, 左放刻: 0, 右放刻: 0, 上放刻: 0, 下放刻: 0}
}
FM_Ticker(this, key := 0){
    左按否 := this.左放刻 < this.左按刻
    右按否 := this.右放刻 < this.右按刻
    上按否 := this.上放刻 < this.上按刻
    下按否 := this.下放刻 < this.下按刻
    Return 
}

#if

$!j:: FM_Test.左按刻 += FM_Test.左区间 := FM_QTick() - FM_Test.左按刻, FM_Ticker(FM_Test)
$!j Up:: FM_Test.左放刻 += FM_Test.左区间 := FM_QTick() - FM_Test.左放刻, FM_Ticker(FM_Test)
$!l:: FM_Test.右按刻 += FM_Test.右区间 := FM_QTick() - FM_Test.右按刻, FM_Ticker(FM_Test)
$!l Up:: FM_Test.右放刻 += FM_Test.右区间 := FM_QTick() - FM_Test.右放刻, FM_Ticker(FM_Test)
$!i:: FM_Test.上按刻 += FM_Test.上区间 := FM_QTick() - FM_Test.上按刻, FM_Ticker(FM_Test)
$!i Up:: FM_Test.上放刻 += FM_Test.上区间 := FM_QTick() - FM_Test.上放刻, FM_Ticker(FM_Test)
$!k:: FM_Test.下按刻 += FM_Test.下区间 := FM_QTick() - FM_Test.下按刻, FM_Ticker(FM_Test)
$!k Up:: FM_Test.下放刻 += FM_Test.下区间 := FM_QTick() - FM_Test.下放刻, FM_Ticker(FM_Test)
