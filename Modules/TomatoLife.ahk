; ========== CapsLockX ==========
; 名称：番茄时钟
; 描述：番茄时钟
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2021.03.26
; ========== CapsLockX ==========

global T_TomatoLife := CLX_Config("TomatoLife", "Enable", 0, t("使用番茄时钟（默认禁用，改为 1 开启）"))
global T_TomatoLife_NoticeOnLaunch := CLX_Config("TomatoLife", "NoticeOnLaunch", 0, t("启动时报告番茄状态"))
global T_TomatoLife_UseTomatoLifeSound := CLX_Config("TomatoLife", "UseTomatoLifeSound", 1, t("使用番茄报时（00分和30分播放工作铃声，每小时的25分和55分播放休息铃声）（需要先开启番茄时钟）"))
global T_TomatoLife_UseTomatoLifeSwitchVirtualDesktop := CLX_Config("TomatoLife", "UseTomatoLifeSwitchVirtualDesktop", 1, t("使用番茄报时时，自动切换桌面（休息为桌面1，工作为桌面2）"))

if (T_TomatoLife) {
    高精度时间配置()
    GoSub CLX_番茄时钟定时任务
    ; [有一个难以复现的 bug・Issue #17・snolab/CapsLockX]( https://github.com/snolab/CapsLockX/issues/17 )
    ; 番茄报时()
}

Return

^!i:: 番茄报时()

高精度时间配置(){
    ; global T_TomatoLife := CLX_Config("TomatoLife", "", 0, "使用定时任务")
    ; MsgBox, 你开启了定时任务，是否现在配置高精度时间？
    ; IfMsgBox, Cancel
    ; return

    global T_TomatoLife_UsingHighPerformanceTime := CLX_Config("TomatoLife", "T_UsingHighPerformanceTime", "0", t("已经配置过高精度时间的Flag"))
    if (T_TomatoLife_UsingHighPerformanceTime)
        return
    ToolTip, 番茄时钟开启，正在为您配置系统高精度时间
    RunWait reg add "HKLM\SYSTEM\CurrentControlSet\Services\W32Time\Config" /v "FrequencyCorrectRate" /t REG_DWORD /d 2 /f, , Hide
    RunWait reg add "HKLM\SYSTEM\CurrentControlSet\Services\W32Time\Config" /v "UpdateInterval" /t REG_DWORD /d 100 /f, , Hide
    RunWait reg add "HKLM\SYSTEM\CurrentControlSet\Services\W32Time\Config" /v "MaxPollInterval" /t REG_DWORD /d 6 /f, , Hide
    RunWait reg add "HKLM\SYSTEM\CurrentControlSet\Services\W32Time\Config" /v "MinPollInterval" /t REG_DWORD /d 6 /f, , Hide
    RunWait reg add "HKLM\SYSTEM\CurrentControlSet\Services\W32Time\Config" /v "MaxAllowedPhaseOffset" /t REG_DWORD /d 0 /f, , Hide
    RunWait reg add "HKLM\SYSTEM\CurrentControlSet\Services\W32Time\TimeProviders\NtpClient" /v "SpecialPollInterval" /t REG_DWORD /d 64 /f, , Hide
    RunWait net stop w32time, , Hide
    RunWait net start w32time, , Hide
    CLX_ConfigSet("TomatoLife", "T_UsingHighPerformanceTime", "1", "")
    ToolTip
}

番茄状态计算(){
    Return ((Mod((UnixTimeGet() / 60000)+30000, 30) < 25) ? "工作时间" : "休息时间")
}

番茄报时(force:=0){
    ; CapsLockX 暂停时，番茄状态也暂停
    if (CLX_Paused)
        Return
 
    ; tooltip skip if sleep
    static SLEEPING_FLAG_CLEAN := 0
    if(!SLEEPING_FLAG_CLEAN) {
        ; 启动时重置标记文件
        FileDelete %TEMP%/SLEEPING_FLAG
        SLEEPING_FLAG_CLEAN := 1
    } else {
        FileRead SLEEPING_FLAG, %TEMP%/SLEEPING_FLAG
        if (SLEEPING_FLAG) {
            Return
        }
    }
    ; skip if afk
    isAfk := A_TimeIdlePhysical > 5 * 60 * 1000 ; 5min
    if(isAfk){
        Return
    }
    番茄状态 := 番茄状态计算()
    ; tooltip 边沿触发过滤器

    static 上次番茄状态 := "" ; : 番茄状态计算()

    global T_TomatoLife_NoticeOnLaunch ;
    tomatoLaunchFlag := 0 ;

    if ( 上次番茄状态 == "" && !T_TomatoLife_NoticeOnLaunch) {
        上次番茄状态 = 番茄状态计算()
        tomatoLaunchFlag := 1
    }
    ; tooltip 边沿触发过滤器
    ; tooltip %上次番茄状态% %番茄状态%
    if (上次番茄状态 == 番茄状态 && !force) {
        Return
    }
    上次番茄状态 := 番茄状态
    ; MsgBox, 番茄：%番茄状态%

    ; 状态动作
    SendEvent {Media_Play_Pause}
    if ("工作时间" == 番茄状态) {
        TrayTip % t("番茄时钟：") . %番茄状态%, % t("工作时间到啦！")
        SoundPlay % "Data/NoteC_G.mp3" ; 升调
        ; sleep 10000 ; 暂缓10秒切桌面
        ; && !tomatoLaunchFlag
        if (T_TomatoLife_UseTomatoLifeSwitchVirtualDesktop) {
            Func("SwitchToDesktop").Call(2) ; 切到工作桌面（桌面2）
        }
    }
    if ("休息时间" == 番茄状态) {
        TrayTip % t("番茄时钟：") . %番茄状态%, % t("起来运动一下收拾收拾东西！")
        SoundPlay % "Data/NoteG_C.mp3" ; 降调
        ; sleep 10000 ; 暂缓10秒切桌面
        ; && !tomatoLaunchFlag
        if(T_TomatoLife_UseTomatoLifeSwitchVirtualDesktop) {
            Func("SwitchToDesktop").Call(1) ; 切到休息桌面（桌面1）
        }
    }
}

UnixTimeGet()
{
    ; ref: https://www.autohotkey.com/boards/viewtopic.php?t=17333
    t := A_NowUTC
    EnvSub, t, 19700101000000, Seconds
    Return t * 1000 + A_MSec
}

CLX_番茄时钟定时任务:
    if (T_TomatoLife)
        番茄报时()
    ; 下次循环时间计算
    间隔 := 60000 ; 间隔为1分钟，精度到毫秒级
    延时 := (间隔 - Mod(UnixTimeGet(), 间隔))
    ; ToolTip, % 延时
    SetTimer CLX_番茄时钟定时任务, %延时%
Return
