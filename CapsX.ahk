Process, Priority,,high         ;脚本高优先级
;#NoTrayIcon                        ;隐藏托盘图标
;#NoEnv                              ;不检查空变量是否为环境变量
;#Persistent                     ;让脚本持久运行(关闭或ExitApp)
#SingleInstance Force               ;跳过对话框并自动替换旧实例
;#MaxHotkeysPerInterval 300      ;时间内按热键最大次数
;#InstallMouseHook

#Include %A_ScriptDir%\Modules
;#Include Core.ahk



^!F12:: ExitApp


CapsLock:: Return
CapsLock Up::
    ; 给 Fn 键让路
    If(A_PriorHotkey != "CapsLock")
        Return
    ; 这里改注册表是为了禁用 Win + L，不过只有用管理员运行才管用。。。
    If(GetKeyState("ScrollLock", "T"))
        RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, 0
    Else
        RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, 1
    Send {ScrollLock}
    Return

!CapsLock:: CapsLock

; 鼠标加速度微分对称模型，每秒误差 2.5ms 以内
Global ta := 0, td := 0, tw := 0, ts := 0, mvx := 0, mvy := 0

; 滚轮加速度微分对称模型（不要在意名字hhhh
Global tr := 0, tf := 0, tz := 0, tc := 0, svx := 0, svy := 0

#If GetKeyState("CapsLock", "P") And !GetKeyState("ScrollLock", "T")
    H::
        Msgbox 帮助2
        Return

#If GetKeyState("CapsLock", "P") And GetKeyState("ScrollLock", "T")
    H::
        Msgbox 帮助1
        Return

#If GetKeyState("ScrollLock", "T")
    #Include Mouse.ahk

    Pause::
        RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, 1
        DllCall("LockWorkStation")
        Sleep, 1000
        RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, 0
        Return
    ~#Tab:: Send {ScrollLock}
    

    z:: Send {Enter}
    h:: Left
    l:: Right
    k:: Up
    j:: Down
    n:: Home
    m:: End
    b:: Send {Delete}

    ; 窗口转到Fn键
    x:: Send ^w
    !x:: Send !{F4}
    ; 撤销
    u:: Send ^z
    ; 重做
    +u:: Send ^z

    1:: Send #1
    2:: Send #2
    3:: Send #3
    4:: Send #4
    5:: Send #5
    6:: Send #6
    7:: Send #7
    8:: Send #8
    9:: Send #9

    F5:: Send {Media_Play_Pause}
    F6:: Send {Media_Prev}
    F7:: Send {Media_Next}
    F8:: Send {Media_Stop}

    
    F10:: Send {Volume_Mute}
    F11:: Send {Volume_Down}
    F12:: Send {Volume_Up}

    ; Google 搜索
    search(q)
    {
        Run, https://www.google.com/search?q=%q%
    }
    copySelected()
    {
        Send ^c
        ClipWait
        Return Clipboard
    }
    g:: search(copySelected())

