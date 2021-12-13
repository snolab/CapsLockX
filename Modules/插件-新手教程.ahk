; ========== CapsLockX ==========
; 名称：CLX 新手教程
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v0.0.1
; ========== CapsLockX ==========

global CapsLockX_FIRST_LAUNCH := CapsLockX_Config("_NOTICE_", "FIRST_LAUNCH", 1, "首次启动？若想重新进入首次使用教学，请改为 1 并保存，然后使用 Ctrl+Alt+\ 重载 CapsLockX。")
global CLX_CONFIG_ONSTARTUP := CapsLockX_Config("Core", "CLX_CONFIG_ONSTARTUP", 1, "启动时显示配置窗口")

Menu, Tray, Add ; Creates a separator line.
Menu, Tray, Add, 配置文件编辑, 配置文件编辑 ; Creates a new menu item.

if (CLX_CONFIG_ONSTARTUP) {
    SetTimer CapsLockX_首次使用配置, -1
}

return

; 修改配置
#if CapsLockXMode

m:: 配置文件编辑()


CapsLockX_首次使用配置(){
    CLX_CONFIG_ONSTARTUP := 0
    Gui, Add, Text, , CapsLockX_配置目录：%CapsLockX_配置目录%
    
    global T_XKeyAsCapsLock
    if (T_XKeyAsCapsLock){
        Gui, Add, CheckBox, gCapsLockX_配置刷新 vT_XKeyAsCapsLock Checked, 使用 CapsLock 作为引导键（默认启用）
    }else{
        Gui, Add, CheckBox, gCapsLockX_配置刷新 vT_XKeyAsCapsLock, 使用 CapsLock 作为引导键（默认启用）
    }
    
    global T_XKeyAsSpace
    if (T_XKeyAsSpace){
        Gui, Add, CheckBox, gCapsLockX_配置刷新 vT_XKeyAsSpace Checked, 使用 Space 作为引导键（默认启用）
    }else{
        Gui, Add, CheckBox, gCapsLockX_配置刷新 vT_XKeyAsSpace, 使用 Space 作为引导键（默认启用）
    }
    
    global T_AskRunAsAdmin
    if (T_AskRunAsAdmin){
        Gui, Add, CheckBox, gCapsLockX_配置刷新 vT_AskRunAsAdmin Checked, 请求管理员权限（权限受限时，鼠标模拟等功能无法正常运行，如果不需要管理权限下的功能，可以改为0）
    }else{
        Gui, Add, CheckBox, gCapsLockX_配置刷新 vT_AskRunAsAdmin, 请求管理员权限（权限受限时，鼠标模拟等功能无法正常运行，如果不需要管理权限下的功能，可以改为0）
    }
    global vCLX_CONFIG_ONSTARTUP
    if (vCLX_CONFIG_ONSTARTUP){
        Gui, Add, CheckBox, gCapsLockX_配置刷新 vCLX_CONFIG_ONSTARTUP Checked, 启动时显示配置窗口
    }else{
        Gui, Add, CheckBox, gCapsLockX_配置刷新 vCLX_CONFIG_ONSTARTUP, 启动时显示配置窗口
    }
    Gui, Add, Button, w80, 配置文件编辑
    Gui, Add, Button, Default w80, 确认
    Gui, Show
}

CapsLockX_配置刷新:
    gui,submit, nohide
    CapsLockX_ConfigSet("Core", "T_XKeyAsCapsLock", T_XKeyAsCapsLock, "使用 Space 作为引导键（默认启用，用户启用）")
    CapsLockX_ConfigSet("Core", "T_XKeyAsSpace", T_XKeyAsSpace, "使用 CapsLock 作为引导键（默认启用，用户启用）")
    CapsLockX_ConfigSet("Core", "T_AskRunAsAdmin", T_AskRunAsAdmin, "请求管理员权限（权限受限时，鼠标模拟等功能无法正常运行，如果不需要管理权限下的功能，可以改为0）")
    ; ToolTip conf %T_XKeyAsCapsLock% %T_XKeyAsSpace% %T_AskRunAsAdmin%
    return
Button确认:
    gui, destroy
    return
Button配置文件编辑:
    配置文件编辑()
    Return



CapsLockX_首次使用教学(){
    ; TODO
}

配置文件编辑(){
    Run notepad %CapsLockX_配置路径%
    ; TrayTip, 配置文件关, 自动重载
    ; CapsLockX_Reload()
}

