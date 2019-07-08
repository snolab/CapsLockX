; UTF-8 with BOM
; 
; 程序核心
; 最后更新：(20190707)
;
; Copyright © 2017-2019 snomiao@gmail.com
; 创建：Snowstar QQ: 997596439
; 参与完善：张工 QQ: 45289331
; LICENCE: GNU GPLv3
; 

Process Priority, , High     ; 脚本高优先级
#SingleInstance Force        ; 跳过对话框并自动替换旧实例
; #NoTrayIcon                ; 隐藏托盘图标
; #NoEnv                     ; 不检查空变量是否为环境变量
; #Persistent                ; 让脚本持久运行(关闭或ExitApp)
; #MaxHotkeysPerInterval 300 ; 时间内按热键最大次数
; #InstallMouseHook          ; 安装鼠标钩子

; 载入设定
#Include CapslockX-Settings.ahk
If(!)
    ExitApp

#If
    If(T_AskRunAsAdmin)
    {
        full_command_line := DllCall("GetCommandLine", "str")
        If(!A_IsAdmin And !RegExMatch(full_command_line, " /restart(?!\S)"))
        {
            Try{
                If A_IsCompiled
                    Run *RunAs "%A_ScriptFullPath%" /restart, "%A_WorkingDir%"
                Else
                    Run *RunAs "%A_AhkPath%" /restart "%A_ScriptFullPath%", "%A_WorkingDir%"
            }
            ExitApp
        }
    }

    ; 模式处理
    global CapslockX := 1 ; 模块运行标识符
    
    global CapslockXMode   := 0
    global ModuleState := 0
    global CapslockX_FnActed   := 0
    global CM_NORMAL := 0 ; 普通模式
    global CM_FN     := 1 ; 临时 CapslockX 模式
    global CM_CapslockX  := 2 ; CapslockX 模式
    global CM_FNX    := 3 ; FnX 模式
    global LastLightState := ((CapslockXMode & CM_CapslockX) || (CapslockXMode & CM_FN))
    ; 切换模式
    UpdateCapslockXMode(){
        CapslockXMode := GetKeyState(T_CapslockXKey, "P")
        If(T_UseScrollLockLight)
            CapslockXMode |= GetKeyState("ScrollLock", "T") << 1
        
        Return CapslockXMode
    }
    UpdateCapslockXMode()
    ; 根据当前模式，切换灯
    Menu,tray,icon,./数据/图标白.ico
    UpdateLight(){
        NowLightState := ((CapslockXMode & CM_CapslockX) || (CapslockXMode & CM_FN))
        If ( NowLightState && !LastLightState){
            Menu,tray,icon, ./数据/图标蓝.ico
            If (T_SwitchSoundOn && T_SwitchSoundOn){
                SoundPlay %T_SwitchSoundOn%
            }
        }
        If ( !NowLightState && LastLightState ){
            Menu,tray,icon,./数据/图标白.ico
            If (T_SwitchSoundOn && T_SwitchSoundOff){
                SoundPlay %T_SwitchSoundOff%
            }
        }
        If (T_UseScrollLockLight){
            ; ToolTip % CapslockXMode
            If (GetKeyState("ScrollLock", "T") != ((CapslockXMode & CM_CapslockX) || (CapslockXMode & CM_FN))){
                Send {ScrollLock}
                Return 1
            }
        }
        ; tips(CapslockXMode)
        LastLightState := NowLightState
    }
    
    CapslockXTurnOff(){
        CapslockXMode &= ~CM_CapslockX
        re =: UpdateLight()
        Return re
    }
    CapslockXTurnOn(){
        CapslockXMode |= CM_CapslockX
        re =: UpdateLight()
        Return re
    }

    Hotkey *%T_CapslockXKey%, CapslockX_Dn
    Hotkey *%T_CapslockXKey% Up, CapslockX_Up

