
SetDefaultMouseSpeed 0
Return
#IfWinActive ahk_class OrpheusBrowserHost ahk_exe cloudmusic.exe
    ^!F12:: ExitApp ; 退出脚本
    
    ^f::
    	CoordMode, Mouse, Relative
    	Click 425, 27
    	Return

;debug
;F12:: ExitApp