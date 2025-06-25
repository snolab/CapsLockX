; gdi+ ahk tutorial 9 written by tic (Tariq Porter)
; Requires Gdip.ahk either in your Lib folder as standard library or using #Include
;
; Example to create a progress bar in a standard gui

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

; Before we start there are some design elements we must consider.
; We can either make the script faster. Creating the bitmap 1st and just write the new progress bar onto it every time and updating it on the gui
; and dispose it all at the end/ This will use more RAM
; Or we can create the entire bitmap and write the information to it, display it and then delete it after every occurence
; This will be slower, but will use less RAM and will be modular (can all be put into a function)
; I will go with the 2nd option, but if more speed is a requirement then choose the 1st

; I am first creating a slider, just as a way to change the percentage on the progress bar
; The progress bar needs to be added as a picture, as all we are doing is creating a gdi+ bitmap and setting it to this control
; Note we have set the 0xE style for it to accept an hBitmap later and also set a variable in order to reference it (could also use hwnd)
; We will set the initial image on the control before showing the gui

;AHK v1
;Gui, 1: -DPIScale
;Gui, 1: Add, Slider, x10 y10 w400 Range0-100 vPercentage gSlider Tooltip, 50
;Gui, 1: Add, Picture, x10 y+30 w400 h50 0xE vProgressBar
;GoSub Slider
;Gui, 1: Show, AutoSize, Example 9 - gdi+ progress bar

Gui1 := GuiCreate("-DPIScale")
Gui1.OnEvent("Close", "Gui_Close")
SliderObj := Gui1.Add("Slider", "x10 y10 w400 Range0-100 vPercentage Tooltip", 50)
SliderObj.OnEvent("Change", "Slider_Change")
PictureObj := Gui1.Add("Picture", "x10 y+30 w400 h50 0xE vProgressBar")
Gdip_SetProgress(PictureObj, 50, 0xff0993ea, 0xffbde5ff, "50`%")
Gui1.Title := "Example 9 - gdi+ progress bar"
Gui1.Show("AutoSize")
Return

;#######################################################################

; This subroutine is activated every time we move the slider as I used gSlider in the options of the slider
;AHK v1
;Slider:
;	Gui, 1: Default
;	Gui, 1: Submit, NoHide
;	Gdip_SetProgress(ProgressBar, Percentage, 0xff0993ea, 0xffbde5ff, Percentage "`%")
;Return

Slider_Change(GuiCtrlObj, Info)
{
	Gdip_SetProgress(GuiCtrlObj.Gui["ProgressBar"], GuiCtrlObj.Value, 0xff0993ea, 0xffbde5ff, GuiCtrlObj.Value "`%")
}

;#######################################################################

Gdip_SetProgress(ByRef Variable, Percentage, Foreground, Background:=0x00000000, Text:="", TextOptions:="x0p y15p s60p Center cff000000 r4 Bold", Font:="Arial")
{
	; We first want the hwnd (handle to the picture control) so that we know where to put the bitmap we create
	; We also want to width and height (posw and Posh)

	;AHK v1
	;GuiControlGet, Pos, Pos, Variable
	;GuiControlGet, hwnd, hwnd, Variable
	Posw := Variable.Pos.w
	Posh := Variable.Pos.h
	hwnd := Variable.hwnd

	; Create 2 brushes, one for the background and one for the foreground. Remember this is in ARGB
	pBrushFront := Gdip_BrushCreateSolid(Foreground), pBrushBack := Gdip_BrushCreateSolid(Background)
	; Create a gdi+ bitmap the width and height that we found the picture control to be
	; We will then get a reference to the graphics of this bitmap
	; We will also set the smoothing mode of the graphics to 4 (Antialias) to make the shapes we use smooth
	pBitmap := Gdip_CreateBitmap(Posw, Posh), G := Gdip_GraphicsFromImage(pBitmap), Gdip_SetSmoothingMode(G, 4)
	
	; We will fill the background colour with out background brush
	; x = 0, y = 0, w = Posw, h = Posh
	Gdip_FillRectangle(G, pBrushBack, 0, 0, Posw, Posh)
	
	; We will then fill a rounded rectangle with our other brush, starting at x = 4 and y = 4
	; The total width is now Posw-8 as we have slightly indented the actual progress bar
	; The last parameter which is the amount the corners will be rounded by in pixels has been made to be 3 pixels...
	; ...however I have made it so that they are smaller if the percentage is too small as it cannot be rounded by that much
	Gdip_FillRoundedRectangle(G, pBrushFront, 4, 4, (Posw-8)*(Percentage/100), Posh-8, (Percentage >= 3) ? 3 : Percentage)
	
	; As mentioned in previous examples, we will provide Gdip_TextToGraphics with the width and height of the graphics
	; We will then write the percentage centred onto the graphics (Look at previous examples to understand all options)
	; I added an optional text parameter at the top of this function, to make it so you could write an indication onto the progress bar
	; such as "Finished!" or whatever, otherwise it will write the percentage to it
	Gdip_TextToGraphics(G, (Text != "") ? Text : Round(Percentage) "`%", TextOptions, Font, Posw, Posh)
	
	; We then get a gdi bitmap from the gdi+ one we've been working with...
	hBitmap := Gdip_CreateHBITMAPFromBitmap(pBitmap)
	; ... and set it to the hwnd we found for the picture control
	SetImage(hwnd, hBitmap)
	
	; We then must delete everything we created
	; So the 2 brushes must be deleted
	; Then we can delete the graphics, our gdi+ bitmap and the gdi bitmap
	Gdip_DeleteBrush(pBrushFront), Gdip_DeleteBrush(pBrushBack)
	Gdip_DeleteGraphics(G), Gdip_DisposeImage(pBitmap), DeleteObject(hBitmap)
	Return 0
}

;#######################################################################

Gui_Close(GuiObj) {
GuiClose:
   ExitApp
return
}

ExitFunc(ExitReason, ExitCode)
{
   global
   ; gdi+ may now be shutdown
   Gdip_Shutdown(pToken)
}

