; ========== CapsLockX ==========
; 名称：快速窗口热键编辑
; 描述：快速窗口热键编辑
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2021.03.18
; 注释：
; ========== CapsLockX ==========

if (!CapsLockX){
    MsgBox, % "本模块只为 CapsLockX 工作"
    ExitApp
}
CapsLockX_AppendHelp( CapsLockX_LoadHelpFrom(CapsLockX_THIS_MODULE_HELP_FILE_PATH))

global 快速窗口热键编辑用户模块目录 := CapsLockX_用户目录 "/"
global 快速窗口热键编辑初始内容 := "
(
; ========== CapsLockX ==========
; 名称：用户模块
; 描述：快速窗口热键编辑
; 作者：你自己
; 联系：你的邮箱 或 QQ
; 版本：0.0.1
; ========== CapsLockX ==========

; 1. 本用户模块文件由 CapsLockX 初始生成，扩展名为 .user.ahk ，不会被版本更新覆盖。
; 2. 使用 CapsLockX + M 键创建窗口热键，并快速编辑本文件（默认打开方式是记事本，你可以按自己个人情况酌情安装 Notepad3 ）
; 3. 编辑完成时，使用 Ctrl+Alt+\ 键重载 CapsLockX 即可生效。
; 4. 需要注意的是本模块有语法错误会导致 CapsLockX 重载失败，请自行调试到成功。。。

; 这里可以写一些入口代码
; TrayTip CapsLockX, 用户宏已加载

; 这里写上 Return 防止加载的时候执行到下面的热键
Return
#if

; 这里可以写上你的自定义全局热键
)"
Menu, Tray, Add ; Creates a separator line.
Menu, Tray, Add, 配置文件编辑, 配置文件编辑 ; Creates a new menu item.

Return

#if CapsLockXMode

UserModuleEdit(路径, 使用进程名AHK := 0){
    WinGet, hWnd, ID, A
    WinGetClass, 窗口类名, ahk_id %hWnd%
    WinGet, 进程名, ProcessName, ahk_id %hWnd%
    if(使用进程名AHK)
        路径 := 路径 "/应用-" 进程名 ".user.ahk"
    WinGetTitle, title, ahk_id %hWnd%
    match = %title% ahk_class %窗口类名% ahk_exe %进程名%
    if (!FileExist(路径))
        FileAppend, %快速窗口热键编辑初始内容%, %路径%
    填充内容 := "`n" "`n" "#if WinActive(""" match """)" "`n" "`n" "!```:`: TrayTip, CapsLockX, 在当前窗口按下了Alt+````" "`n" 
    FileAppend, %填充内容%, %路径%
    Run notepad %路径%
}

; 快速创建宏
; !m:: UserModuleEdit(快速窗口热键编辑用户模块目录 "/CapsLockX_用户脚本.user.ahk")
!m:: UserModuleEdit(快速窗口热键编辑用户模块目录, "使用进程名AHK")