; 动态开始：载入模块
    GoSub Setup_加速模型
    GoSub Setup_模拟鼠标
    GoSub Setup_Anki增强
    GoSub Setup_秒打时间戳
    GoSub Setup_Acrobat增强
    GoSub Setup_Acrobat自动缩放
    GoSub Setup_Cursor
    GoSub Setup_Edge增强
    GoSub Setup_DAP
    GoSub Setup_LoopbackExemptionManager
    GoSub Setup_mstsc远程桌面增强
    GoSub Setup_OneNote2016增强
    GoSub Setup_OneNoteMetro拓展
    GoSub Setup_QQ_UWP增强
    GoSub Setup_TIM添加常驻功能
    GoSub Setup_TIM连接OneNote2016
    GoSub Setup_UWP应用增强
    GoSub Setup_文明6回车左置
    GoSub Setup_网易云音乐
    GoSub Setup_讯飞输入法语音悬浮窗
    GoSub Setup_窗口增强
    GoSub Setup_Chrome增强
    GoSub Setup_OneNote剪贴板收集器
    GoSub Setup_合并右Ctrl与Menu键
    GoSub Setup_媒体键
    GoSub Setup_帮助
    GoSub Setup_搜索键
    GoSub Setup_编辑增强
    GoSub Setup_自动滚动
    GoSub Setup_雪星转屏
    Return
    #If
        Setup_加速模型:
            #Include 模块\00-插件-加速模型.ahk
    #If
        Setup_模拟鼠标:
            #Include 模块\01-插件-模拟鼠标.ahk
    #If
        Setup_Anki增强:
            #Include 模块\03-应用-Anki增强.ahk
    #If
        Setup_秒打时间戳:
            #Include 模块\功能-秒打时间戳.ahk
    #If
        Setup_Acrobat增强:
            #Include 模块\应用-Acrobat增强.ahk
    #If
        Setup_Acrobat自动缩放:
            #Include 模块\应用-Acrobat自动缩放.ahk
    #If
        Setup_Cursor:
            #Include 模块\应用-CapsX-Cursor.ahk
    #If
        Setup_Edge增强:
            #Include 模块\应用-Edge增强.ahk
    #If
        Setup_DAP:
            #Include 模块\应用-IAR改选项为CMSIS-DAP.ahk
    #If
        Setup_LoopbackExemptionManager:
            #Include 模块\应用-LoopbackExemptionManager.ahk
    #If
        Setup_mstsc远程桌面增强:
            #Include 模块\应用-mstsc远程桌面增强.ahk
    #If
        Setup_OneNote2016增强:
            #Include 模块\应用-OneNote2016增强.ahk
    #If
        Setup_OneNoteMetro拓展:
            #Include 模块\应用-OneNoteMetro拓展.ahk
    #If
        Setup_QQ_UWP增强:
            #Include 模块\应用-QQ_UWP增强.ahk
    #If
        Setup_TIM添加常驻功能:
            #Include 模块\应用-TIM添加常驻功能.ahk
    #If
        Setup_TIM连接OneNote2016:
            #Include 模块\应用-TIM连接OneNote2016.ahk
    #If
        Setup_UWP应用增强:
            #Include 模块\应用-UWP应用增强.ahk
    #If
        Setup_文明6回车左置:
            #Include 模块\应用-文明6回车左置.ahk
    #If
        Setup_网易云音乐:
            #Include 模块\应用-网易云音乐.ahk
    #If
        Setup_讯飞输入法语音悬浮窗:
            #Include 模块\应用-讯飞输入法语音悬浮窗.ahk
    #If
        Setup_窗口增强:
            #Include 模块\插件-02-窗口增强.ahk
    #If
        Setup_Chrome增强:
            #Include 模块\插件-Chrome增强.ahk
    #If
        Setup_OneNote剪贴板收集器:
            #Include 模块\插件-OneNote剪贴板收集器.ahk
    #If
        Setup_合并右Ctrl与Menu键:
            #Include 模块\插件-合并右Ctrl与Menu键.ahk
    #If
        Setup_媒体键:
            #Include 模块\插件-媒体键.ahk
    #If
        Setup_帮助:
            #Include 模块\插件-帮助.ahk
    #If
        Setup_搜索键:
            #Include 模块\插件-搜索键.ahk
    #If
        Setup_编辑增强:
            #Include 模块\插件-编辑增强.ahk
    #If
        Setup_自动滚动:
            #Include 模块\插件-自动滚动.ahk
    #If
        Setup_雪星转屏:
            #Include 模块\插件-雪星转屏.ahk
; 动态结束；
#If
    ; CapslockX模式切换
    CapslockX_Dn:
        ; 进入 Fn 模式
        CapslockXMode |= CM_FN
        ; 限制在远程桌面里无法进入 Fn 模式，避免和远程桌面里的 CapslockX 冲突
        if (WinActive("ahk_class TscShellContainerClass ahk_exe mstsc.exe")){
            CapslockXMode &= ~CM_FN
        }

        UpdateLight()
        Return

    CapslockX_Up:
        ; 退出 Fn 模式
        CapslockXMode &= ~CM_FN
        ; 规避 Fn 功能键
        CapslockX_FnActed := CapslockX_FnActed || (A_PriorKey != T_CapslockXKey && A_PriorKey != "Insert")
        If (!CapslockX_FnActed) {
            CapslockXMode ^= CM_CapslockX

            ; 限制在远程桌面里无法进入 CapslockX 模式，避免和远程桌面里的 CapslockX 冲突
            if (WinActive("ahk_class TscShellContainerClass ahk_exe mstsc.exe")){
                CapslockXMode &= ~CM_CapslockX
            }
        }
        ;ToolTip, %CapslockX_FnActed%
        CapslockX_FnActed := 0
        UpdateLight()
        Return

; 
; #If T_CapslockXKey == "CapsLock"
;     !CapsLock:: CapsLock ; 

; 用ScrollLock代替Capslock键
#If T_UseScrollLockAsCapslock
    $ScrollLock:: CapsLock

#If T_UseDoubleClickShiftAsCapslock
    ; TODO

#If
    ; 软重启键
    !F12:: Reload

    ; 硬重启键
    ^!F12::
        Run CapslockX.ahk, %A_WorkingDir%
        ExitApp
        Return

    ; 结束键
    ^!+F12:: ExitApp

    *Insert:: GoSub CapslockX_Dn
    *Insert Up:: GoSub CapslockX_Up