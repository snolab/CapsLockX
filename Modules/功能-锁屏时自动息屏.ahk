Return

; Run rundll32.exe user32.dll LockWorkStation, , Hide
;    cmdScreenOff := "cmd /c start """" powershell (Add-Type '[DllImport(\""user32.dll\"")]^public static extern int SendMessage(int hWnd, int hMsg, int wParam, int lParam);' -Name a -Pas)::SendMessage(-1, 0x0112, 0xF170, 2)"
;    SendMessage, 0x112, 0xF170, 2, , Program Manager
~#l::
    SendMessage, 0x112, 0xF170, 2, , Program Manager
    Sleep 1000
    SendMessage, 0x112, 0xF170, 2, , Program Manager
    ;    Run %cmdScreenOff%, , Hide
Return

; 创意参考： [按下Win+L键，玄机来了]( http://m.cfan.com.cn/article/105095 )
