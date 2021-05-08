; ========== CapsLockX ==========
; Note：Save as UTF-8 with BOM please
; 名称：CapsLockX 重启键
; 作者：snomiao (snomiao@gmail.com)
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2021.01.20
; 版权：Copyright © 2017-2021 Snowstar Laboratory. All Rights Reserved.
; LICENCE: GNU GPLv3
; ========== CapsLockX ==========

WatchFolder(A_WorkingDir "\User\", "CapsLockX_UserFolderChanged", true, 0x03 | 0x10)

#include Modules/WatchFolder/WatchFolder.ahk
return

CapsLockX_UserFolderChanged(Folder, Changes){
    global CapsLockX_ConfigChangedTickCount
    ; 跳过 CapsLockX 自己改的配置，容差 2-5 秒
    Sleep, 2000
    if ( CapsLockX_ConfigChangedTickCount && A_TickCount - CapsLockX_ConfigChangedTickCount < 5000)
        return

    global T_AutoReloadOnConfigsChange := CapsLockX_Config("Advanced", "T_AutoReloadOnConfigsChange", 1, "用户配置修改保存时自动重载")
    if(T_AutoReloadOnConfigsChange){
        TrayTip, CapsLockX 重载模块, 检测到配置更改，正在自动重载。
        CapsLockX_Reload()
    }else{
        MsgBox, 4, CapsLockX 重载模块, 检测到配置更改，是否重载？
        IfMsgBox Yes
        CapsLockX_Reload()
    }
}

#if

; 软重启键
^!\:: CapsLockX_Reload()

; 退出键、结束键
~^!+\:: ExitApp
