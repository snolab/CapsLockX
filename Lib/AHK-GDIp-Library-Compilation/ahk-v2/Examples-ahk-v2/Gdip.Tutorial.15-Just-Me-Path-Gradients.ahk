; https://autohotkey.com/board/topic/29449-gdi-standard-library-145-by-tic/page-65
; by just_me 
; adapted to ahk v2 by Marius Șucan

#Include ../Gdip_All.ahk
Global w, h
W := 200
H := 200

GdipToken := Gdip_Startup()

Gui1 := GuiCreate("+LastFound +AlwaysOnTop +OwnDialogs")
hGui := Gui1.hwnd
Gui1.Title := "Path gradients"

ctrlPicA := Gui1.Add("Picture" , "w" w " h" h)
hpic := ctrlPicA.hwnd
ScaleX := 0.0 ; focus scale for x-axis (0.0 - 1.0)
ScaleY := 0.0 ; focus scale for y-axis (0.0 - 1.0)
BlendFocus := 0.5 ; blend focus (0.0 - 1.0)
PathGradientBrush(hpic, BlendFocus, ScaleX, ScaleY)
ctrlPicB := Gui1.Add("Picture" , "w" w " h" h)
hpic := ctrlPicB.hwnd
ScaleX := 0.75 ; focus scale for x-axis (0.0 - 1.0)
ScaleY := 0.75 ; focus scale for y-axis (0.0 - 1.0)
BlendFocus := 1.0 ; blend focus (0.0 - 1.0)
PathGradientBrush(hpic, BlendFocus, ScaleX, ScaleY)
Gui1.Show("AutoSize")
Return

GuiClose:
GuiEscape:
Gdip_ShutDown(GdipToken)
ExitApp

PathGradientBrush(ctrl, BlendFocus, ScaleX, ScaleY) {
   SS_BITMAP    := 0xE
   SS_ICON      := 0x3
   STM_SETIMAGE := 0x172
   IMAGE_BITMAP := 0x0

   PBitMap := Gdip_CreateBitmap(W, H)
   PGraphics := Gdip_GraphicsFromImage(PBitMap)
   Gdip_SetSmoothingMode(PGraphics, 4)
   PPath := Gdip_CreatePath(PGraphics)
   Gdip_AddPathRectangle(PPath, 0, 0, W, H)
   PBrush := Gdip_PathGradientCreateFromPath(PPath)
   Gdip_PathGradientSetCenterPoint(PBrush, W / 2, H / 2)
   Gdip_PathGradientSetCenterColor(PBrush, 0xFFFFFFFF)
   Gdip_PathGradientSetSurroundColors(PBrush, 0xFF202090)
   Gdip_PathGradientSetSigmaBlend(PBrush, BlendFocus)
   Gdip_PathGradientSetLinearBlend(PBrush, BlendFocus)
   Gdip_PathGradientSetFocusScales(PBrush, ScaleX, ScaleY)
   Gdip_FillPath(PGraphics, PBrush, PPath)
   HBitmap := Gdip_CreateHBITMAPFromBitmap(PBitMap, 0x00FFFFFF)
   Gdip_DeleteBrush(PBrush)
   Gdip_DeletePath(PPath)
   Gdip_DeleteGraphics(PGraphics)
   Gdip_DisposeImage(PBitmap)
   ; Set control styles
   ControlSetStyle("-" SS_ICON, ctrl)
   ControlSetStyle("+" SS_BITMAP, ctrl)
   ; Assign the bitmap
   SendMessage(STM_SETIMAGE, IMAGE_BITMAP, HBitmap, ctrl)
   ; Done!
   ToolTip(ctrl)
   DeleteObject(HBitmap)
}

Esc::
ExitApp
Return