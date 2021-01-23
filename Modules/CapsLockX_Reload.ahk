; ========== CapsLockX ==========
; Note：Save as UTF-8 with BOM please
; 名称：CapsLockX 重启键
; 作者：snomiao (snomiao@gmail.com)
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2021.01.20
; 版权：Copyright © 2018-2020 Snowstar Laboratory. All Rights Reserved.
; LICENCE: GNU GPLv3
; ========== CapsLockX ==========

return
; 硬重启键
^!\::
    ; Run CapsLockX.ahk, %A_WorkingDir%
    Run CapsLockX.exe, %A_WorkingDir%
ExitApp
Return

; 退出键、结束键
^!+\:: ExitApp
