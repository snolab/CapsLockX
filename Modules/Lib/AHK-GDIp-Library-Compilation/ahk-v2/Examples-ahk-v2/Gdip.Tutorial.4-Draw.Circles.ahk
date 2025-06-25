; gdi+ ahk tutorial 4 written by tic (Tariq Porter)
; Requires Gdip.ahk either in your Lib folder as standard library or using #Include
;
; Tutorial to fill the screen with randomly hatched ellipses

#SingleInstance Force
;#NoEnv
;SetBatchLines -1

; Uncomment if Gdip.ahk is not in your standard library
#Include ../Gdip_All.ahk

; Start gdi+
If !pToken := Gdip_Startup()
{
	MsgBox "Gdiplus failed to start. Please ensure you have gdiplus on your system"
	ExitApp
}
OnExit("ExitFunc")

; Get the dimensions of the primary monitor
; these funcs are based off MDMF lib and are now included in the Gdip_All library 
MonitorPrimary := GetPrimaryMonitor()
M := GetMonitorInfo(MonitorPrimary)
WALeft := M.WALeft 
WATop := M.WATop
WARight := M.WARight
WABottom := M.WABottom
WAWidth := M.WARight-M.WALeft
WAHeight := M.WABottom-M.WATop

; Create a layered window (+E0x80000) that is always on top (+AlwaysOnTop), has no taskbar entry or caption
;AHK v1
;Gui, 1: -Caption +E0x80000 +LastFound +AlwaysOnTop +ToolWindow +OwnDialogs
;Gui, 1: Show, NA
Gui1 := GuiCreate("-Caption +E0x80000 +LastFound +AlwaysOnTop +ToolWindow +OwnDialogs")
Gui1.Show("NA")

; Get a handle to this window we have created in order to update it later
hwnd1 := WinExist()

; Create a gdi bitmap with width and height of the work area
hbm := CreateDIBSection(WAWidth, WAHeight)

; Get a device context compatible with the screen
hdc := CreateCompatibleDC()

; Select the bitmap into the device context
obm := SelectObject(hdc, hbm)

; Get a pointer to the graphics of the bitmap, for use with drawing functions
G := Gdip_GraphicsFromHDC(hdc)

; Set the smoothing mode to antialias = 4 to make shapes appear smother (only used for vector drawing and filling)
Gdip_SetSmoothingMode(G, 4)

; Set a timer to draw a new ellipse every 200ms
;AHK v1
;SetTimer DrawCircle, 200
SetTimer "DrawCircle", 200
Return

;#######################################################################

DrawCircle()
{
global
DrawCircle:
; Get a random colour for the background and foreground of hatch style used to fill the ellipse,
; as well as random brush style, x and y coordinates and width/height

/*
;AHK v1
Random, RandBackColour, 0.0, 0xffffffff
Random, RandForeColour, 0.0, 0xffffffff
Random, RandBrush, 0, 53
Random, RandElipseWidth, 1, 200
Random, RandElipseHeight, 1, 200
Random, RandElipsexPos, %WALeft%, % WAWidth-RandElipseWidth
Random, RandElipseyPos, %WATop%, % WAHeight-RandElipseHeight
*/
RandBackColour := Random(0.0, 0xffffffff)
RandForeColour := Random(0.0, 0xffffffff)
RandBrush := Random(0, 53)
RandElipseWidth := Random(1, 200)
RandElipseHeight := Random(1, 200)
RandElipsexPos := Random(WALeft, WAWidth-RandElipseWidth)
RandElipseyPos := Random(WATop, WAHeight-RandElipseHeight)

; Create the random brush
pBrush := Gdip_BrushCreateHatch(RandBackColour, RandForeColour, RandBrush)

; Fill the graphics of the bitmap with an ellipse using the brush created
Gdip_FillEllipse(G, pBrush, RandElipsexPos, RandElipseyPos, RandElipseWidth, RandElipseHeight)

; Update the specified window
UpdateLayeredWindow(hwnd1, hdc, WALeft, WATop, WAWidth, WAHeight)

; Delete the brush as it is no longer needed and wastes memory
Gdip_DeleteBrush(pBrush)
Return
}

;#######################################################################

ExitFunc(ExitReason, ExitCode)
{
   global
   ; Select the object back into the hdc
   SelectObject(hdc, obm)

   ; Now the bitmap may be deleted
   DeleteObject(hbm)

   ; Also the device context related to the bitmap may be deleted
   DeleteDC(hdc)

   ; The graphics may now be deleted
   Gdip_DeleteGraphics(G)

   ; ...and gdi+ may now be shutdown
   Gdip_Shutdown(pToken)
}

Esc::
ExitApp
Return