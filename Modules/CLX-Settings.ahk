; ========== CapsLockX ==========
; 名称：CLX 配置修改
; 描述：提供一个GUI用于 修改 CLX 的配置，热键为 CapsLockX+m
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v0.0.1
; ========== CapsLockX ==========

global CLX_FIRST_LAUNCH := CLX_Config("_NOTICE_", "FIRST_LAUNCH", 1, t("首次启动？若想重新进入首次使用教学，请改为 1 并保存，然后使用 Ctrl+Alt+\ 重载 CapsLockX。"))
global CLX_CONFIG_ONSTARTUP := CLX_Config("Core", "CLX_CONFIG_ONSTARTUP", 1, t("启动时显示配置窗口"))
global CLX_Lang
Menu, Tray, Add ; Creates a separator line.
Menu, Tray, Add, % t("Edit config.ini"), CLX_ConfigureEdit
Menu, Tray, Add, % t("Reload CapsLockX"), CLX_Reload
Menu, Tray, Add, % t("Exit CapsLockX"), CLX_Exit

if (CLX_CONFIG_ONSTARTUP) {
    SetTimer CLX_ConfigWindow, -1
}
return

CLX_Exit()
{
    ExitApp
}
; 修改配置
#if CapsLockXMode

;, :: 配置文件编辑()
,:: CLX_ConfigWindow()

CLX_ConfigWindow()
{
    Gui, Destroy
    ; TODO: update this to web view
    ; webview2 needs windows 10, but CLX still want support win7, so maybe other way to do this
    Gui, Add, Text, , % t("你可以按 'CapsLockX +, ' （CLX+逗号） 打开此窗口")
    Gui, Add, Text, , % t("当前配置目录：") . CLX_ConfigDir
    Gui, Add, Text, , % t("CLX脚本交流群： QQ群 100949388 、 Telegram 群 https://t.me/capslockx 、微信群: 添加 @snomiao 拉你")
    Gui, Add, Text, , % "CapsLockX " . t("'版本'") . ": " . CLX_VersionName
    Gui, Add, Text, , % t("作者: 雪星 ( Snowstar Miao <snomiao@gmail.com> )")
    Gui, Add, Button, Default w120 gButton确认, % t("确定")
    Gui, Add, Button, w200 gButtonLanguageSwitch, % t("'切换語言'") . "`n" . t("'Current Language is: '") . " " .  CLX_Lang
    Gui, Add, Button, w200 gButton打开BUG反馈与建议页面, % t("打开BUG反馈与建议页面")
    Gui, Add, Button, w200 gButton打开官方文档, % t("打开官方文档")
    Gui, Add, Button, w200 gButton添加开机自动启动, % t("添加开机自动启动")
    Gui, Add, Button, w200 gButton配置文件编辑, % t("配置文件编辑")
    Gui, Add, Button, w200 gButton重新載入, % t("重新載入CapsLockX")

    global T_TomatoLife ;
    if (T_TomatoLife) {
        Gui, Add, CheckBox, gCLX_ConfigureUpdate vT_TomatoLife Checked, % t("启用番茄时钟，每25分钟休息5分钟·。")
    } else {
        Gui, Add, CheckBox, gCLX_ConfigureUpdate vT_TomatoLife, % t("启用番茄时钟，每25分钟休息5分钟·。")
    }

    global T_XKeyAsCapsLock
    if (T_XKeyAsCapsLock) {
        Gui, Add, CheckBox, gCLX_ConfigureUpdate vT_XKeyAsCapsLock Checked, % t("使用 CapsLock 作为引导键（默认启用）")
    } else {
        Gui, Add, CheckBox, gCLX_ConfigureUpdate vT_XKeyAsCapsLock, % t("使用 CapsLock 作为引导键（默认启用）")
    }

    global T_XKeyAsSpace
    if (T_XKeyAsSpace) {
        Gui, Add, CheckBox, gCLX_ConfigureUpdate vT_XKeyAsSpace Checked, % t("使用 Space 作为引导键（默认启用）")
    } else {
        Gui, Add, CheckBox, gCLX_ConfigureUpdate vT_XKeyAsSpace, % t("使用 Space 作为引导键（默认启用）")
    }

    global T_AskRunAsAdmin
    if (T_AskRunAsAdmin) {
        Gui, Add, CheckBox, gCLX_ConfigureUpdate vT_AskRunAsAdmin Checked, % t("请求管理员权限（权限受限时，鼠标模拟等功能无法正常运行，如果不需要管理权限下的功能，可以改为0）")
    } else {
        Gui, Add, CheckBox, gCLX_ConfigureUpdate vT_AskRunAsAdmin, % t("请求管理员权限（权限受限时，鼠标模拟等功能无法正常运行，如果不需要管理权限下的功能，可以改为0）")
    }
    global vCLX_CONFIG_ONSTARTUP
    if (vCLX_CONFIG_ONSTARTUP) {
        Gui, Add, CheckBox, gCLX_ConfigureUpdate vCLX_CONFIG_ONSTARTUP Checked, % t("启动时显示配置窗口")
    } else {
        Gui, Add, CheckBox, gCLX_ConfigureUpdate vCLX_CONFIG_ONSTARTUP, % t("启动时显示配置窗口")
    }
    Gui, Show
}

