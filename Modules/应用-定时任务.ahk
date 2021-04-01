; ========== CapsLockX ==========
; 名称：定时任务
; 描述：打开 CapsLockX 的 Github 页面
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2021.03.26
; ========== CapsLockX ==========

global T_ScheduleTasks_UseTomatoLife := CapsLockX_Config("ScheduleTasks", "UseTomatoLife", 0, "使用番茄报时")
global T_ScheduleTasks_UseTomatoLifeSwitchVirtualDesktop := CapsLockX_Config("ScheduleTasks", "UseTomatoLifeSwitchVirtualDesktop", 1, "使用番茄报时自动切换桌面（休息桌面为1，工作桌面为2）")
GoSub CapsLockX定时任务
Return

番茄状态计算(){
    Return ((Mod((UnixTimeGet() / 60), 30) < 25) ? "工作时间" : "休息时间")
}
番茄报时(force:=0){
    番茄状态 := 番茄状态计算()
    ; 边沿触发过滤器
    static 上次番茄状态 := 番茄状态计算()
    if(上次番茄状态 == 番茄状态 && !force)
        Return
    上次番茄状态 := 番茄状态
    ; 状态动作
    if("工作时间" == 番茄状态){
        SoundPlay % "Data/NoteC_G.mp3" ; 升调
        if(T_ScheduleTasks_UseTomatoLifeSwitchVirtualDesktop)
            Func("SwitchToDesktop").Call(2) ; 切到工作桌面（桌面2）
    }
    if("休息时间" == 番茄状态){
        SoundPlay % "Data/NoteG_C.mp3" ; 降调
        if(T_ScheduleTasks_UseTomatoLifeSwitchVirtualDesktop)
            Func("SwitchToDesktop").Call(1) ; 切到休息桌面（桌面1）
    }
}

UnixTimeGet(){
    ; ref: https://www.autohotkey.com/boards/viewtopic.php?t=17333
    Time := A_NowUTC
    EnvSub, Time, 19700101000000, Seconds
    Return Time
}

CapsLockX定时任务:
    if(T_ScheduleTasks_UseTomatoLife)
        番茄报时()
    间隔 := 60 ; 精度到1分钟
    延时 := 1000 * (间隔 - Mod(UnixTimeGet(), 间隔))
    SetTimer CapsLockX定时任务, %延时%
Return
