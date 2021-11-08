; ========== CapsLockX ==========
; Note：Save as UTF-8 with BOM please
; 名称：CapsLockX 重启键
; 作者：snomiao (snomiao@gmail.com)
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2021.01.20
; 版权：Copyright © 2017-2021 Snowstar Laboratory. All Rights Reserved.
; LICENCE: GNU GPLv3
; ========== CapsLockX ==========
; tooltip loaded
; WatchFolder(A_WorkingDir "\User\", "CapsLockX_FolderModified", true, 0x08)
WatchFolder(A_WorkingDir "\Modules\", "CapsLockX_FolderModified", true, 0x08) ; chagned
WatchFolder(A_WorkingDir "\User\", "CapsLockX_FolderChanged", true, 0x02 | 0x03 | 0x08) ; delete or add, iguess
; WatchFolder(A_WorkingDir "\Modules\", "CapsLockX_FolderChanged", true, 0x02 | 0x03) ; delete or add
TrayTip CapsLockX 载入成功
#include Modules/WatchFolder/WatchFolder.ahk

return

; 只 reload 不重新编译模块
CapsLockX_FolderModified(Folder, Changes) {
    MsgBox, 4, CapsLockX 重载模块, 检测到配置更改，是否软重载？
    IfMsgBox Yes
    reload
}
CapsLockX_FolderChanged(Folder, Changes)
{
    global CapsLockX_ConfigChangedTickCount
    ; 跳过 CapsLockX 自己改的配置，容差 2-5 秒
    Sleep, 2000
    if ( CapsLockX_ConfigChangedTickCount && A_TickCount - CapsLockX_ConfigChangedTickCount < 5000) {
        return
    }
    
    global T_AutoReloadOnConfigsChange := CapsLockX_Config("Advanced", "T_AutoReloadOnConfigsChange", 0, "用户配置修改保存时自动重载")
    if (CapsLockX_DontReload)
        return
    if(T_AutoReloadOnConfigsChange) {
        TrayTip, CapsLockX 重载模块, 检测到配置更改，正在自动重载。
        CapsLockX_Reload()
    } else {
        MsgBox, 4, CapsLockX 重载模块, 检测到配置更改，是否重载？
        IfMsgBox Yes
        CapsLockX_Reload()
    }
}

#if

^!\:: CapsLockX_Reload() ; CapsLockX_重新启动
~^!+\:: ExitApp ; CapsLockX_退出