Button添加开机自动启动:
    Func("CLX_MakeStartup").Call()
return
Button打开BUG反馈与建议页面:
    Run https://github.com/snolab/CapsLockX/issues
return
Button打开官方文档:
    Run https://capslockx.snomiao.com/
return
Button重新載入:
    Func("CLX_Reload").Call()
    Reload
return
CLX_ConfigureUpdate:
    global T_TomatoLife
        global T_XKeyAsCapsLock
    global T_XKeyAsSpace
    global T_AskRunAsAdmin
    global CLX_CONFIG_ONSTARTUP
    Gui, Submit, NoHide
    reloadFlag := 0
    reloadFlag := reloadFlag || ( CLX_ConfigGet("Core", "T_XKeyAsCapsLock", T_XKeyAsCapsLock) != T_XKeyAsCapsLock )
    reloadFlag := reloadFlag || ( CLX_ConfigGet("Core", "T_XKeyAsSpace", T_XKeyAsSpace) != T_XKeyAsSpace )
    reloadFlagAdmin := 0
    reloadFlagAdmin := reloadFlagAdmin || ( CLX_ConfigGet("Core", "T_AskRunAsAdmin", T_AskRunAsAdmin) != T_AskRunAsAdmin )
    CLX_ConfigSet("TomatoLife", "Enable", T_TomatoLife, t("使用番茄时钟（默认禁用，改为 1 开启）"))
    CLX_ConfigSet("Core", "T_XKeyAsCapsLock", T_XKeyAsCapsLock, t("使用 Space 作为引导键（默认启用，用户启用）"))
    CLX_ConfigSet("Core", "T_XKeyAsSpace", T_XKeyAsSpace, t("使用 CapsLock 作为引导键（默认启用，用户启用）"))
    CLX_ConfigSet("Core", "T_AskRunAsAdmin", T_AskRunAsAdmin, t("请求管理员权限（权限受限时，鼠标模拟等功能无法正常运行，如果不需要管理权限下的功能，可以改为0）"))
    CLX_ConfigSet("Core", "CLX_CONFIG_ONSTARTUP", CLX_CONFIG_ONSTARTUP, t("启动时显示配置窗口"))
    if (reloadFlag) {
        reload
    }
    if (reloadFlagAdmin) {
        Func("AskRunAsAdminIfNeeded").Call()
    }
return
Button确认:
    Gosub, CLX_ConfigureUpdate
    ; TrayTip conf, %T_XKeyAsCapsLock% %T_XKeyAsSpace% %T_AskRunAsAdmin% %CLX_CONFIG_ONSTARTUP%
    gui, destroy
return
ButtonLanguageSwitch:
    CLX_LanguageSwitch()
return
Button配置文件编辑:
    CLX_ConfigureEdit()
Return

CLX_LanguageSwitch(){
    msg := t("'Choose your language'") . "`n" . t("For example: zh,ja,en,fr,es,ar...'")
    InputBox, targetLang, % t("Change Language of CapsLockX"), % msg ,,,,,,,,% CLX_Lang
    ; InputBox, OutputVar [, Title, Prompt, HIDE, Width, Height, X, Y, Locale, Timeout, Default
    ; targetLang
    if (targetLang) {
        i18n_changeLanguage(targetLang)
        Reload
    }
}
CLX_首次使用教学(){
    ; TODO
}

CLX_ConfigureEdit(){
    Run notepad %CLX_ConfigPath%
    ; TrayTip, 配置文件关, 自动重载
    ; CLX_Reload()
}
