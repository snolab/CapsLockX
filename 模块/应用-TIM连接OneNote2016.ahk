SetTitleMatchMode RegEx
Return

#IfWinActive ahk_class TXGuiFoundation ahk_exe TIM.exe
	$!n::
		CoordMode, Mouse, Relative
		Clipboard := ""
		Click 548, 321
		Send ^a^c
		ClipWait

		Send #n
		WinWaitActive, .*- OneNote ahk_class Framework\:\:CFrame
		;MsgBox, asdf
		Send ^v
		Return