; ========== CapsLockX ==========
; 名称：快速输入各种时间戳和随机数
; 描述：快速输入各种时间戳和随机数
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.06.27
; 版权：Copyright © 2017-2022 Snowstar Laboratory. All Rights Reserved.
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
SNODateStringGenerate()
{
    FormatTime, TimeString, , (yyyyMMdd)
    return TimeString
}
ISODateStringGenerate()
{
    FormatTime, TimeString, , yyyy-MM-dd
    return TimeString
}
TimeStringGenerate()
{
    FormatTime, TimeString, , (yyyyMMdd.HHmmss)
    return TimeString
}
DateTimeStringGenerate()
{
    FormatTime, TimeString, , yyyy-MM-dd HH:mm:ss
    return TimeString
}
QuickTextInput(str)
{
    SendInput {Text}%str%
}

#if
    
:*?:#D#:: ; 日期输入：如 (20220217)
QuickTextInput(ISODateStringGenerate())
return

:*?:#F#:: ; 日期输入：如 2022-02-17
QuickTextInput(SNODateStringGenerate())
return

:*?:#T#:: ; 时间输入：(20220217.220717)
QuickTextInput(TimeStringGenerate())
return

:*?:#DT#:: ; 日期时间输入：2022-02-17 22:07:33
QuickTextInput(DateTimeStringGenerate())
return

:*?:#NPW#:: ; 随机输入数字密码如： 7500331260229289
QuickTextInput(GenPassword("0123456789", 16))
return

:*?:#HEX#:: ; 随机输入数字字母密码如：
QuickTextInput(GenPassword("0123456789ABCDEF", 16))
return

:*?:#HEXL#:: ; 随机输入小写16进制如：
QuickTextInput(GenPassword("0123456789abcdef", 16))
return

:*?:#PW#:: ; 随机输入数字字母密码如： yyCTCNYodECTLr2h
QuickTextInput(GenPassword("123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz", 16))
return

:*?:#WPW#:: ; 随机输入数字字母密码如： FtD5BB1m5H98eY7Y
QuickTextInput(GenPassword("123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz", 16))
return

:*?:#SPW#:: ; 随机输入数字字母符号密码如：KO?C[D_>!c$sQ-|7]
QuickTextInput(GenPassword("!""#$%&\'()*+, -./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_``abcdefghijklmnopqrstuvwxyz{|}~", 16))
return
