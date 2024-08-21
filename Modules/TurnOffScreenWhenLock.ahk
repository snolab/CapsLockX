global TurnOffScreenWhenLock := CLX_Config("TurnOffScreenWhenLock", "TurnOffScreenWhenLock", "0", t("按 Win + L 锁定电脑时，立即关闭屏幕，默认不启用，你可以按 Win+Alt+L 达到该效果"))

Return

; Win+Shift+L 锁定电脑 + 关闭屏幕
+#l::
    Run rundll32.exe user32.dll LockWorkStation, , Hide
    SendMessage, 0x112, 0xF170, 2, , Program Manager
    Sleep 1000
    SendMessage, 0x112, 0xF170, 2, , Program Manager
    Sleep 1000
    SendMessage, 0x112, 0xF170, 2, , Program Manager
    Return
    
; Win+Alt+L  仅关闭屏幕，按任意键可解除
!#l::
    SendMessage, 0x112, 0xF170, 2, , Program Manager
    Sleep 1000
    SendMessage, 0x112, 0xF170, 2, , Program Manager
    Return

#if TurnOffScreenWhenLock

; Run rundll32.exe user32.dll LockWorkStation, , Hide
;    cmdScreenOff := "cmd /c start """" powershell (Add-Type '[DllImport(\""user32.dll\"")]^public static extern int SendMessage(int hWnd, int hMsg, int wParam, int lParam);' -Name a -Pas)::SendMessage(-1, 0x0112, 0xF170, 2)"
;    SendMessage, 0x112, 0xF170, 2, , Program Manager

~#l::
    SendMessage, 0x112, 0xF170, 2, , Program Manager
    Sleep 1000
    SendMessage, 0x112, 0xF170, 2, , Program Manager
    Return

    ; ref： [按下Win+L键，玄机来了]( http://m.cfan.com.cn/article/105095 )
