; ========== CapsLockX ==========
; 名称：快速窗口热键编辑
; 描述：rt
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

global 快速窗口热键编辑用户模块路径 := CapsLockX_PathModules "/快速窗口热键编辑.user.ahk"

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
; 这里可以写上你的自定义全局热键，由CapsLockX。

)"
Return

#if CapsLockXMode

m:: 
    WinGet, Active_ID, ID, A
    WinGetClass, cls, ahk_id %Active_ID%
    WinGet, Active_Process, ProcessName, ahk_id %Active_ID%
    WinGetTitle, title, ahk_id %Active_ID%
    match = %title% ahk_class %cls% ahk_exe %Active_Process%
    if (!FileExist(快速窗口热键编辑用户模块路径)){
        FileAppend, %快速窗口热键编辑初始内容%, %快速窗口热键编辑用户模块路径%
    }
    ifstatement := "`n" "#if WinActive(""" match """)" "`n" "`n" "!```:`: TrayTip, CapsLockX, 在当前窗口按下了Alt+````" "`n" 
    FileAppend, %ifstatement%, %快速窗口热键编辑用户模块路径%
    Run notepad %快速窗口热键编辑用户模块路径%
return