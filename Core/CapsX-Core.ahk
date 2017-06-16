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
    UpdateLight(){
        If(T_UseScrollLockLight)
            SetScrollLockState % (CapsXMode & CM_CAPSX) ? "AlwaysOn" : "AlwaysOff"
        ;tips(CapsXMode)
    }
    
    CapsXOff(){
        CapsXMode &= ~CM_CAPSX
        UpdateLight()
    }


    Hotkey, %T_CapsXKey%, CapsX_Dn
    Hotkey, %T_CapsXKey% Up, CapsX_Up

; 动态开始：载入模块
        GoSub Setup_Accelerate
        GoSub Setup_Mouse
        GoSub Setup_WinTab
        GoSub Setup_Clip
        GoSub Setup_Edit
        GoSub Setup_Help
        GoSub Setup_Media
        GoSub Setup_Search
    Return
    #If
        Setup_Accelerate:
        #Include Modules\00-Accelerate.ahk
    #If
        Setup_Mouse:
        #Include Modules\01-Mouse.ahk
    #If
        Setup_WinTab:
        #Include Modules\02-WinTab.ahk
    #If
        Setup_Clip:
        #Include Modules\Clip.ahk
    #If
        Setup_Edit:
        #Include Modules\Edit.ahk
    #If
        Setup_Help:
        #Include Modules\Help.ahk
    #If
        Setup_Media:
        #Include Modules\Media.ahk
    #If
        Setup_Search:
        #Include Modules\Search.ahk
; 动态结束；

    CapsX_Dn:
        CapsXMode |= CM_FN
        UpdateLight()
        Return

    CapsX_Up:
        CapsXMode &= ~CM_FN
        ; 规避 Fn 功能键
        If(A_PriorHotKey == T_CapsXKey)
            CapsXMode ^= CM_CAPSX
        UpdateLight()
        Return

#If T_CapsXKey == "CapsLock"
    !CapsLock:: CapsLock ; 

#If T_UseScrollLockLight
    $ScrollLock:: CapsLock


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