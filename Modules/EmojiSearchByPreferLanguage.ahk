; ========== CapsLockX ==========
; 描述：Search emoji by prefer language
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v1.0.0
; 版权：Copyright © 2017-2022 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========

Return
#If

; ref: [(35 封私信 / 17 条消息) 输入法智能切换中英文，用autohotkey如何实现？ - 知乎]( https://www.zhihu.com/question/41446565 )
SetInputLang(languageIdentifier) {
    WinExist("A")
    ControlGetFocus, CtrlInFocus
    SendMessage, 0x50, 0, % languageIdentifier, %CtrlInFocus%
}

ToggleToEnglish()
{
    SetInputLang(0x0409)
}

emojiSearch()
{
    ToggleToEnglish()
    SendEvent #.
}
#.::  emojiSearch()
