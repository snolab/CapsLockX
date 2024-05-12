; ========== CapsLockX ==========
; Note：Save as UTF-8 with BOM please
; 名称：CapsLockX 重启键
; 作者：snomiao (snomiao@gmail.com)
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2021.01.20
; 版权：Copyright © 2017-2022 Snowstar Laboratory. All Rights Reserved.
; LICENCE: GNU GPLv3
; ========== CapsLockX ==========
; tooltip loaded
; WatchFolder(A_WorkingDir "\User\", "CLX_FolderModified", true, 0x08)
WatchFolder(A_WorkingDir "\Modules\", "CLX_FolderModified", true, 0x08) ; chagned
WatchFolder(CLX_ConfigDir, "CLX_FolderChanged", true, 0x02 | 0x03 | 0x08) ; delete or add, iguess
; WatchFolder(A_WorkingDir "\Modules\", "CLX_FolderChanged", true, 0x02 | 0x03) ; delete or add
TrayTip % t("CapsLockX 载入成功")
#include Modules/WatchFolder/WatchFolder.ahk
global Reload_DeveloperAsYouInstallMeByGitClone := FileExist(A_WorkingDir "/.git")
return

CLX_JustConfigured()
{
    ; 跳过 CapsLockX 自己改的配置，容差 2-5 秒
    global CLX_ConfigChangedTickCount
    return CLX_ConfigChangedTickCount && A_TickCount - CLX_ConfigChangedTickCount < 5000
}

; 只 reload 不重新编译模块
CLX_FolderModified(Folder, Changes) {
    if ( CLX_JustConfigured() ) {
        return
    }
    ; don reload
    if (CLX_DontReload) {
        return
    }
    ; 只在 git clone 安装方式下询问配置重载
    if (!Reload_DeveloperAsYouInstallMeByGitClone) {
        return
    }
    MsgBox, 4, % t("CapsLockX 重载模块"), 检测到配置更改，是否软重载？
    IfMsgBox Yes
    ; MsgBox, 4, CapsLockX 重载模块, 检测到配置更改，是否软重载？
    ; IfMsgBox Yes
    TrayTip, CapsLockX 重载模块, 检测到配置更改，正在自动软重载。
    sleep 200
    reload
}
CLX_FolderChanged(Folder, Changes)
{
    if ( CLX_JustConfigured() ) {
        return
    }
    ; don reload
    if (CLX_DontReload) {
        return
    }

    global T_AutoReloadOnConfigsChange := CLX_Config("Advanced", "T_AutoReloadOnConfigsChange", 0, "用户配置修改保存时自动重载")

    if (T_AutoReloadOnConfigsChange) {
        TrayTip, CapsLockX 重载模块, 检测到配置更改，正在自动重载。
        sleep 200
        ; CLX_Reload()
        reload
    } else {
        ; 只在 git clone 安装方式下询问重载
        if (!Reload_DeveloperAsYouInstallMeByGitClone) {
            return
        }
        MsgBox, 4, CapsLockX 重载模块, 检测到配置更改，是否重载？
        IfMsgBox Yes
            Reload
        ; CLX_Reload()
    }
}

#if CapsLockXMode

.:: Reload ; CLX_模块重载
+.:: CLX_Reload() ; CLX_重新启动
^+.:: ExitApp ; CLX_退出
