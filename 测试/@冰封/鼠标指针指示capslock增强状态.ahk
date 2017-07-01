;
; 模块：在增强模式下，用另一套鼠标指针
; 贡献者： @冰封 QQ: 124702759
;
;鼠标特征值
OCR_APPSTARTING = 32650
OCR_NORMAL      = 32512
OCR_CROSS       = 32515
OCR_HAND        = 32649
OCR_HELP        = 32651
OCR_IBEAM       = 32513
OCR_NO          = 32648
OCR_SIZEALL     = 32646
OCR_SIZENESW    = 32643
OCR_SIZENS      = 32645
OCR_SIZENWSE    = 32642
OCR_SIZEWE      = 32644
OCR_UP          = 32516
OCR_WAIT        = 32514

RegRead, CapsXState, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation
;ToolTip % CapsXState

SetCursor(code, curFile){
	;加载capslock增强的鼠标指针
	hIcon := DllCall("LoadCursorFromFile","Str", curFile)
	ToolTip % hIcon
	DllCall("SetSystemCursor", "UInt", hIcon, "Int", code)
}
PATH_CURSOR := "cursor"

If(CapsXState){
	SetCursor(OCR_APPSTARTING, PATH_CURSOR "\appstarting.cur") ;AppStarting.cur")
	SetCursor(OCR_NORMAL     , PATH_CURSOR "\normal.cur") ;正常选择.cur")
	SetCursor(OCR_CROSS      , PATH_CURSOR "\cross.cur") ;精确选择.cur")
	SetCursor(OCR_HAND       , PATH_CURSOR "\hand.cur") ;链接选择.cur")
	SetCursor(OCR_HELP       , PATH_CURSOR "\help.cur") ;帮助选择.cur")
	SetCursor(OCR_IBEAM      , PATH_CURSOR "\IBeam.cur") ;文本选择.cur")
	SetCursor(OCR_NO         , PATH_CURSOR "\no.cur") ;不可用.cur")
	SetCursor(OCR_SIZEALL    , PATH_CURSOR "\sizeall.cur") ;移动.cur")
	SetCursor(OCR_SIZENESW   , PATH_CURSOR "\sizenesw.cur") ;沿对角线调整大小2.cur")
	SetCursor(OCR_SIZENS     , PATH_CURSOR "\sizens.cur") ;垂直调整大小.cur")
	SetCursor(OCR_SIZENWSE   , PATH_CURSOR "\sizenwse.cur") ;沿对角线调整大小1.cur")
	SetCursor(OCR_SIZEWE     , PATH_CURSOR "\sizewe.cur") ;水平调整大小.cur")
	SetCursor(OCR_UP         , PATH_CURSOR "\up.cur") ;候选.cur")
	SetCursor(OCR_WAIT       , PATH_CURSOR "\wait.cur") ;忙.cur")
	IsChang=2
}Else{
	;恢复默认指针
	SPI_SETCURSORS := 0x57
	DllCall( "SystemParametersInfo", "UInt", SPI_SETCURSORS, "UInt", 0, "UInt", 0, "UInt", 0)
}