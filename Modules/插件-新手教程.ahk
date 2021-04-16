; ========== CapsLockX ==========
; 名称：CLX 新手教程
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v0.0.1
; ========== CapsLockX ==========

global CapsLockX_FIRST_LAUNCH := CapsLockX_Config("_NOTICE_", "FIRST_LAUNCH", 1, "首次启动？")
if(CapsLockX_FIRST_LAUNCH)
    CapsLockX_首次使用说明()
return

; 修改配置
#if CapsLockXMode
    m:: 配置文件编辑()

CapsLockX_首次使用说明(){
    MsgBox, 4, CapsLockX 教程, 首次启动 CapsLockX ，是否打开配置文件进行基本调整？`n`n（你可以随时按 CapsLockX + M 打开配置文件）
    IfMsgBox No
    return
    SetTimer, 配置文件编辑, -1000
    CapsLockX_ConfigSet("_NOTICE_", "FIRST_LAUNCH", 0)
}
配置文件编辑(){
    Run notepad %CapsLockX_配置路径%
}
