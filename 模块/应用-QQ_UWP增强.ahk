; save as utf8 with bom
SetTitleMatchMode RegEx
; SetKeyDelay, 0, 0

Return

#IfWinActive QQ ahk_class ApplicationFrameWindow ahk_exe ApplicationFrameHost.exe
	$Enter:: Send ^{End}{Enter}
	$!s:: Send ^{End}{Enter}
	$!f::
        CoordMode, Mouse, Relative
        x := 90 * TMouse_DPIRatio
        y := 64 * TMouse_DPIRatio
        y2 := 150 * TMouse_DPIRatio
        Click %x%, %y%
		MouseMove, %x%, %y2%, 0
        Return