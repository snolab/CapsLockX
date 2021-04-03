; ========== CapsLockX ==========
; 名称：定时任务
; 描述：打开 CapsLockX 的 Github 页面
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2021.03.26
; ========== CapsLockX ==========

global T_ScheduleTasks := CapsLockX_Config("ScheduleTasks", "EnableScheduleTasks", 0, "使用定时任务")
global T_ScheduleTasks_UseTomatoLife := CapsLockX_Config("ScheduleTasks", "UseTomatoLife", 1, "使用番茄报时（需要先开启定时任务）")
global T_ScheduleTasks_UseTomatoLifeSwitchVirtualDesktop := CapsLockX_Config("ScheduleTasks", "UseTomatoLifeSwitchVirtualDesktop", 1, "使用番茄报时时，自动切换桌面（休息桌面为1，工作桌面为2）")

if(T_ScheduleTasks){
    高精度时间配置()
}

GoSub CapsLockX定时任务

Return
高精度时间配置(){
    ; global T_ScheduleTasks := CapsLockX_Config("ScheduleTasks", "", 0, "使用定时任务")
    ; MsgBox, 你开启了定时任务，是否现在配置高精度时间？
    ; IfMsgBox, Cancel
    ; return

    global T_ScheduleTasks_UsingHighPerformanceTime := CapsLockX_Config("ScheduleTasks", "T_UsingHighPerformanceTime", "0", "")
    if(T_ScheduleTasks_UsingHighPerformanceTime)
        return
    ToolTip, 正在配置系统高精度时间
    RunWait reg add "HKLM\SYSTEM\CurrentControlSet\Services\W32Time\Config" /v "FrequencyCorrectRate" /t REG_DWORD /d 2 /f
    RunWait reg add "HKLM\SYSTEM\CurrentControlSet\Services\W32Time\Config" /v "UpdateInterval" /t REG_DWORD /d 100 /f
    RunWait reg add "HKLM\SYSTEM\CurrentControlSet\Services\W32Time\Config" /v "MaxPollInterval" /t REG_DWORD /d 6 /f
    RunWait reg add "HKLM\SYSTEM\CurrentControlSet\Services\W32Time\Config" /v "MinPollInterval" /t REG_DWORD /d 6 /f
    RunWait reg add "HKLM\SYSTEM\CurrentControlSet\Services\W32Time\Config" /v "MaxAllowedPhaseOffset" /t REG_DWORD /d 0 /f
    RunWait reg add "HKLM\SYSTEM\CurrentControlSet\Services\W32Time\TimeProviders\NtpClient" /v "SpecialPollInterval" /t REG_DWORD /d 64 /f
    RunWait net stop w32time
    RunWait net start w32time
    CapsLockX_ConfigSet("ScheduleTasks", "T_UsingHighPerformanceTime", "1", "")
    ToolTip
}
番茄状态计算(){
    Return ((Mod((UnixTimeGet() / 60000), 30) < 25) ? "工作时间" : "休息时间")
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
    t := A_NowUTC
    EnvSub, t, 19700101000000, Seconds
    Return t*1000+A_MSec
}

CapsLockX定时任务:
    if(T_ScheduleTasks_UseTomatoLife)
        番茄报时()
    间隔 := 60000 ; 间隔为1分钟，精度到毫秒级
    延时 := (间隔 - Mod(UnixTimeGet(), 间隔))
    ; ToolTip, % 延时
    SetTimer CapsLockX定时任务, %延时%
Return