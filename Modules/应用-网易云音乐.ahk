
SetDefaultMouseSpeed 0
Return
#IfWinActive ahk_class OrpheusBrowserHost ahk_exe cloudmusic.exe
    
    ^f::
    	CoordMode, Mouse, Relative
    	Click 425, 27
    	Return

;debug