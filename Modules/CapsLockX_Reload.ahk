; 软重启键
; !F12:: Reload

return
; 硬重启键
^!\::
    ; Run CapsLockX.ahk, %A_WorkingDir%
    Run CapsLockX.exe, %A_WorkingDir%
ExitApp
Return

; 退出键、结束键
^!+\:: ExitApp
