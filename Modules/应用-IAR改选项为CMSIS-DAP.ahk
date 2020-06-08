Return
#IfWinActive ahk_class #32770 ahk_exe IarIdePm.exe
	!o::
		Send {Home}{Tab 2}{Down 9}
		Sleep, 200
		send !d{Up 3}
		Sleep, 200
		Send ^{Tab}!u
		Sleep, 200
		Send {Tab 5}{Down 3}^{Tab 2}!s
		Return