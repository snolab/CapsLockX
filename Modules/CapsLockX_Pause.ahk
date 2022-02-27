; ========== CapsLockX ==========
; Note：Save as UTF-8 with BOM please
; 名称：CapsLockX 暂停键与自动暂停功能
; 作者：snomiao (snomiao@gmail.com)
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2021.01.20
; 版权：Copyright © 2017-2022 Snowstar Laboratory. All Rights Reserved.
; LICENCE: GNU GPLv3
; ========== CapsLockX ==========

; TODO: 显示器状态检测

; return
; ; 思路1 检测POWERBROADCAST消息 -- fail
; OnMessage(0x218, "WM_POWERBROADCAST")
; This checks for the message 0x218 which is a message sent by windows
; on some power conditions (shutdown, resume, wake and so on)
; and if the message has been sent then it will execute the function that
; we set up below.
return

; WM_POWERBROADCAST(wParam, lParam){
;     ; wParam will contain a code depending on the power state
;     ; PBT_APMRESUMESTANDBY (6)
;     ; PBT_APMRESUMESUSPEND (7)
;     ; PBT_APMRESUMEAUTOMATIC (18) etc.
;     ; there are plenty more

;     msgbox % wParam
;     if (wParam = 6 || wParam = 7 || wParam = 18){
;         ;    we can execute some cool code here
;     }
; }

; OnMessage(0x218, "OnMessage_WM_POWERBROADCAST")
; OnMessage_WM_POWERBROADCAST(wParam, lParam){
;     MsgBox, %wParam%
;     If (wParam == 0)
;         Return 0x424D5144
;     Return 0
; }

; ; 思路2 注册电源通知事件 [How to detect monitor off OR laptop lid closed - Ask for Help - AutoHotkey Community](https://autohotkey.com/board/topic/16982-how-to-detect-monitor-off-or-laptop-lid-closed/)
; Gui, lid-detector:New [, Options, Title]
; DllCall("RegisterPowerSettingNotification", "UInt", WinExist("lid-detector"), "str", "GUID_MONITOR_POWER_ON")
; MsgBox %Errorlevel%

; 思路3 检查设备文件
; [How to detect the power state of LCD display? - Ask for Help - AutoHotkey Community](https://autohotkey.com/board/topic/54195-how-to-detect-the-power-state-of-lcd-display/)

; test3(){
;     SendMessage, 0x112, 0xF170, 2, , Program Manager

;     Sleep 2000
;     hDisp := DllCall("CreateFile", "Str", "\\.\LCD", "Uint", 0xC0000000, "Uint", 0x3, "Uint", 0, "Uint", 0x3, "Uint", 0, "Uint", 0)
;     DllCall("GetDevicePowerState", "UInt", hDisp, "IntP", stat)
;     DllCall("CloseHandle", "Uint", hDisp)
;     MsgBox % stat

;     Send {Esc}
;     Sleep 2000

;     hDisp := DllCall("CreateFile", "Str", "\\.\LCD", "Uint", 0xC0000000, "Uint", 0x3, "Uint", 0, "Uint", 0x3, "Uint", 0, "Uint", 0)
;     DllCall("GetDevicePowerState", "UInt", hDisp, "IntP", stat)
;     DllCall("CloseHandle", "Uint", hDisp)
;     MsgBox % stat

; }
; test4(){
;     hDisp := DllCall("CreateFile", "Str", "\\.\LCD", "Uint", 0xC0000000, "Uint", 0x3, "Uint", 0, "Uint", 0x3, "Uint", 0, "Uint", 0)
;     DllCall("GetDevicePowerState", "UInt", hDisp, "IntP", stat)
;     DllCall("CloseHandle", "Uint", hDisp)
;     MsgBox % stat
; }
#if
    
^!Home::
    CapsLockX_Paused := 0
    if(CapsLockX_Paused) {
        TrayTip, 暂停, CapsLockX 已暂停
    } else {
        TrayTip, 暂停, CapsLockX 已恢复
    }
Return

^!End::
    CapsLockX_Paused := 1
    if(CapsLockX_Paused) {
        TrayTip, 暂停, CapsLockX 已暂停
    } else {
        TrayTip, 暂停, CapsLockX 已恢复
    }
Return
