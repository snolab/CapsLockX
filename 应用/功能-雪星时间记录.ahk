;SetWorkingDir, %APPDATA%\雪星时间记录\
global LastWinTitle

#Persistent
	;SetTimer, markTime, 60000
	SetTimer, markTime, 200
	return

markTime:
	WinGetTitle WinTitle, A
	If (LastWinTitle != WinTitle){
		LastWinTitle := WinTitle

    	FormatTime, strTime, , [yyyy-MM-dd HH:mm]
		log := strTime . WinTitle
		FileAppend, %log%`n, WinTitle.log
	}
	Return


#i::
	FileAppend, Another line.`n, time.log
	Return


^F12::
	ExitApp