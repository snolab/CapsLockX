; ========== CapsLockX ==========
; 名称：快速输入各种时间戳
; 作者：snomiaou
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.06.27
; 版权：Copyright © 2017-2021 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========

CapsLockX_AppendHelp( CapsLockX_LoadHelpFrom(CapsLockX_THIS_MODULE_HELP_FILE_PATH))

Return

GenPassword(Chars, Length := 16)
{
    Min := 1
    Max := StrLen(chars)
    pw := ""
    Loop %Length% {
        Random StartPos, Min, Max
        pw := pw SubStr(Chars, StartPos, 1)
    }
    Return pw
}
DateInput()
{
    FormatTime, TimeString, , (yyyyMMdd)
    SendInput {Text}%TimeString%
}
TimeInput()
{
    FormatTime, TimeString, , (yyyyMMdd.HHmmss)
    SendInput {Text}%TimeString%
}
DateTimeInput()
{
    FormatTime, TimeString, , yyyy-MM-dd HH:mm:ss
    SendInput {Text}%TimeString%
}

#if
    
:*:#D#::
    DateInput()
return
:*:#T#::
    TimeInput()
return
:*:#DT#::
    DateTimeInput()
return
:*:#NPW#::
    pw:= GenPassword("0123456789", 16)
    SendInput {Text}%pw%
return
:*:#PW#::
    pw:= GenPassword("123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz", 16)
    SendInput {Text}%pw%
return
:*:#WPW#::
    pw:= GenPassword("123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz", 16)
    SendInput {Text}%pw%
return
:*:#SPW#::
    pw:= GenPassword("!""#$%&\'()*+, -./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_``abcdefghijklmnopqrstuvwxyz{|}~", 16)
    SendInput {Text}%pw%
return
