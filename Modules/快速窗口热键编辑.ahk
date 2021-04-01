; ========== CapsLockX ==========
; 名称：快速窗口热键编辑
; 描述：快速窗口热键编辑
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2021.03.18
; 注释：
; ========== CapsLockX ==========

if (!CapsLockX) {
    MsgBox, % "本模块只在 CapsLockX 下工作"
    ExitApp
}

global 快速窗口热键编辑用户模块目录 := CapsLockX_PathModules
global 快速窗口热键编辑初始内容 := "
(
; 1. 本用户模块文件由 CapsLockX 初始生成，扩展名为 .user.ahk ，不会被版本更新覆盖。
; 2. 使用 CapsLockX + M 键创建窗口热键，并快速编辑本文件（默认打开方式是记事本，你可以按自己个人情况酌情安装 Notepad3 ）
; 3. 编辑完成时，使用 Ctrl+Alt+\ 键重载 CapsLockX 即可生效。
; 4. 需要注意的是本模块有语法错误会导致 CapsLockX 重载失败，请自行调试到成功。。。

TrayTip CapsLockX, 用户宏已加载

; 这里的 Return 需要保留才能正常加载。
Return

#if

; 这里可以写上你的自定义全局热键

)"
Return

#if CapsLockXMode

UserModuleEdit(路径){
    WinGet, hWnd, ID, A
    WinGetClass, 窗口类名, ahk_id %hWnd%
    WinGet, 进程名, ProcessName, ahk_id %hWnd%
    WinGetTitle, title, ahk_id %hWnd%
    match = %title% ahk_class %窗口类名% ahk_exe %进程名%
    if (!FileExist(路径))
        FileAppend, %快速窗口热键编辑初始内容%, %路径%
    填充内容 := "`n" "`n" "#if WinActive(""" match """)" "`n" "`n" "!```:`: TrayTip, CapsLockX, 在当前窗口按下了Alt+````" "`n" 
    FileAppend, %填充内容%, %路径%
    Run notepad %路径%
}

m:: UserModuleEdit(快速窗口热键编辑用户模块目录 "/快速窗口热键编辑内容.user.ahk")
!m:: UserModuleEdit(快速窗口热键编辑用户模块目录 "/应用-" 进程名 ".user.ahk")
; 修改配置
^!m:: run notepad %CapsLockXConfigPath%