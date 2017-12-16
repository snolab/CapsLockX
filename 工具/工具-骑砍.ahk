SetTitleMatchMode RegEx
SendMode, Input
SetTimer shoot, 1000
SetTimer shoot, Off

Return

$~`::
	MouseGetPos, x, y
	Click, x, y
	Sleep, 16
	Click 1939, 1781, 10
	Sleep, 16
	MouseMove, x, y, 0

	Return

$t:: SetTimer shoot, On
$y:: SetTimer shoot, Off
$F12:: ExitApp

shoot:
	Send {LButton}
	Return