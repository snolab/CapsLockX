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
        If(T_UseScrollLockLight){
            If(GetKeyState("ScrollLock", "T") != ((CapsXMode == CM_CAPSX) || (CapsXMode == CM_FN))){
                Send {ScrollLock}
                Return 1
            }
        }
        ;tips(CapsXMode)
    }
    
    CapsXTurnOff(){
        CapsXMode &= ~CM_CAPSX
        Return UpdateLight()
    }
    CapsXTurnOn(){
        CapsXMode |= CM_CAPSX
        Return UpdateLight()
    }

    Hotkey, %T_CapsXKey%, CapsX_Dn
    Hotkey, %T_CapsXKey% Up, CapsX_Up

; 动态开始：载入模块
    GoSub Setup_Anki方向键转换
    GoSub Setup_Cursor
    GoSub Setup_OneNote2016拓展
    GoSub Setup_OneNote拓展
    GoSub Setup_TIM添加常驻功能
    GoSub Setup_TIM连接OneNote2016
    GoSub Setup_网易云音乐
    GoSub Setup_加速模型
    GoSub Setup_模拟鼠标
    GoSub Setup_WinTab
    GoSub Setup_剪贴板增强
    GoSub Setup_媒体键
    GoSub Setup_帮助
    GoSub Setup_搜索键
    GoSub Setup_编辑增强
    Return
    #If
        Setup_Anki方向键转换:
            #Include 模块\应用-Anki方向键转换.ahk
    #If
        Setup_Cursor:
            #Include 模块\应用-CapsX-Cursor.ahk-bak
    #If
        Setup_OneNote2016拓展:
            #Include 模块\应用-OneNote2016拓展.ahk
    #If
        Setup_OneNote拓展:
            #Include 模块\应用-OneNote拓展.ahk
    #If
        Setup_TIM添加常驻功能:
            #Include 模块\应用-TIM添加常驻功能.ahk
    #If
        Setup_TIM连接OneNote2016:
            #Include 模块\应用-TIM连接OneNote2016.ahk
    #If
        Setup_网易云音乐:
            #Include 模块\应用-网易云音乐.ahk
    #If
        Setup_加速模型:
            #Include 模块\插件-00-加速模型.ahk
    #If
        Setup_模拟鼠标:
            #Include 模块\插件-01-模拟鼠标.ahk
    #If
        Setup_WinTab:
            #Include 模块\插件-02-WinTab.ahk
    #If
        Setup_剪贴板增强:
            #Include 模块\插件-剪贴板增强.ahk
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
            #Include 模块\插件-禁用-编辑增强.ahk
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