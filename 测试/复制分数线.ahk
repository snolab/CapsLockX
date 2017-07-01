
F12:: ExitApp

`::
	Clipboard := ""
	MouseClickDrag, L, 0, 0, -1660, -1080, 0, R
	MouseMove, 1660, 1080, 0, R

	Send ^c
	Send !{Tab}
	Sleep, 128
	Send ^g
	; ClipWait, 0.5, 1
	Return

^g::
	Send {AppsKey}m{Enter}
	Send ^{Down 2}{Down 2}
	Return
