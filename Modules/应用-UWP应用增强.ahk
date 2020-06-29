If(!CapsLockX)
	ExitApp
Return

clickInWindow(px, py, win="A"){
	CoordMode, Mouse, Screen
	CoordMode, ToolTip, Screen
	WinGetPos, X, Y, W, H, %win%
	if (px < 0) {
		px += W
	}
	if (py < 0) {
		py += H
	}
	px := px + X
	py := py + Y
	Click %px%, %py%
}

#IfWinActive Nextgen Reader ahk_class ApplicationFrameWindow ahk_exe ApplicationFrameHost.exe
    f3:: ^e
    !f:: ^e
    !a::
    	SetKeyDelay, 16, 16
    	clickInWindow(-26, 56)
    	Send {Tab 3}{Enter}
    	Sleep 20
    	Send {Tab}{Down 3}{Enter}
    	Sleep 20
    	Send {Tab 2}
    	Sleep 20
    	Return
    !s:: Send {Click}{Tab 6}{Enter}