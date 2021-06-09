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

:*:#D#::
    FormatTime, TimeString, , (yyyyMMdd)
    SendInput {Text}%TimeString%
Return
:*:#T#::
    FormatTime, TimeString, , (yyyyMMdd.HHmmss)
    SendInput {Text}%TimeString%
Return
:*:#DT#::
    FormatTime, TimeString, , yyyy-MM-dd HH:mm:ss
    SendInput {Text}%TimeString%
Return