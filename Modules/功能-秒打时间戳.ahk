; ========== CapsLockX ==========
; 名称：快速输入各种时间戳
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.06.27
; 版权：Copyright © 2018-2020 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========

If(CapsLockX){
    help := "秒打时间戳" . "`n"
    FormatTime, TimeString, , yyyy-MM-dd
    help .= "| /d`t| " . TimeString . "`n"
    FormatTime, DateString, , (yyyyMMdd)
    help .= "| /tf`t| " . DateString . "`n"
    FormatTime, TimeString, , (yyyyMMdd.HHmmss)
    help .= "| /t`t| " . TimeString . "`n"
    FormatTime, TimeString, , (HHmm)
    help .= "| /s`t| " . TimeString . "`n"
    FormatTime, TimeString, , vyyyy.MM.dd
    help .= "| /v`t| " . TimeString . "`n"
    FormatTime, TimeString, , vyyyy.MM.dd
    help .= "| /v`t| " . TimeString
    CapslockXAddHelp(help)
}
Return

::/date::
    FormatTime, TimeString, , yyyy-MM-dd
    SendInput %TimeString%
    Return
::/d::
    FormatTime, DateString, , (yyyyMMdd)
    SendInput %DateString%
    Return
::/tick::
    SendInput % A_TickCount
    Return
::/t::
    FormatTime, TimeString, , (yyyyMMdd.HHmmss)
    SendInput %TimeString%
    Return
::/dt::
    FormatTime, TimeString, , yyyy-MM-dd HH:mm:ss
    SendInput %TimeString%
    Return
::/v::
    FormatTime, TimeString, , vyyyy.MM.dd
    SendInput %TimeString%
    Return