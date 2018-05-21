SetTitleMatchMode RegEx
			 			 
Return
#IfWinActive .*- Microsoft Edge ahk_class ApplicationFrameWindow ahk_exe ApplicationFrameHost.exe
	; 对于 UWP 应用，SendEvent 比 SendInput更好用
	!w:: SendEvent {F4}{Tab}{Tab}{Tab}{Space}
	!q:: SendEvent {F6}{Up}{Space}
	!e:: SendEvent {F6}{Down}{Space}