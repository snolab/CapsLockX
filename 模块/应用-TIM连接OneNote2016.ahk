SetTitleMatchMode RegEx
Return

#IfWinActive ahk_class TXGuiFoundation ahk_exe TIM.exe
	; ^!F12:: ExitApp ; 退出脚本
	$!n::
		Clipboard := ""
		Click 548, 321
		Send ^a^c
		ClipWait

		Send #n
		WinWaitActive, .*- OneNote ahk_class Framework\:\:CFrame
		;MsgBox, asdf
		Send ^v
		Return