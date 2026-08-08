; ========== CapsLockX ==========
; 名称：CLX开机运行
; 描述：管理 CLX 的开机自启动。优先使用计划任务（以最高权限运行，避免每次
;       登录弹出 UAC），失败时回退到用户 Startup 文件夹。
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v0.0.2
; ========== CapsLockX ==========

; 计划任务名，与 setup_autostart.bat 保持一致，两者可互相识别与清理。
global CLX_AUTOSTART_TASK := "CapsLockX_AutoStart"

return

; Startup 文件夹里的启动脚本路径。
; 注意用 A_AppData 而不是裸 APPDATA：后者在函数内是空的局部变量，
; 会把路径拼成相对路径。
CLX_AutostartCmdPath()
{
    return A_AppData "\Microsoft\Windows\Start Menu\Programs\Startup\capslockx-startup.cmd"
}

; 计划任务是否已存在。
CLX_AutostartTaskExists()
{
    global CLX_AUTOSTART_TASK
    q := """"
    query := "schtasks /query /tn " q CLX_AUTOSTART_TASK q " >nul 2>&1"
    RunWait, %A_ComSpec% /c %query%, , Hide UseErrorLevel
    return (ErrorLevel = 0)
}

; 两种方式任意一种生效即视为已启用。
CLX_AutostartEnabled()
{
    if (FileExist(CLX_AutostartCmdPath()))
        return true
    return CLX_AutostartTaskExists()
}

; 以管理员权限运行 schtasks；已经是管理员时直接静默运行。
; 返回 schtasks 的 ErrorLevel（用户拒绝 UAC 时为非 0 或 "ERROR"）。
CLX_AutostartSchtasks(args)
{
    if (A_IsAdmin) {
        RunWait, %A_ComSpec% /c schtasks %args%, , Hide UseErrorLevel
    } else {
        RunWait, *RunAs %A_ComSpec% /c schtasks %args%, , Hide UseErrorLevel
    }
    return ErrorLevel
}

; 启用开机自启动。优先计划任务，失败则询问是否回退到 Startup 文件夹。
CLX_AutostartEnable()
{
    global CLX_AUTOSTART_TASK
    exePath := A_WorkingDir "\CapsLockX.exe"
    if (!FileExist(exePath)) {
        MsgBox, 16, % t("CapsLockX"), % t("找不到 CapsLockX.exe，无法设置开机自动启动。")
        return false
    }

    ; /rl highest 让 CLX 直接以管理员权限启动，避免每次登录都弹 UAC
    ; （T_AskRunAsAdmin 默认开启时，普通启动方式每次登录都会弹窗）。
    q := """"
    args := "/create /tn " q CLX_AUTOSTART_TASK q " /tr " q exePath q " /sc onlogon /rl highest /f"
    CLX_AutostartSchtasks(args)

    if (CLX_AutostartTaskExists()) {
        TrayTip, % t("CapsLockX"), % t("已设置开机自动启动（计划任务，以管理员权限运行）。")
        return true
    }

    MsgBox, 4, % t("CapsLockX"), % t("创建计划任务失败（通常是未授予管理员权限）。`n是否改用 Startup 文件夹方式？该方式在每次登录时可能弹出 UAC 提示。")
    IfMsgBox, Yes
    {
        return CLX_MakeStartup()
    }
    return false
}

; 关闭开机自启动：两种方式都清理掉，避免残留导致登录时启动两次。
CLX_AutostartDisable()
{
    global CLX_AUTOSTART_TASK
    ok := true

    cmdPath := CLX_AutostartCmdPath()
    if (FileExist(cmdPath)) {
        FileDelete, %cmdPath%
        if (ErrorLevel)
            ok := false
    }

    if (CLX_AutostartTaskExists()) {
        q := """"
        CLX_AutostartSchtasks("/delete /tn " q CLX_AUTOSTART_TASK q " /f")
        if (CLX_AutostartTaskExists())
            ok := false
    }

    if (ok) {
        TrayTip, % t("CapsLockX"), % t("已取消开机自动启动。")
    } else {
        MsgBox, 16, % t("CapsLockX"), % t("取消开机自动启动失败，请检查是否已授予管理员权限。")
    }
    return ok
}

CLX_AutostartToggle()
{
    if (CLX_AutostartEnabled())
        return CLX_AutostartDisable()
    return CLX_AutostartEnable()
}

; 回退方式：写入用户 Startup 文件夹。不需要管理员权限，但 CLX 请求提权时
; 每次登录都会弹 UAC，所以只作为计划任务失败后的备选。
CLX_MakeStartup()
{
    startCMDPath := CLX_AutostartCmdPath()
    content = cd "%A_WorkingDir%" && start "" CapsLockX.exe
    FileDelete, %startCMDPath%
    FileAppend, echo off`r`n, %startCMDPath%
    FileAppend, %content%, %startCMDPath%
    if (!FileExist(startCMDPath)) {
        MsgBox, 16, % t("CapsLockX"), % t("写入 Startup 文件夹失败。")
        return false
    }
    TrayTip, % t("CapsLockX"), % t("已在Startup文件夹添加CLX的开机自启动，请确认。")
    return true
}
