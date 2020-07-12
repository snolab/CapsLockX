
CoordMode, Pixel, Screen
CoordMode, Mouse, Screen

DllCall("Shcore.dll\SetProcessDpiAwareness", "UInt", 2)

`::
	MouseGetPos, X, Y, Win, Ctrl
	;PixelGetColor, OutputVar, X, Y [, Alt|Slow|RGB]
	;PixelGetColor, pColor, X, Y [, Alt|Slow|RGB]
	PixelGetColor, pColor, X, Y, RGB
	ToolTip % pColor
	Return
F12::ExitApp