SetDefaultMouseSpeed 0
Return
#IfWinActive ahk_class OrpheusBrowserHost ahk_exe cloudmusic.exe
    ^!F12:: ExitApp ; 退出脚本
    
    ^f:: Click 850, 55

;debug
;F12:: ExitApp