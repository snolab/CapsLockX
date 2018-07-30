Process Priority, , High     ; 脚本高优先级
#SingleInstance Force        ; 跳过对话框并自动替换旧实例
; #NoTrayIcon                ; 隐藏托盘图标
; #NoEnv                     ; 不检查空变量是否为环境变量
; #Persistent                ; 让脚本持久运行(关闭或ExitApp)
; #MaxHotkeysPerInterval 300 ; 时间内按热键最大次数
; #InstallMouseHook          ; 安装鼠标钩子

; 载入设定
#Include CapsX-Settings.ahk
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
    global CapsX := 1 ; 模块运行标识符
    
    global CapsXMode   := 0
    global ModuleState := 0
    global CapsX_FnActed   := 0
    global CM_NORMAL := 0 ; 普通模式
    global CM_FN     := 1 ; 临时 CapsX 模式
    global CM_CAPSX  := 2 ; CapsX 模式
    global CM_FNX    := 3 ; FnX 模式

    ; 切换模式
    UpdateCapsXMode(){
        CapsXMode := GetKeyState(T_CapsXKey, "P")
        If(T_UseScrollLockLight)
            CapsXMode |= GetKeyState("ScrollLock", "T") << 1
        
        Return CapsXMode
    }
    UpdateCapsXMode()
    ; 根据当前模式，切换灯
    Menu,tray,icon,./数据/图标白.ico
    UpdateLight(){
        If (  ((CapsXMode & CM_CAPSX) || (CapsXMode & CM_FN)) ){
            Menu,tray,icon, ./数据/图标蓝.ico
            If (T_SwitchSoundOn && T_SwitchSoundOn){
                SoundPlay %T_SwitchSoundOn%
            }
        }Else{
            Menu,tray,icon,./数据/图标白.ico
            If (T_SwitchSoundOn && T_SwitchSoundOff){
                SoundPlay %T_SwitchSoundOff%
            }
        }
        If (T_UseScrollLockLight){
            ; ToolTip % CapsXMode
            If (GetKeyState("ScrollLock", "T") != ((CapsXMode & CM_CAPSX) || (CapsXMode & CM_FN))){
                Send {ScrollLock}
                Return 1
            }
        }
        ; tips(CapsXMode)
    }
    
    CapsXTurnOff(){
        CapsXMode &= ~CM_CAPSX
        re =: UpdateLight()
        Return re
    }
    CapsXTurnOn(){
        CapsXMode |= CM_CAPSX
        re =: UpdateLight()
        Return re
    }

    Hotkey *%T_CapsXKey%, CapsX_Dn
    Hotkey *%T_CapsXKey% Up, CapsX_Up

; 动态开始：载入模块
    GoSub Setup_加速模型
    GoSub Setup_模拟鼠标
    GoSub Setup_Anki增强
    GoSub Setup_秒打时间戳
    GoSub Setup_Acrobat增强
    GoSub Setup_Acrobat自动缩放
    GoSub Setup_Cursor
    GoSub Setup_Edge手写热键
    GoSub Setup_DAP
    GoSub Setup_LoopbackExemptionManager
    GoSub Setup_mstsc远程桌面增强
    GoSub Setup_OneNote2016增强
    GoSub Setup_OneNoteMetro拓展
    GoSub Setup_TIM添加常驻功能
    GoSub Setup_TIM连接OneNote2016
    GoSub Setup_UWP应用增强
    GoSub Setup_文明6回车左置
    GoSub Setup_网易云音乐
    GoSub Setup_窗口增强
    GoSub Setup_媒体键
    GoSub Setup_帮助
    GoSub Setup_搜索键
    GoSub Setup_编辑增强
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
            #Include 模块\应用-CapsX-Cursor.ahk-禁用
    #If
        Setup_Edge手写热键:
            #Include 模块\应用-Edge手写热键.ahk
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
        Setup_窗口增强:
            #Include 模块\插件-02-窗口增强.ahk
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
        Setup_雪星转屏:
            #Include 模块\插件-雪星转屏.ahk
; 动态结束；
#If

    ; CapsX模式切换
    CapsX_Dn:
        CapsXMode |= CM_FN
        UpdateLight()
        Return

    CapsX_Up:
        CapsXMode &= ~CM_FN
        ; 规避 Fn 功能键
        CapsX_FnActed := CapsX_FnActed || (A_PriorKey != T_CapsXKey && A_PriorKey != "Insert")
        If (!CapsX_FnActed) {
            CapsXMode ^= CM_CAPSX
        }
        ;ToolTip, %CapsX_FnActed%
        CapsX_FnActed := 0
        UpdateLight()
        Return

; 
; #If T_CapsXKey == "CapsLock"
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
        Run CapsX.ahk, %A_WorkingDir%
        ExitApp
        Return

    ; 结束键
    ^!+F12:: ExitApp

    *Insert:: GoSub CapsX_Dn
    *Insert Up:: GoSub CapsX_Up