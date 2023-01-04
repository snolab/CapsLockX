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
GenRandomHex(Length){
	; 此处 `` 为转义
    Chars := "0123456789abcdef"
    Min := 1
    Max := StrLen(chars)

	randHex := ""
    Loop %Length% {
	    Random StartPos, Min, Max
	    randHex := randHex SubStr(Chars, StartPos, 1)
    }
    Return randHex
}
GenRandomUUID(){
    Return GenRandomHex(8) "-" GenRandomHex(4) "-" GenRandomHex(4) "-" GenRandomHex(4) "-" GenRandomHex(12)
}
QuickTextInput(str)
{
    SendInput {Text}%str%
}

JapaneseRomajiChar(){
    return GenPassword("xktnmwhrypbdsfg", 1)GenPassword("aeiou", 1)
}
JapaneseRomaji7Char(){
    return JapaneseRomajiChar()JapaneseRomajiChar()JapaneseRomajiChar()JapaneseRomajiChar()JapaneseRomajiChar()JapaneseRomajiChar()JapaneseRomajiChar()
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

:*?:#DPW#:: ; 随机输入2段密码如：ZG1Y9XY-HCSWT71
QuickTextInput(GenPassword("123456789ABCDEFGHJKLMNPQRSTUVWXYZ", 7) "-" GenPassword("123456789ABCDEFGHJKLMNPQRSTUVWXYZ", 7))
return

:*?:#QPW#:: ; 随机输入4段密码如：4428-UW4R-58YS-ALLR
QuickTextInput(GenPassword("123456789ABCDEFGHJKLMNPQRSTUVWXYZ", 4) "-" GenPassword("123456789ABCDEFGHJKLMNPQRSTUVWXYZ", 4) "-" GenPassword("123456789ABCDEFGHJKLMNPQRSTUVWXYZ", 4) "-" GenPassword("123456789ABCDEFGHJKLMNPQRSTUVWXYZ", 4))
return

:*?:#UUID#:: ; 随机输入偽UUID如：345103d0-9de1-d5c6-425f-867dfbf555ea
QuickTextInput(GenRandomUUID())
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

:*?:#JPW#:: ; 随机输入日本語発音密码如：
QuickTextInput( JapaneseRomaji7Char() "-" JapaneseRomaji7Char() )
return
