; ========== CapsLockX ==========
; 名称：CLX 新手教程
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v0.0.1
; ========== CapsLockX ==========

global CapsLockX_FIRST_LAUNCH := CapsLockX_Config("_NOTICE_", "FIRST_LAUNCH", 1, "首次启动？若想重新进入首次使用教学，请改为 1 并保存，然后使用 Ctrl+Alt+\ 重载 CapsLockX。")
if(CapsLockX_FIRST_LAUNCH)
    CapsLockX_首次使用教学()
return

; 修改配置
#if CapsLockXMode
    m:: 配置文件编辑()


CapsLockX_首次使用教学2(){
    
    Gui, Add, CheckBox, T_XKeyAsCapsLock, 使用 CapsLock 作为引导键（默认启用，用户启用）
    Gui, Add, CheckBox, T_AskRunAsAdmin, 请求管理员权限（权限受限时，鼠标模拟等功能无法正常运行，如果不需要管理权限下的功能，可以改为0）
    Gui, Add, CheckBox, FIRST_LAUNCH, 启动时显示配置
    
    ; 打开进阶配置编辑器？
    ; SetTimer, 配置文件编辑, -1
    
    CapsLockX_ConfigSet("_NOTICE_", "FIRST_LAUNCH", 0)
}

CapsLockX_首次使用教学(){
    MsgBox, 4, CapsLockX 教程, 首次启动 CapsLockX ，是否进行基本偏好设置？`n`n（你可以随时按 CapsLockX + M 修改配置文件）
    IfMsgBox No
    return

    MsgBox, 4, CapsLockX 教程, 是否使用空格作为 CapsLockX 引导键？（例如，启用后，使用空格组合键 空格 + WASD 可控制鼠标，而单独按下时将保持原空格的功能，不影响打字）
    IfMsgBox Yes
    global T_XKeyAsCapsLock := CapsLockX_ConfigSet("Core", "T_XKeyAsCapsLock", 1, "使用 CapsLock 作为引导键（默认启用，用户启用）")
    IfMsgBox No
    global T_XKeyAsCapsLock := CapsLockX_ConfigSet("Core", "T_XKeyAsCapsLock", 0, "使用 CapsLock 作为引导键（默认启用，用户禁用）")

    MsgBox, 4, CapsLockX 教程, 是否在启动时询问管理员权限？（权限受限时，权限受限，例如鼠标模拟等功能无法正常运行，默认请求提升权限，如果不需要管理权限下的功能，可以改为0）
    IfMsgBox Yes
    global T_AskRunAsAdmin := CapsLockX_ConfigSet("Core", "T_AskRunAsAdmin", 1, "请求管理员权限（权限受限时，鼠标模拟等功能无法正常运行，如果不需要管理权限下的功能，可以改为0）")
    IfMsgBox No
    global T_AskRunAsAdmin := CapsLockX_ConfigSet("Core", "T_AskRunAsAdmin", 0, "请求管理员权限（权限受限时，鼠标模拟等功能无法正常运行，如果不需要管理权限下的功能，可以改为0）")

    AskRunAsAdminIfNeeded()
    
    MsgBox, 4, CapsLockX 教程, 完成，是否打开进阶配置编辑器？
    IfMsgBox Yes
    SetTimer, 配置文件编辑, -1000

    CapsLockX_ConfigSet("_NOTICE_", "FIRST_LAUNCH", 0)
}
配置文件编辑(){
    Run notepad "%CapsLockX_配置路径%"
    ; TrayTip, 配置文件关, 自动重载
    ; CapsLockX_Reload()
}
