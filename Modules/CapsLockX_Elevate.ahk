
global T_AskRunAsAdmin := CapsLockX_Config("Core", "T_AskRunAsAdmin", 0, "请求管理员权限（权限受限时，权限受限，例如鼠标模拟等功能无法正常运行，默认请求提升权限，如果不需要管理权限下的功能，可以改为0）")
AskRunAsAdminIfNeeded()
Return
AskRunAsAdminIfNeeded(){
    if (T_AskRunAsAdmin)
        AskRunAsAdmin()
}
; 管理员模式运行
AskRunAsAdmin(){
    full_command_line := DllCall("GetCommandLine", "str")
    if (!A_IsAdmin And !RegExMatch(full_command_line, " /restart(?!\S)")){
        TrayTip, CapsLockX 权限受限, 当前权限受限，例如鼠标模拟等功能无法正常运行，正在请求提升权限。
        try {
            if A_IsCompiled {
                Run *RunAs "%A_ScriptFullPath%" /restart, "%A_WorkingDir%"
            } else {
                Run *RunAs "%A_AhkPath%" /restart "%A_ScriptFullPath%", "%A_WorkingDir%"
            }
        }
        ; ExitApp
    }
}
