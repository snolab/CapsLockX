;
; 模块：在增强模式下，用另一套鼠标指针
; 贡献者： @冰封 QQ: 124702759
;
;鼠标特征值
Return

SetCursor(code, curFile)
{
    ;加载CapsLock增强的标指针
    hIcon := DllCall("LoadCursorFromFile", "Str", curFile)
    DllCall("SetSystemCursor", "UInt", hIcon, "Int", code)
}

UpdateCapsCursor(s)
{
    if (!s) {
        ;恢复默认指针
        SPI_SETCURSORS := 0x57
        DllCall( "SystemParametersInfo", "UInt", SPI_SETCURSORS, "UInt", 0, "UInt", 0, "UInt", 0)
        Return
    }
    
    PATH_CURSOR := "数据/cursor"
    ; OCR_APPSTARTING := 32650
    SetCursor(32650, PATH_CURSOR "/APPSTARTING.cur") ;AppStarting.cur")
    ; OCR_NORMAL      := 32512
    SetCursor(32512, PATH_CURSOR "/NORMAL.cur") ;正常选择.cur")
    ; OCR_CROSS       := 32515
    SetCursor(32515, PATH_CURSOR "/CROSS.cur") ;精确选择.cur")
    ; OCR_HAND        := 32649
    SetCursor(32649, PATH_CURSOR "/HAND.cur") ;链接选择.cur")
    ; OCR_HELP        := 32651
    SetCursor(32651, PATH_CURSOR "/HELP.cur") ;帮助选择.cur")
    ; OCR_IBEAM       := 32513
    SetCursor(32513, PATH_CURSOR "/IBEAM.cur") ;文本选择.cur")
    ; OCR_NO          := 32648
    SetCursor(32648, PATH_CURSOR "/NO.cur") ;不可用.cur")
    ; OCR_SIZEALL     := 32646
    SetCursor(32646, PATH_CURSOR "/SIZEALL.cur") ;移动.cur")
    ; OCR_SIZENESW    := 32643
    SetCursor(32643, PATH_CURSOR "/SIZENESW.cur") ;沿对角线调整大小2.cur")
    ; OCR_SIZENS      := 32645
    SetCursor(32645, PATH_CURSOR "/SIZENS.cur") ;垂直调整大小.cur")
    ; OCR_SIZENWSE    := 32642
    SetCursor(32642, PATH_CURSOR "/SIZENWSE.cur") ;沿对角线调整大小1.cur")
    ; OCR_SIZEWE      := 32644
    SetCursor(32644, PATH_CURSOR "/SIZEWE.cur") ;水平调整大小.cur")
    ; OCR_UP          := 32516
    SetCursor(32516, PATH_CURSOR "/UP.cur") ;候选.cur")
    ; OCR_WAIT        := 32514
    SetCursor(32514, PATH_CURSOR "/WAIT.cur") ;忙.cur")
}