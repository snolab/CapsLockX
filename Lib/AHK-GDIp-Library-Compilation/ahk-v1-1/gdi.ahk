; GDI library for AHK v1.1.
; made by Marius Șucan
; License: NONE
; Contains trace amounts from:
; GDI class by GeekDuede https://www.autohotkey.com/boards/viewtopic.php?t=5820

; It also contains GDI functions already found in GDI+ library made by Tic.
; https://github.com/marius-sucan/AHK-GDIp-Library-Compilation

; It also contains functions borrowed and modified from Font Library 3 by jBalli.
; https://www.autohotkey.com/boards/viewtopic.php?f=6&t=4379

; Other functions added by Marius Șucan.
; Last update on: mercredi samedi 21 janvier 2023; 21/01/2023
; version: 1.31
;
; The maximum object size is 2GB = 2,147,483,648 bytes
; The largest bitmap allowed is based on its color depth.
; If we want a square, 32-bits, the largest we can get is sqrt(2GB/4) = 23,170 pixels (536.4 mgpx)
; For 24-bits: sqrt(2GB/3) = 26,745 pixels (715.3 mgpx)

Gdi_DrawTextHelper(hDC, hFont, Text, x, y, txtColor, bgrColor:="") {
      ; Transparent background, no color needed
      If (bgrColor="")
         Gdi_SetBgrMode(hDC, 1)
      Else
         Gdi_SetBgrColor(hDC, bgrColor)
      
      Gdi_SetTextColor(hDC, txtColor)
      hOriginalFont := Gdi_SelectObject(hDC, hFont)

      ; Gdi_DrawText(hDC, Text, 0, 0, 900, 900, "LT")
      E := Gdi_TextOut(hDC, Text, x, y)
      Gdi_SelectObject(hDC, hOriginalFont)
      ; ToolTip, % E " , " hdc " , " hFont " , " txtColor " , " bgrColor "`n" x ";" y ";" w ";" h "`n" text, , , 2
      Return E
}

Gdi_TextOut(hDC, Text, x, y) {
      Return DllCall("gdi32\TextOut", "UPtr", hDC, "Int", x, "Int", y, "Str", Text, "Int", StrLen(Text))
}

Gdi_DrawText(hDC, Text, x, y, w, h, flags:=0) {
     ; For the «flags» parameter you can use any of the following
     ; DT_Format flags separated by |. Please omit the DT_ prefix.
     ; e.g., "left|vcenter|wordbreak"

     ; DrawText flags
     Static DT_LEFT := 0x0
                ;-- Aligns text to the left.  Note: This format is used by
                ;   default unless there is an overriding format (Ex: DT_RIGHT).

          , DT_TOP := 0x0
                ;-- Justifies the text to the top of the rectangle.  Note: This
                ;   format is used by default unless there is an overriding
                ;   format (Ex: DT_BOTTOM).

          , DT_CENTER := 0x1
                ;-- Centers text horizontally in the rectangle.

          , DT_RIGHT := 0x2
                ;-- Aligns text to the right.

          , DT_VCENTER := 0x4
                ;-- Centers text vertically.  This value is used only with the
                ;   DT_SINGLELINE format.

          , DT_BOTTOM := 0x8
                ;-- Justifies the text to the bottom of the rectangle.  This
                ;   format is used only with the DT_SINGLELINE format.

          , DT_WORDBREAK := 0x10
                ;-- Breaks words.  Lines are automatically broken between words
                ;   if a word extends past the edge of the rectangle specified
                ;   by the lprc parameter.  A carriage return-line feed sequence
                ;   also breaks the line.

          , DT_SINGLELINE := 0x20
                ;-- Displays text on a single line only.  Carriage returns and
                ;   line feeds do not break the line.

          , DT_EXPANDTABS := 0x40
                ;-- Expands tab characters.  The default number of characters
                ;   per tab is eight.

          , DT_TABSTOP := 0x80
                ;-- Sets tab stops.  The DRAWTEXTPARAMS structure pointed to by
                ;   the lpDTParams parameter specifies the number of average
                ;   character widths per tab stop.

          , DT_NOCLIP := 0x100
                ;-- Draws without clipping.  DrawTextEx is somewhat faster when
                ;   DT_NOCLIP is used.

          , DT_EXTERNALLEADING := 0x200
                ;-- Includes the font external leading in line height.
                ;   Normally, external leading is not included in the height of
                ;   a line of text.

          , DT_CALCRECT := 0x400
                ;-- Determines the width and height of the rectangle.  The text
                ;   is not drawn.

          , DT_NOPREFIX := 0x800
                ;-- Turns off processing of prefix characters.  Normally,
                ;   DrawTextEx interprets the ampersand (&) mnemonic-prefix
                ;   character as a directive to underscore the character that
                ;   follows, and the double-ampersand (&&) mnemonic-prefix
                ;   characters as a directive to print a single ampersand.  By
                ;   specifying DT_NOPREFIX, this processing is turned off.
                ;   Compare with DT_HIDEPREFIX and DT_PREFIXONLY.

          , DT_INTERNAL := 0x1000
                ;-- Uses the system font to calculate text metrics.

          , DT_EDITCONTROL := 0x2000
                ;-- Duplicates the text-displaying characteristics of a
                ;   multiline edit control.  Specifically, the average character
                ;   width is calculated in the same manner as for an Edit
                ;   control, and the function does not display a partially
                ;   visible last line.

          , DT_PATHELLIPSIS  := 0x4000  ;-- Alias
          , DT_PATH_ELLIPSIS := 0x4000
                ;-- For displayed text, replaces characters in the middle of the
                ;   string with ellipses so that the result fits in the
                ;   specified rectangle.  If the string contains backslash (\)
                ;   characters, DT_PATH_ELLIPSIS preserves as much as possible
                ;   of the text after the last backslash.  The string is not
                ;   modified unless the DT_MODIFYSTRING flag is specified.
                ;   Compare with DT_END_ELLIPSIS and DT_WORD_ELLIPSIS.

          , DT_ENDELLIPSIS  := 0x8000  ;-- Alias
          , DT_END_ELLIPSIS := 0x8000
                ;-- For displayed text, replaces the end of a string with
                ;   ellipses so that the result fits in the specified rectangle.
                ;   Any word (not at the end of the string) that goes beyond the
                ;   limits of the rectangle is truncated without ellipses. The
                ;   string is not modified unless the DT_MODIFYSTRING flag is
                ;   specified.  Compare with DT_PATH_ELLIPSIS and
                ;   DT_WORD_ELLIPSIS.

          , DT_MODIFYSTRING := 0x10000
                ;-- Modifies the specified string to match the displayed text.
                ;   This format has no effect unless DT_END_ELLIPSIS or
                ;   DT_PATH_ELLIPSIS is specified.

          , DT_RTLREADING := 0x20000
                ;-- Layout in right-to-left reading order for bidirectional text
                ;   when the font selected into the hdc is a Hebrew or Arabic
                ;   font.  The default reading order for all text is
                ;   left-to-right.

          , DT_WORDELLIPSIS  := 0x40000  ;-- Alias
          , DT_WORD_ELLIPSIS := 0x40000
                ;-- Truncates any word that does not fit in the rectangle and
                ;   adds ellipses.  Compare with DT_END_ELLIPSIS and
                ;   DT_PATH_ELLIPSIS.

          , DT_NOFULLWIDTHCHARBREAK := 0x80000
                ;-- Prevents a line break at a DBCS (double-wide character
                ;   string), so that the line-breaking rule is equivalent to
                ;   SBCS strings.  For example, this can be used in Korean
                ;   windows, for more readability of icon labels.  This format
                ;   has no effect unless DT_WORDBREAK is specified.

          , DT_HIDEPREFIX := 0x100000
                ;-- Ignores the ampersand (&) prefix character in the text.  The
                ;   letter that follows will not be underlined, but other
                ;   mnemonic-prefix characters are still processed.  Compare
                ;   with DT_NOPREFIX and DT_PREFIXONLY.  See the full
                ;   documentation on this flag for examples.

          , DT_PREFIXONLY := 0x200000
                ;-- Draws only an underline at the position of the character
                ;   following the ampersand (&) prefix character.  Does not draw
                ;   any character in the string.  Compare with DT_NOPREFIX and
                ;   DT_HIDEPREFIX.  See the full documentation on this flag for
                ;   examples.

      DT_Format := 0x0
      Loop, Parse, flags, |
      {
            If DT_%A_LoopField% is not Space
               DT_Format|=DT_%A_LoopField%
      }

      obju := []
      VarSetCapacity(Rect, 16, 0)
      NumPut(x, Rect, 0, "uint"), NumPut(y, Rect, 4, "uint")
      NumPut(x + w, Rect, 8, "uint"), NumPut(y + h, Rect, 12, "uint")

      E := DllCall("user32\DrawText", "UPtr", hDC, "Str", Text, "Int", -1, "UPtr", &Rect, "UInt", DT_Format)
      obju.w := NumGet(Rect, 8, "Int")
      obju.h := NumGet(Rect, 12, "Int")
      obju.dll := E
      ; ToolTip, % DT_Format " === " E " == " obju.w "===" obju.h " ===`n" flags , , , 2
      Return obju
}

Gdi_CreateRectRegion(x, y, x2, y2) {
   Return DllCall("gdi32\CreateRectRgn", "Int", x, "Int", y, "Int", x2, "Int", y2, "UPtr")
}

Gdi_CreateRoundRectRegion(x, y, x2, y2, w, h) {
   ; w, h - width and height of the ellipse used to create the rounded corners in device units.
   Return DllCall("gdi32\CreateRoundRectRgn", "Int", x, "Int", y, "Int", x2, "Int", y2, "Int", w, "Int", h, "UPtr")
}

Gdi_CreateEllipticRegion(x, y, x2, y2) {
   Return DllCall("gdi32\CreateEllipticRgn", "Int", x, "Int", y, "Int", x2, "Int", y2, "UPtr")
}

Gdi_CombineRegion(hRgnDst, hRgn1, hRgn2, mode) {
   ; The CombineRgn function combines two regions and stores the result
   ; in a third region. The two regions are combined according to
   ; the specified mode. 

   ; Mode options:
   ; RGN_AND = 1
   ; RGN_OR = 2
   ; RGN_XOR = 3
   ; RGN_DIFF = 4
   ; RGN_COPY = 5

   Return DllCall("gdi32\CombineRgn", "UPtr", hRgnDst, "UPtr", hRgn1, "UPtr", hRgn2, "UInt", mode)
}

Gdi_EqualRegions(hRgn1, hRgn2) {
   ; The EqualRgn function checks the two specified regions to determine
   ; whether they are identical. The function considers two regions
   ; identical if they are equal in size and shape.

   ; Return value
   ; If the two regions are equal, the return value is nonzero.
   ; If the two regions are not equal, the return value is zero.
   ; A return value of ERROR means at least one of the region
   ; handles is invalid.

   Return DllCall("gdi32\EqualRgn", "UPtr", hRgn1, "UPtr", hRgn2)
}

Gdi_FillRegion(hDC, hRgn, hBrush) {
   ; Return value
   ; If the function succeeds, the return value is nonzero.
   ; If the function fails, the return value is zero.
   If (hBrush && hRgn && hDC)
     Return DllCall("gdi32\FillRgn", "UPtr", hDC, "UPtr", hRgn, "UPtr", hBrush)
}

Gdi_PaintRegion(hDC, hRgn) {
   ; The function paints the specified region by using the brush 
   ; currently selected into the device context.

   ; Return value
   ; If the function succeeds, the return value is nonzero.
   ; If the function fails, the return value is zero.

   Return DllCall("gdi32\PaintRgn", "UPtr", hDC, "UPtr", hRgn)
}

Gdi_FrameRegion(hDC, hRgn, hBrush, w, h) {
   ; w = specifies the width, in logical units, of vertical brush strokes.
   ; h = specifies the height, in logical units, of horizontal brush strokes.

   ; Return value
   ; If the function succeeds, the return value is nonzero.
   ; If the function fails, the return value is zero.
   Return DllCall("gdi32\FillRgn", "UPtr", hDC, "UPtr", hRgn, "UPtr", hBrush, "Int", w, "Int", h)
}

Gdi_GetPolyFillMode(hDC) {
   ; Return value
   ; If the function succeeds, the return value specifies the polygon fill mode,
   ; which can be one of the following values.
   ; 1 = ALTERNATE
   ; Selects alternate mode (fills area between odd-numbered and even
   ;-numbered polygon sides on each scan line).

   ; 2 = WINDING
   ; Selects winding mode (fills any region with a nonzero winding value).
 
   ; If an error occurs, the return value is zero.

   Return DllCall("gdi32\GetPolyFillMode", "UPtr", hDC)
}

Gdi_SetPolyFillMode(hDC, mode) {
   ; Mode options:
   ; 1 = ALTERNATE
   ; Selects alternate mode (fills area between odd-numbered and even
   ;-numbered polygon sides on each scan line).

   ; 2 = WINDING
   ; Selects winding mode (fills any region with a nonzero winding value).

   ; Return value
   ; The return value specifies the previous filling mode. If an error 
   ; occurs, the return value is zero.

   ; Remarks
   ; In general, the modes differ only in cases where a complex, overlapping 
   ; polygon must be filled (for example, a five-sided polygon that forms a five-
   ; pointed star with a pentagon in the center). In such cases, ALTERNATE mode 
   ; fills every other enclosed region within the polygon (that is, the points of the 
   ; star), but WINDING mode fills all regions (that is, the points and the pentagon).

   ; When the fill mode is ALTERNATE, GDI fills the area between odd-numbered and
   ; even-numbered polygon sides on each scan line. That is, GDI fills the area
   ; between the first and second side, between the third and fourth side, and so on.

   ; When the fill mode is WINDING, GDI fills any region that has a nonzero winding
   ; value. This value is defined as the number of times a pen used to draw the
   ; polygon would go around the region. The direction of each edge of the polygon is important.

   Return DllCall("gdi32\SetPolyFillMode", "UPtr", hDC, "Int", mode)
}

Gdi_GetRandomRegion(hDC, hRgn) {
   ; hrgn - A handle to a pre-existing region. After the function returns,
   ; this identifies a copy of the current system region. 
   ; The old region identified by hrgn is overwritten.

   ; Return value
   ; If the function succeeds, the return value is 1. If the function fails,
   ; the return value is -1. If the region to be retrieved is NULL,
   ; the return value is 0. If the function fails or the region to be
   ; retrieved is NULL, hrgn is not initialized.
   ; The region returned is in screen coordinates.

   ; Remarks
   ; Note that the system clipping region might not be current because of
   ; window movements. Nonetheless, it is safe to retrieve and use the system
   ; clipping region within the BeginPaint - EndPaint block during WM_PAINT
   ; processing. In this case, the system region is the intersection of the
   ; update region and the current visible area of the window. Any window
   ; movement following the return of GetRandomRgn and before EndPaint will
   ; result in a new WM_PAINT message. Any other use of the SYSRGN flag
   ; may result in painting errors in your application.


   Return DllCall("gdi32\GetRandomRgn", "UPtr", hDC, "UPtr", hRgn, "Int", 4)
}


Gdi_GetMetaRegion(hDC, hRgn) {
   ; The function retrieves the current metaregion for the specified hDC.

   ; hrgn - A handle to a pre-existing region. After the function returns,
   ; this parameter is a handle to a copy of the current metaregion.

   ; Return value
   ; If the function succeeds, the return value is nonzero.
   ; If the function fails, the return value is zero.

   ; Remarks
   ; If the function succeeds, hrgn is a handle to a copy of the current
   ; metaregion. Subsequent changes to this copy will not affect the
   ; current metaregion.

   ; The current clipping region of a device context is defined by the
   ; intersection of its clipping region and its metaregion.
   Return DllCall("gdi32\GetMetaRgn", "UPtr", hDC, "UPtr", hRgn)
}

Gdi_SetMetaRegion(hDC) {
   ; The function intersects the current clipping region for the specified
   ; device context with the current metaregion and saves the combined
   ; region as the new metaregion for the specified device context.
   ; The clipping region is reset to a null region.

   ; Return value
   ; The return value specifies the new clipping region's complexity and can
   ; be one of the following values:
        ; 1 = NULLREGION -  Region is empty.
        ; 2 = SIMPLEREGION - Region is a single rectangle.
        ; 3 = COMPLEXREGION - Region is more than one rectangle.
        ; 0 = An error occurred. The previous clipping region is unaffected.

   ; Remarks
   ; The function should only be called after an application's original
   ; device context was saved by calling the SaveDC function.

   Return DllCall("gdi32\SetMetaRgn", "UPtr", hDC)
}

Gdi_GetRegionBox(hRgn) {
   ; The function retrieves the bounding box of the given hRgn [region handle]
   ; The function will return an object.
     ; obju.x1, obju.y1, obju.x2, obju.y2 - the coordinates of two points representing the bounding box
        ; x1, y1 - top, left corner
        ; x2, y2 - bottom, right corner
     ; obj.E - the value returned by the internal API. It defines the region complexity.
        ; 1 = NULLREGION -  Region is empty.
        ; 2 = SIMPLEREGION - Region is a single rectangle.
        ; 3 = COMPLEXREGION - Region is more than one rectangle.
        ; 0 = An error occurred.

   VarSetCapacity(Rect, 16, 0)
   obju.E := DllCall("gdi32\GetRgnBox", "UPtr", hRgn, "UPtr", &Rect)
   obju.x1 := NumGet(Rect, 0, "uint")
   obju.y1 := NumGet(Rect, 4, "uint")
   obju.x2 := NumGet(Rect, 8, "uint")
   obju.y2 := NumGet(Rect, 12, "uint")
   Return obju
}

Gdi_InvertColorsRegion(hDC, hRgn) {
   ; The function inverts the colors in the specified region.

   ; Return value
   ; If the function succeeds, the return value is nonzero.
   ; If the function fails, the return value is zero.

   Return DllCall("gdi32\InvertRgn", "UPtr", hDC, "UPtr", hRgn)
}

Gdi_InvertColorsRect(hDC, x, y, w, h) {
   ; The function inverts the colors in the specified region.

   ; Return value
   ; If the function succeeds, the return value is nonzero.
   ; If the function fails, the return value is zero.
   VarSetCapacity(Rect, 16, 0)
   NumPut(x, Rect, 0, "uint"), NumPut(y, Rect, 4, "uint")
   NumPut(x + w, Rect, 8, "uint"), NumPut(y + h, Rect, 12, "uint")
   Return DllCall("gdi32\InvertRect", "UPtr", hDC, "UPtr", &Rect)
}

Gdi_PatBlt(hDC, x, y, w, h, mRop) {
   ; The function paints the specified rectangle using the brush that
   ; is currently selected into the specified hDC. The brush color and
   ; the surface color or colors are combined by using the specified
   ; raster operation [mRop].

   ; mRop options:
   ; PATCOPY   = 0x00F00021
   ; PATINVERT = 0x005A0049
   ; DSTINVERT = 0x00550009
   ; BLACKNESS = 0x00000042
   ; WHITENESS = 0x00FF0062

   ; Return value
   ; If the function succeeds, the return value is nonzero.
   ; If the function fails, the return value is zero.

   Return DllCall("gdi32\PatBlt", "UPtr", hDC, "Int", x, "Int", y, "Int", w, "Int", h, "uInt", mRop)
}

Gdi_OffsetRegion(hRgn, x, y) {
   ; Return value defines the region complexity.
   ; 1 = NULLREGION -  Region is empty.
   ; 2 = SIMPLEREGION - Region is a single rectangle.
   ; 3 = COMPLEXREGION - Region is more than one rectangle.
   ; 0 = An error occurred.

   Return DllCall("gdi32\OffsetRgn", "UPtr", hRgn, "Int", x, "Int", y)
}

Gdi_SetRectRegion(hRgn, x, y, w, h) {
   ; The function converts a region into a rectangular region with the specified coordinates.
   ; Return value
   ; If the specified point is in the region, the return value is nonzero.
   ; If the specified point is not in the region, the return value is zero.

   Return DllCall("gdi32\SetRectRgn", "UPtr", hRgn, "Int", x, "Int", y, "Int", x + w, "Int", y + h)
}

Gdi_IsPointInRegion(hRgn, x, y) {
   ; The function determines whether the specified point is inside the specified region.

   ; Return value
   ; If the specified point is in the region, the return value is nonzero.
   ; If the specified point is not in the region, the return value is zero.

   Return DllCall("gdi32\PtInRegion", "UPtr", hRgn, "Int", x, "Int", y)
}

Gdi_IsRectInRegion(hRgn, x, y, w, h) {
   ; The function determines whether any part of the specified rectangle
   ; is within the boundaries of a region.

   ; Return value
   ; If any part of the specified rectangle lies within the boundaries of the 
   ; region, the return value is nonzero. Otherwise, the value is zero.

   VarSetCapacity(Rect, 16, 0)
   NumPut(x, Rect, 0, "uint"), NumPut(y, Rect, 4, "uint")
   NumPut(x + w, Rect, 8, "uint"), NumPut(y + h, Rect, 12, "uint")
   Return DllCall("gdi32\RectInRegion", "UPtr", hRgn, "UPtr", &Rect)
}

Gdi_DrawTextOutline(hDC, hFont, Text, x, y, BorderColor, BorderWidth) {
      Pen := Gdi_CreatePen(BorderColor, BorderWidth)
      hOriginalPen := Gdi_SelectObject(hDC, Pen)
      hOriginalFont := Gdi_SelectObject(hDC, hFont)
      Gdi_BeginPath(hDC)
      Gdi_SetBgrMode(hDC, 1)
      ; E1 := Gdi_DrawText(hDC, Text, x, y, w, h, Alignment)
      E1 := Gdi_TextOut(hDC, Text, x, y)
      Gdi_EndPath(hDC)

      E2 := Gdi_StrokePath(hDC)
      Gdi_SelectObject(hDC, hOriginalPen)
      Gdi_SelectObject(hDC, hOriginalFont)
      Gdi_DeleteObject(Pen)
      ; Gdi_DeleteObject(hRgn)
      ; ToolTip, % E1 " , " E2 " , " hdc " , " hFont " , " borderColor " , " BorderWidth "`n" pen ";" y ";" w ";" h "`n" text, , , 2
      Return E1
}

Gdi_GetTextExtentPoint32(hDC, string, ByRef w, ByRef h) {
   VarSetCapacity(SIZE, 8, 0)
   E := DllCall("gdi32\GetTextExtentPoint32"
            ,"UPtr", hDC                                ;-- hDC
            ,"Str", string                             ;-- lpString
            ,"Int", StrLen(string)                     ;-- c (string length)
            ,"UPtr", &SIZE)                             ;-- lpSize

  w := NumGet(SIZE, 0, "Int")
  h := NumGet(SIZE, 4, "Int")
  SIZE := ""
  Return E
}

Gdi_MeasureString(hFont, string, precision, ByRef w, ByRef h) {
    ; precision = 0 ; uses GetTextExtentExPoint()
    ; precision = 1 ; uses DrawText()

    hDC := Gdi_GetDC()
    old_hFont := Gdi_SelectObject(hDC, hFont)

    If (precision=1)
       obju := Gdi_DrawText(hDC, string, 0, 0, 0, 0, "CALCRECT|NOCLIP|NOPREFIX|SINGLELINE")
    Else
       E := Gdi_GetTextExtentPoint32(hDC, string, w, h)

    If (precision=1)
    {
       w := obju.w
       h := obju.h
       E := obju.dll
    }
    Gdi_SelectObject(hDC, old_hFont)
    Gdi_ReleaseDC(hDC, 0)
    Return E
}

Gdi_SelectClipRegion(hDC, hRgn) {
     ; The function selects a region as the current clipping 
     ; region for the specified device context.
     ; Return values:
     ; 1 = NULLREGION -  Region is empty.
     ; 2 = SIMPLEREGION - Region is a single rectangle.
     ; 3 = COMPLEXREGION - Region is more than one rectangle.
     ; 0 = An error occurred. The previous clipping region is unaffected.

     Return DllCall("gdi32\SelectClipRgn", "UPtr", hDC, "UPtr", hRgn)
}

Gdi_GetClipBox(hDC) {
   ; The function  retrieves the dimensions of the tightest bounding rectangle
   ; that can be drawn around the current visible area on the device.
   ; The visible area is defined by the current clipping region or clip path,
   ; as well as any overlapping windows.
   ; GetClipBox returns logical coordinates based on the given device context.

   ; The function will return an object.
     ; obju.x1, obju.y1, obju.x2, obju.y2 - the coordinates of two points representing the bounding box
        ; x1, y1 - top, left corner
        ; x2, y2 - bottom, right corner
     ; obj.E - the value returned by the internal API. It defines the region complexity.
        ; 1 = NULLREGION -  Region is empty.
        ; 2 = SIMPLEREGION - Region is a single rectangle.
        ; 3 = COMPLEXREGION - Region is more than one rectangle.
        ; 0 = An error occurred.

   VarSetCapacity(Rect, 16, 0)
   obju.E := DllCall("gdi32\GetClipBox", "UPtr", hDC, "UPtr", &Rect)
   obju.x1 := NumGet(Rect, 0, "uint")
   obju.y1 := NumGet(Rect, 4, "uint")
   obju.x2 := NumGet(Rect, 8, "uint")
   obju.y2 := NumGet(Rect, 12, "uint")
   Return obju
}

Gdi_ExtSelectClipRgn(hDC, hRgn, mode) {
   ; Parameters:
   ; hrgn - A handle to the region to be selected. This handle must not be 
   ; NULL unless the RGN_COPY mode is specified.

   ; Modes:
   ; RGN_AND = 1
   ;   The new clipping region includes the intersection (overlapping areas) of the current clipping region and the current path.
   ; RGN_OR = 2
   ;   The new clipping region includes the union (combined areas) of the current clipping region and the current path.
   ; RGN_XOR = 3
   ;   The new clipping region includes the union of the current clipping region and the current path but without the overlapping areas. 
   ; RGN_DIFF = 4
   ;   The new clipping region includes the areas of the current clipping region with those of the current path excluded.
   ; RGN_COPY = 5
   ;   The new clipping region is the current path.

   ; Return value
   ; The return value specifies the new clipping region's complexity; it can
   ; be one of the following values:
      ; 1 = NULLREGION -  Region is empty.
      ; 2 = SIMPLEREGION - Region is a single rectangle.
      ; 3 = COMPLEXREGION - Region is more than one rectangle.
      ; 0 = An error occurred.


   ; Remarks
   ; If an error occurs when this function is called, the previous clipping
   ; region for the specified device context is not affected.

   ; This function assumes that the coordinates for the specified region
   ; are specified in device units.

   ; Only a copy of the region identified by the hRgn parameter is used.
   ; The region itself can be reused after this call or it can be deleted.

   Return DllCall("gdi32\ExtSelectClipRgn", "UPtr", hDC, "UPtr", hRgn, "Int", mode)
}

Gdi_ClipPointVisible(hDC, x, y) {
   ; The function determines whether the specified point is within the
   ; clipping region of a device context.

   ; Return value
   ; If the specified point is within the clipping region of the hDC,
   ; the return value is TRUE (1).

   ; If the specified point is not within the clipping region of the hDC,
   ; the return value is FALSE (0).

   ; If the hDC is not valid, the return value is -1.
   Return DllCall("gdi32\PtVisible", "UPtr", hDC, "Int", x, "Int", y)
}

Gdi_ClipRectVisible(hDC, x, y, w, h) {
   ; The function determines whether any part of the specified rectangle
   ; lies within the clipping region of a device context.

   VarSetCapacity(Rect, 16, 0)
   NumPut(x, Rect, 0, "uint"), NumPut(y, Rect, 4, "uint")
   NumPut(x + w, Rect, 8, "uint"), NumPut(y + h, Rect, 12, "uint")

   Return DllCall("gdi32\RectVisible", "UPtr", hDC, "UPtr", &Rect)
}

Gdi_GetClipRegion(hDC, hRgn) {
   ; The function retrieves a handle identifying the current application
   ;-defined clipping region for the specified device context.

   ; hrgn - A handle to an existing region before the function is called.
   ; After the function returns, this parameter is a handle to a copy of
   ; the current clipping region.

   ; Return value
   ; If the function succeeds and there is no clipping region for the given
   ; device context, the return value is zero. If the function succeeds and
   ; there is a clipping region for the given DC, the return value is 1. 
   ; If an error occurs, the return value is -1.

   ; Remarks
   ; An application-defined clipping region is a clipping region identified
   ; by the SelectClipRgn function. It is not a clipping region created when
   ; the application calls the BeginPaint function.

   ; If the function succeeds, the hRgn parameter is a handle to a copy of
   ; the current clipping region. Subsequent changes to this copy will
   ; not affect the current clipping region.

   Return DllCall("gdi32\GetClipRgn", "UPtr", hDC, "UPtr", hRgn)
}

Gdi_SelectClipPath(hDC, mode) {
     ; The SelectClipPath function selects the current path as a clipping region
     ; for a device context, combining the new region with any existing clipping 
     ; region using the specified mode. The provided path must be a closed path.

     ; Modes:
     ; RGN_AND = 1
     ;   The new clipping region includes the intersection (overlapping areas) of the current clipping region and the current path.
     ; RGN_OR = 2
     ;   The new clipping region includes the union (combined areas) of the current clipping region and the current path.
     ; RGN_XOR = 3
     ;   The new clipping region includes the union of the current clipping region and the current path but without the overlapping areas. 
     ; RGN_DIFF = 4
     ;   The new clipping region includes the areas of the current clipping region with those of the current path excluded.
     ; RGN_COPY = 5
     ;   The new clipping region is the current path.
 
     Return DllCall("gdi32\SelectClipPath", "UPtr", hDC, "int", mode)
}

Gdi_IntersectClipRect(hDC, x, y, x2, y2) {
     ; The SelectClipRgn function selects a region as the current clipping 
     ; region for the specified device context.

     ; Return values:
     ; 1 = NULLREGION -  Region is empty.
     ; 2 = SIMPLEREGION - Region is a single rectangle.
     ; 3 = COMPLEXREGION - Region is more than one rectangle.
     ; 0 = An error occurred. The previous clipping region is unaffected.

     Return DllCall("gdi32\IntersectClipRect", "UPtr", hDC, "Int", x, "Int", y, "Int", x2, "Int", y2)
}

Gdi_OffsetClipRegion(hDC, x, y) {
     ; The OffsetClipRgn function moves the clipping region of a
     ; device context by the specified offsets.

     ; Return values:
     ; 1 = NULLREGION -  Region is empty.
     ; 2 = SIMPLEREGION - Region is a single rectangle.
     ; 3 = COMPLEXREGION - Region is more than one rectangle.
     ; 0 = An error occurred.

     Return DllCall("gdi32\OffsetClipRgn", "UPtr", hDC, "Int", x, "Int", y)
}


Gdi_ExcludeClipRect(hDC, x, y, x2, y2) {
     ; The ExcludeClipRect function creates a new clipping region that consists
     ; of the existing clipping region minus the specified rectangle.

     ; Return values:
     ; 1 = NULLREGION -  Region is empty.
     ; 2 = SIMPLEREGION - Region is a single rectangle.
     ; 3 = COMPLEXREGION - Region is more than one rectangle.
     ; 0 = An error occurred. The previous clipping region is unaffected.

     Return DllCall("gdi32\ExcludeClipRect", "UPtr", hDC, "Int", x, "Int", y, "Int", x2, "Int", y2)
}


Gdi_SetTextAlign(hDC, flags) {
     ; TA_LEFT = 0
     ; TA_RIGHT = 2
     ; TA_CENTER = 6
     ; TA_TOP = 0
     ; TA_BOTTOM = 8
     ; TA_BASELINE = 24

     ; TA_RTLREADING = 256
     ; Middle East language edition of Windows: The text is laid out in
     ; right to left reading order, as opposed to the default left to right
     ; order. This applies only when the font selected into the
     ; device context is either Hebrew or Arabic.

     ; TA_NOUPDATECP = 0
     ; The current position is not updated after each text output
     ; call. The reference point is passed to the text output function.

     ; TA_UPDATECP = 1
     ; The current position is updated after each text output call. The
     ; current position is used as the reference point. 

     ; Vertical alignment:
     ; VTA_BASELINE = TA_BASELINE
     ; VTA_LEFT = TA_BOTTOM
     ; VTA_RIGHT = TA_TOP
     ; VTA_CENTER = TA_CENTER
     ; VTA_BOTTOM = TA_RIGHT
     ; VTA_TOP = TA_LEFT
     
     ; The SetTextAlign function sets the text-alignment flags for the
     ; specified device context.

     ; To set text alignment use the bit-wise OR operator |
     ; Example: 2|24
     ; Equivalent to: TA_RIGHT|VTA_BASELINE
     ; Default values for an hDC are TA_LEFT, TA_TOP, and TA_NOUPDATECP.

     Return DllCall("gdi32\SetTextAlign", "UPtr", hDC, "UInt", flags)
}

Gdi_SetTextColor(hDC, color) {
     Return DllCall("gdi32\SetTextColor", "UPtr", hDC, "UInt", color)
}

Gdi_SetTextJustification(hDC, extraSpace, countBreaks) {
     ; The SetTextJustification function specifies the amount of space the 
     ; system should add to the break characters in a string of text.
     ; The space is added when an application calls the TextOut
     ; or ExtTextOut functions.

     ; extraSpace
     ; The total extra space, in logical units, to be added to the line of
     ; text. If the current mapping mode is not MM_TEXT, the value 
     ; identified by the nBreakExtra parameter is transformed and rounded
     ; to the nearest pixel.

     ; countBreaks
     ; The number of break characters in the line.

     ; Return value
     ; If the function succeeds, the return value is nonzero.

     ; Remarks
     ; The break character is usually the space character (ASCII 32), 
     ; but it may be defined by a font as some other character.
     ; The GetTextMetrics function can be used to retrieve a
     ; font's break character.

     ; The TextOut function distributes the specified extra space evenly
     ; among the break characters in the line.

     ; The GetTextExtentPoint32 function is always used with 
     ; the SetTextJustification function. Sometimes the
     ; GetTextExtentPoint32 function takes justification
     ; into account when computing the width of a specified line
     ; before justification, and sometimes it does not. 

     Return DllCall("gdi32\SetTextJustification", "UPtr", hDC, "Int", extraSpace, "Int", countBreaks)
}

Gdi_BeginPath(hDC) {
     ; Opens a path bracket in the specified device context.
     Return DllCall("gdi32\BeginPath", "UPtr", hDC)
}

Gdi_EndPath(hDC) {
     ; Closes a path bracket and selects the path defined by the bracket
     ; into the specified device context.

     Return DllCall("gdi32\EndPath", "UPtr", hDC)
}

Gdi_AbortPath(hDC) {
     ; Closes and discards any paths in the specified device context.
     Return DllCall("gdi32\AbortPath", "UPtr", hDC)
}

Gdi_FlattenPath(hDC) {
     ; Transforms any curves in the path that is selected into
     ; the current device context (DC), turning each curve into
     ; a sequence of lines.

     Return DllCall("gdi32\FlattenPath", "UPtr", hDC)
}

Gdi_WidenPath(hDC) {
     ; Redefines the current path as the area that would be painted if
     ; the path were stroked using the pen currently selected into
     ; the given device context.

     Return DllCall("gdi32\WidenPath", "UPtr", hDC)
}

Gdi_StrokePath(hDC) {
     ; Renders the specified path by using the current pen.
     ; If the function fails, the return value is zero.
     Return DllCall("gdi32\StrokePath", "UPtr", hDC)
}

Gdi_FillPath(hDC) {
     ; The FillPath function closes any open figures in the
     ; current path and fills the path's interior by using
     ; the current brush and polygon-filling mode.
     ; If the function fails, the return value is zero.

     Return DllCall("gdi32\StrokePath", "UPtr", hDC)
}

Gdi_StrokeAndFillPath(hDC) {
     ; The StrokeAndFillPath function closes any open figures in a path,
     ; strokes the outline of the path by using the current pen,
     ; and fills its interior by using the current brush.
     ; If the function fails, the return value is zero.

     Return DllCall("gdi32\StrokeAndFillPath", "UPtr", hDC)
}

Gdi_PathToRegion(hDC) {
     ; If the function succeeds, the return value identifies a valid region.
     ; If the function fails, the return value is zero.
     Return DllCall("gdi32\PathToRegion", "UPtr", hDC)
}

Gdi_SetMiterLimit(hDC, limit:=10) {
     ; The SetMiterLimit function sets the limit for the length of
     ; miter joins for the specified device context.
     ; Minimum value is 1.
     Return DllCall("gdi32\SetMiterLimit", "UPtr", hDC, "uint", limit)
}

Gdi_CloseFigure(hDC) {
     Return DllCall("gdi32\CloseFigure", "UPtr", hDC)
}

Gdi_SetBgrMode(hDC, mode) {
     ; mode = 1 ; transparent
     ; mode = 2 ; opaque
    Return DllCall("gdi32\SetBkMode", "UPtr", hDC, "Int", mode)
}

Gdi_SetBgrColor(hDC, color) {
     ; The SetBkColor function sets the current background color to the specified color value, or to the nearest physical color if the device cannot represent the specified color value.
    Return DllCall("gdi32\SetBkColor", "UPtr", hDC, "UInt", color)
}

Gdi_CreatePen(Color, Width:=1, Style:=0) {
   ; Style options:
   ; 0 = PS_SOLID - solid.
   ; 1 = PS_DASH - dashed. Valid only when the pen width is one or less in device units.
   ; 2 = PS_DOT - dotted. Valid only when the pen width is one or less in device units.
   ; 3 = PS_DASHDOT - alternating dashes and dots. Valid only when the pen width is one or less in device units.
   ; 4 = PS_DASHDOTDOT - alternating dashes and double dots. Valid only when the pen width is one or less in device units.
   ; 5 = PS_NULL - The pen is invisible.
   ; 6 = PS_INSIDEFRAME - The pen is solid. When this pen is used in any GDI drawing function that takes
         ; a bounding rectangle, the dimensions of the figure are shrunk so that it fits entirely in the 
         ; bounding rectangle, taking into account the width of the pen. This applies only to geometric pens. 
   ; 7 = PS_USERSTYLE
   ; 8 = PS_ALTERNATE

   Return DllCall("gdi32\CreatePen", "Int", Style, "Int", Width, "UInt", Color, "UPtr")
}

Gdi_CreateSolidBrush(gColor) {
   Return DllCall("gdi32\CreateSolidBrush", "UInt", gColor, "UPtr")
}

Gdi_CreatePatternBrush(hBitmap) {
   ; The CreatePatternBrush function creates a logical brush with the
   ; specified bitmap pattern. The bitmap can be a DIB section bitmap, 
   ; which is created by the CreateDIBSection function, or it can
   ; be a device-dependent bitmap.

   Return DllCall("gdi32\CreatePatternBrush", "UPtr", hBitmap)
}

Gdi_CreateHatchBrush(iHatch, color) {
   ; The CreateHatchBrush function creates a logical brush that has
   ; the specified hatch pattern and color.

   ; iHatch options:
   ; HS_HORIZONTAL = 0  ; Horizontal hatch
   ; HS_VERTICAL = 1    ; Vertical hatch 
   ; HS_FDIAGONAL = 2   ; 45-degree downward left-to-right hatch
   ; HS_BDIAGONAL = 3   ; 45-degree upward left-to-right hatch
   ; HS_CROSS = 4       ; Horizontal and vertical crosshatch
   ; HS_DIAGCROSS = 5   ; 45-degree crosshatch

   Return DllCall("gdi32\CreateHatchBrush", "Int", iHatch, "UInt", Color, "UPtr")
}

Gdi_ExtFloodFill(X, Y, Color, Type) {
   ; Type options:
   ; 0 = FLOODFILLBORDER -The fill area is bounded by the color specified by the crColor parameter. This style is identical to the filling performed by the FloodFill function.
   ; 1 = FLOODFILLSURFACE - The fill area is defined by the color that is specified by crColor. Filling continues outward in all directions as long as the color is encountered. This style is useful for filling areas with multicolored boundaries. 
   ; If the function fails, the return value is zero.

   Return DllCall("gdi32\ExtFloodFill", "Int", x, "Int", y, "UInt", Color, "Int", Type)
}

Gdi_SetBrushOrgEx(hDC, X, Y) {
   Return DllCall("gdi32\SetBrushOrgEx", "UPtr", hDC, "Int", x, "Int", y, "UPtr", 0)
}

Gdi_SetDCBrushColor(hDC, Color) {
   Return DllCall("gdi32\SetDCBrushColor", "UPtr", hDC, "UInt", Color, "UPtr")
}

Gdi_SetDCPenColor(hDC, Color) {
   Return DllCall("gdi32\SetDCPenColor", "UPtr", hDC, "UInt", Color, "UPtr")
}

Gdi_GetROP2(hDC) {
; The function retrieves the foreground mix mode of the specified device
; context. The mix mode specifies how the pen or interior color and 
; the color already on the screen are combined to yield a new color.

; Return value
; If the function succeeds, the return value specifies the foreground mix mode.
; otherwise, the return value is zero.

   Return DllCall("gdi32\GetROP2", "UPtr", hDC)
}

Gdi_SetROP2(hDC, rop2) {
; The function sets the current foreground mix mode. GDI uses the foreground 
; mix mode to combine pens and interiors of filled objects with the
; colors already on the screen. The foreground mix mode defines how colors
; from the brush or pen and the colors in the existing image are to be combined.

; The mix modes define how GDI combines source and destination colors when
; drawing with the current pen. The mix modes are binary raster operation
; codes, representing all possible Boolean functions of two variables, using
; the binary operations AND, OR, and XOR (exclusive OR), and the unary operation
; NOT. The mix mode is for raster devices only; it is not available
; for vector devices.

; Parameters
;   hdc  - A handle to the device context.
;   rop2 - The mix mode.

; Return value
; If the function succeeds, the return value specifies the previous mix mode.
; otherwise, the return value is zero.

   Return DllCall("gdi32\SetROP2", "UPtr", hDC, "Int", rop2)
}

Gdi_FillShape(hDC, x, y, w, h, Color, Shape, BorderColor:=0, BorderWidth:=0) {
      If (BorderColor && BorderWidth)
         Pen := Gdi_CreatePen(BorderColor, BorderWidth)

      Brush := Gdi_CreateSolidBrush(Color)
      ; Replace the original pen and brush with our own
      hOriginalPen := Gdi_SelectObject(hDC, Pen)
      hOriginalBrush := Gdi_SelectObject(hDC, Brush)
      func2call := (Shape=1) ? "Rectangle" : "Ellipse"
      E := Gdi_%func2call%(hDC, x, y, x + w, y + h)

      ; Reselect the original pen and brush
      Gdi_SelectObject(hDC, hOriginalPen)
      Gdi_SelectObject(hDC, hOriginalBrush)
      If Pen
         Gdi_DeleteObject(Pen)
      Gdi_DeleteObject(Brush)
      Return E
}

Gdi_Rectangle(hDC, x1, y1, x2, y2) {
   Return DllCall("gdi32\Rectangle", "UPtr", hDC
          , "Int", x1, "Int", y1
          , "Int", x2, "Int", y2)
}

Gdi_FillRectangle(hDC, x, y, w, h, hBrush) {
   ; Return value
   ; If the function succeeds, the return value is nonzero.
   ; If the function fails, the return value is zero.

   VarSetCapacity(Rect, 16, 0)
   NumPut(x, Rect, 0, "uint"), NumPut(y, Rect, 4, "uint")
   NumPut(x + w, Rect, 8, "uint"), NumPut(y + h, Rect, 12, "uint")

   Return DllCall("gdi32\FillRect", "UPtr", hDC, "UPtr", &Rect, "UPtr", hBrush)
}

Gdi_FrameRectangle(hDC, x, y, w, h, hBrush) {
   ; The FrameRect function draws a border around the specified rectangle by using the specified brush. 
   ; The width and height of the border are always one logical unit.

   ; Return value
   ; If the function succeeds, the return value is nonzero.
   ; If the function fails, the return value is zero.
   ; If the bottom member of the RECT structure is less than the top member,
   ; or if the right member is less than the left member, the function
   ; does not draw the rectangle.

   VarSetCapacity(Rect, 16, 0)
   NumPut(x, Rect, 0, "uint"), NumPut(y, Rect, 4, "uint")
   NumPut(x + w, Rect, 8, "uint"), NumPut(y + h, Rect, 12, "uint")
   Return DllCall("gdi32\FrameRect", "UPtr", hDC, "UPtr", &Rect, "UPtr", hBrush)
}

Gdi_Chord(hDC, x1, y1, x2, y2, x3, y3, x4, y4) {
   ; The Chord function draws a chord (a region bounded by the intersection
   ; of an ellipse and a line segment, called a secant). The chord is outlined
   ; and filled by using the currently selected pen and brush in DC.

   Return DllCall("gdi32\Chord", "UPtr", hDC
          , "Int", x1, "Int", y1
          , "Int", x2, "Int", y2
          , "Int", x3, "Int", y3
          , "Int", x4, "Int", y4)
}

Gdi_Pie(hDC, x1, y1, x2, y2, xr1, yr1, xr2, yr2) {
   ; The Pie function draws a pie-shaped wedge bounded by the intersection of an ellipse and two radials. 
   ; The pie is outlined and filled by using the currently selected pen and brush in DC.

   Return DllCall("gdi32\Pie", "UPtr", hDC
          , "Int", x1, "Int", y1
          , "Int", x2, "Int", y2
          , "Int", xr1, "Int", yr1
          , "Int", xr2, "Int", yr2)
}

Gdi_DrawLine(hDC, x, y, x2, y2, Pen:=0) {
    Gdi_MoveToEx(HDC, x, y)
    If Pen
       hOriginalPen := Gdi_SelectObject(hDC, Pen)

    E := Gdi_LineTo(hDC, x2, y2)
    If hOriginalPen
       hOriginalPen := Gdi_SelectObject(hDC, hOriginalPen)
    Return E
}

Gdi_LineTo(hDC, x, y) {
   Return DllCall("gdi32\LineTo", "UPtr", hDC, "Int", x, "Int", y)
}

Gdi_MoveToEx(hDC, x, y) {
   Return DllCall("gdi32\MoveToEx", "UPtr", hDC, "Int", x, "Int", y, "UPtr", 0)
}

Gdi_AngleArc(hDC, x, y, radius, StartAngle, SweepAngle) {
; Return value:
; If the function succeeds, the return value is nonzero.

; Remarks:
; The AngleArc function moves the current position to the ending point
; of the arc.

; The arc drawn by this function may appear to be elliptical,
; depending on the current transformation and mapping mode.
; Before drawing the arc, AngleArc draws the line segment from
; the current position to the beginning of the arc.

; The arc is drawn by constructing an imaginary circle around
; the specified center point with the specified radius.
; The starting point of the arc is determined by measuring
; counterclockwise from the x-axis of the circle by the number
; of degrees in the start angle. The ending point is similarly
; located by measuring counterclockwise from the starting point
; by the number of degrees in the sweep angle.

; If the sweep angle is greater than 360 degrees, the arc is
; swept multiple times.

; This function draws lines by using the current pen.
; The figure is not filled.

   Return DllCall("gdi32\AngleArc", "UPtr", hDC
                   , "int", x        , "int", y
                   , "uint", radius  , "float", StartAngle
                   , "float", SweepAngle)
}

Gdi_ArcTo(hDC, left, top, right, bottom, xr1, yr1, xr2, yr2) {
; The ArcTo function draws an elliptical arc.
; Return: If the arc is drawn, the return value is nonzero.
   Return DllCall("gdi32\ArcTo", "UPtr", hDC
                   , "int", left     , "int", top
                   , "int", right    , "int", bottom
                   , "int", xr1      , "int", yr1
                   , "int", xr2      , "int", yr2)
}

Gdi_Arc(hDC, x1, y1, x2, y2, x3, y3, x4, y4) {
; The Arc function draws an elliptical arc.
; Return: If the arc is drawn, the return value is nonzero.
   Return DllCall("gdi32\ArcTo", "UPtr", hDC
                   , "int", x1     , "int", y1
                   , "int", x2     , "int", y2
                   , "int", x3     , "int", y3
                   , "int", x4     , "int", y4)
}

Gdi_Ellipse(hDC, x1, y1, x2, y2) {
; Return: If the ellipse is drawn, the return value is nonzero.
   Return DllCall("gdi32\Ellipse", "UPtr", hDC
          , "Int", x1, "Int", y1
          , "Int", x2, "Int", y2)
}

Gdi_FillRoundedRect(hDC, x, y, w, h, Color, Radius, BorderColor:=0, BorderWidth:=0) {
      If (BorderColor && BorderWidth)
         Pen := Gdi_CreatePen(BorderColor, BorderWidth)

      Brush := Gdi_CreateSolidBrush(Color)

      ; Replace the original pen and brush with our own
      hOriginalPen := Gdi_SelectObject(hDC, Pen)
      hOriginalBrush := Gdi_SelectObject(hDC, Brush)

      E := Gdi_RoundRect(hDC, x, y, x + w, y + h, Radius, Radius)
      ; Reselect the original pen and brush
      Gdi_SelectObject(hDC, hOriginalPen)
      Gdi_SelectObject(hDC, hOriginalBrush)
      Gdi_DeleteObject(Pen)
      Gdi_DeleteObject(Brush)
      Return E
}

Gdi_RoundRect(hDC, x, y, x2, y2, RadiusX, RadiusY) {
   Return DllCall("gdi32\RoundRect", "UPtr", hDC
          , "Int", x, "Int", y
          , "Int", x2, "Int", y2
          , "Int", RadiusX, "Int", RadiusY)
}

Gdi_GetImageDimensions(hBitmap, ByRef width, ByRef height, ByRef BPP) {
     If (Gdi_GetObjectType(hBitmap)!="BITMAP")
        Return -1

     size := VarSetCapacity(dib, 76+2*(A_PtrSize=8?4:0)+2*A_PtrSize, 0) ; sizeof(DIBSECTION) = x86:84, x64:104
     E := DllCall("gdi32\GetObject", "UPtr", hBitmap, "int", size, "UPtr", &dib)
     width := NumGet(dib, 4, "uint")
     height := NumGet(dib, 8, "uint")
     BPP := NumGet(dib, 18, "ushort")
     dib := ""
     Return E
}

Gdi_GetObjectType(obj) {
   Static objTypes := {1:"PEN", 2:"BRUSH", 3:"DC", 4:"METADC", 5:"PAL", 6:"FONT", 7:"BITMAP", 8:"REGION", 9:"METAFILE", 10:"MEMDC", 11:"EXTPEN", 12:"ENHMETADC", 13:"ENHMETAFILE", 14:"COLORSPACE"}

   E := DllCall("gdi32\GetObjectType", "UPtr", obj)
   R := objTypes[E]
   Return R ? R : E
}

Gdi_CreateBitmap(w, h, BitCount:=32, Planes:=1, pBits:=0) {
  ; The function creates a device-dependent bitmap [DDB].
  ; After a bitmap is created, it can be selected into a device context [DC] by calling
  ; the Gdi_SelectObject function. However, the bitmap can only be selected into
  ; a DC if the bitmap and the DC have the same format.

   Return DllCall("gdi32\CreateBitmap", "Int", w, "Int", h, "UInt", Planes, "UInt", BitCount, "UPtr", pBits, "UPtr")
}

Gdi_CreateDIBitmap(hDC, bmpInfoHeader, CBM_INIT, pBits, BITMAPINFO, DIB_COLORS) {
; This function creates a hBitmap, a device-dependent bitmap [DDB]
; from a pointer of data-bits [pBits].
;
; The hBitmap is created according to the information found in
; the BITMAPINFO and bmpInfoHeader pointers.
;
; If the function fails, the return value is NULL,
; otherwise a handle to the hBitmap
;
; Function written by Marius Șucan.
; Many thanks to Drugwash for the help offered.

   hBitmap := DllCall("gdi32\CreateDIBitmap"
            , "UPtr", hDC
            , "UPtr", bmpInfoHeader
            , "uint", CBM_INIT    ; = 4
            , "UPtr", pBits
            , "UPtr", BITMAPINFO
            , "uint", DIB_COLORS, "UPtr")    ; PAL=1 ; RGB=2

   Return hBitmap
}

Gdi_StretchDIBits(hDestDC, dX, dY, dW, dH, sX, sY, sW, sH, tBITMAPINFO, DIB_COLORS, pBits, RasterOper) {
   ; The StretchDIBits function copies the color data for a rectangle of pixels
   ; in a DIB, BMP, JPEG, or PNG image to the specified destination rectangle.
   ; If the destination rectangle is larger than the source rectangle, this
   ; function stretches the rows and columns of color data to fit the
   ; destination rectangle. If the destination rectangle is smaller than the
   ; source rectangle, this function compresses the rows and columns by using
   ; the specified raster operation.

   ; pBits
   ; A handle to the image bits, which are stored as an array of bytes.

   ; tBITMAPINFO
   ; A BITMAPINFO structure that contains information about the DIB [Device Independent Bitmap].

   ; DIB_COLORS
   ; Specifies whether the bmiColors member of the BITMAPINFO structure was
   ; provided and, if so, whether bmiColors contains explicit red, green, blue
   ; (RGB) values or indexes. The parameter must be one of the following values.
   ;    DIB_PAL_COLORS = 1
   ;    The array contains 16-bit indexes into the logical palette of the source device context.

   ;    DIB_RGB_COLORS = 2
   ;    The color table contains literal RGB values. 

   ; RasterOper
   ; A raster-operation code that specifies how the source pixels, the destination
   ; device context's current brush, and the destination pixels are to be
   ; combined to form the new image. For a list of some common raster operation
   ; codes, see BitBlt.

   ; Return value
   ; If the function succeeds, the return value is the number of scan lines copied. 
   ; Note that this value can be negative for mirrored content.
   ; If the function fails, or no scan lines are copied, the return value is 0.

   ; If the driver cannot support the JPEG or PNG file image passed to
   ; StretchDIBits, the function will fail and return GDI_ERROR.
   ; If failure does occur, the application must fall back on its own
   ; JPEG or PNG support to decompress the image into a bitmap, and then
   ; pass the bitmap to StretchDIBits.

   Return DllCall("gdi32\StretchDIBits"
      , "UPtr", hDestDC, "int", dX, "int", dY
      , "int", dW, "int", dH, "int", sX, "int", sY
      , "int", sW, "int", sH, "UPtr", pBits, "UPtr", tBITMAPINFO
      , "int", DIB_COLORS, "uint", RasterOper)
}

Gdi_SetDIBitsToDevice(hDC, dX, dY, Width, Height, sX, sY, StartScan, ScanLines, pBits, BITMAPINFO, DIB_COLORS) {
   ; The SetDIBitsToDevice function sets the pixels in the specified rectangle
   ; on the device that is associated with the destination device context
   ; using color data from a DIB, JPEG, or PNG image.

   ; StartScan
   ; The starting scan line in the image.

   ; ScanLines
   ; The number of DIB scan lines contained in the array pointed to by the pBits parameter.

   ; pBits
   ; A handle to the color data stored as an array of bytes.

   ; BITMAPINFO
   ; A BITMAPINFO structure that contains information about the DIB.

   ; DIB_COLORS
   ; Indicates whether the bmiColors member of the BITMAPINFO structure contains
   ; explicit red, green, blue (RGB) values or indexes into a palette.
      ;    DIB_PAL_COLORS = 1
      ;    DIB_RGB_COLORS = 2

   ; Return value
   ; If the function succeeds, the return value is the number of scan lines set.
   ; If zero scan lines are set (such as when dwHeight is 0) or the function fails,
   ; the function returns zero.

   ; If the driver cannot support the JPEG or PNG file image passed to
   ; SetDIBitsToDevice, the function will fail and return GDI_ERROR.
   ; If failure does occur, the application must fall back on its own
   ; JPEG or PNG support to decompress the image into a bitmap, and then
   ; pass the bitmap to SetDIBitsToDevice.

   ; Optimal bitmap drawing speed is obtained when the bitmap bits
   ; are indexes into the system palette.

   Return DllCall("gdi32\SetDIBitsToDevice", "UPtr", hDC
         , "int", dX, "int", dY
         , "uint", Width, "uint", Height
         , "int", sX, "int", sY
         , "uint", StartScan, "uint", ScanLines
         , "UPtr", pBits, "UPtr", BITMAPINFO, "uint", DIB_COLORS)
}

Gdi_GetDIBits(hDC, hBitmap, start, cLines, pBits, BITMAPINFO, DIB_COLORS) {
   ; hDC     - A handle to the device context.
   ; hBitmap - A handle to the GDI bitmap. This must be a compatible bitmap (DDB).
   ; pBits   --A pointer to a buffer to receive the bitmap data.
   ;           If this parameter is NULL, the function passes the dimensions
   ;           and format of the bitmap to the BITMAPINFO structure pointed to 
   ;           by the BITMAPINFO parameter.
   ;
   ; A DDB is a Device-Dependent Bitmap, (as opposed to a DIB, or Device-Independent Bitmap).
   ; That means: a DDB does not contain color values; instead, the colors are in a
   ; device-dependent format. Therefore, it requires a hDC.
   ; 
   ; This function returns the data-bits as device-independent bitmap
   ; from a hBitmap into the pBits pointer.
   ;
   ; Return: if the function fails, the return value is zero.
   ; It can also return ERROR_INVALID_PARAMETER.

   Return DllCall("gdi32\GetDIBits"
            , "UPtr", hDC
            , "UPtr", hBitmap
            , "uint", start
            , "uint", cLines
            , "UPtr", pBits
            , "UPtr", BITMAPINFO
            , "uint", DIB_COLORS, "UPtr")    ; PAL=1 ; RGB=2
}


Gdi_GetWindowDC(hwnd) {
  ; The GetWindowDC function retrieves the device context (DC) for the entire window,
  ; including title bar, menus, and scroll bars. A window DC permits painting anywhere
  ; in a window, because the origin of the DC is the upper-left corner of the window
  ; instead of the client area. This DC is a display DC.

  ; GetWindowDC assigns default attributes to the window DC each time it is retrieved.
  ; Previous attributes are lost.

   return DllCall("user32\GetWindowDC", "UPtr", hwnd)
}

; == Device Contexts (DCs) ==
; There are four types of DCs: display, printer, memory (or compatible),
; and information. Each type serves a specific purpose, as described
; in the following:
;   Display == Supports drawing operations on a video display.
;   Printer == Supports drawing operations on a printer or plotter.
;   Memory  == Supports drawing operations on a bitmap; it is a compatible DC.
;   Info    == Supports the retrieval of device data.

; == DCs - Display ==
; An application obtains a display DC by calling the BeginPaint,
; GetDC, or GetDCEx function and identifying the window in which
; the corresponding output will appear. Typically, an application
; obtains a display DC only when it must draw in the client area.
; However, one may obtain a window DC by calling the GetWindowDC
; function. When the application is finished drawing, it must
; release the DC by calling the EndPaint or ReleaseDC function.

; There are five types of DCs for video displays:
    ; Class (obsolete), Common, Private, Window, Parent

; == DCs - Memory ==
; To enable applications to place output in memory rather than sending
; it to an actual device, use a special device context for bitmap
; operations called a memory device context. A memory DC enables the
; system to treat a portion of memory as a virtual device. It is an
; array of bits in memory that an application can use temporarily to
; store the color data for bitmaps created on a normal drawing surface.
; Because the bitmap is compatible with the device, a memory DC is
; also sometimes referred to as a compatible device context.

; The memory DC stores bitmap images for a particular device.
; An application can create a memory DC by calling the
; CreateCompatibleDC() function.

; The original bitmap in a memory DC is simply a placeholder.
; Its dimensions are one pixel by one pixel. Before an application
; can begin drawing, it must select a bitmap with the appropriate
; width and height into the DC by calling the SelectObject() function.
; To create a bitmap of the appropriate dimensions, use the 
; CreateBitmap(), CreateBitmapIndirect(), or CreateCompatibleBitmap()
; function. After the bitmap is selected into the memory DC, the
; system replaces the single-bit array with an array large enough
; to store color information for the specified rectangle of pixels.

; When an application passes the handle returned by CreateCompatibleDC()
; to one of the drawing functions, the requested output does not appear
; on a device's drawing surface. Instead, the system stores the color
; information for the resultant line, curve, text, or region in the
; array of bits. The application can copy the image stored in memory
; back onto a drawing surface by calling the BitBlt() function,
; identifying the memory DC as the source device context and a window
; or screen DC as the target DC.

Gdi_GetDC(hwnd:=0) {
  ; Description
  ; This function retrieves a handle to a display device context (DC)
  ; for the client area of the specified window.
  ; The display DC can be used in subsequent GDI
  ; functions to draw in the client area of the window.
  ;
  ; hwnd
  ; Handle to the window whose device context is to be retrieved.
  ; If this value is NULL, GetDC retrieves the DC for the entire screen.
  ;
  ; Remarks
  ; this type of DC must be released with Gdi_ReleaseDC()
  ;
  ; Return
  ; The handle of the DC of the specified window's client area
  ; indicates success. NULL indicates failure,

   return DllCall("user32\GetDC", "UPtr", hwnd)
}

Gdi_CreateCompatibleDC(hDC:=0) {
  ; this type of DC must be released with Gdi_DeleteDC()
   return DllCall("gdi32\CreateCompatibleDC", "UPtr", hDC, "UPtr")
}

Gdi_SetDIBcolorTable(hDC, iStart, entries, RGBQUADs) {
    ; hDC
    ; A device context. A DIB must be selected into this device context.

    ; iStart
    ; A zero-based color table index that specifies the first color table entry to set.

    ; entries
    ; The number of color table entries to set.

    ; RGBQUADs
    ; A pointer to an array of RGBQUAD structures containing new color information for the DIB's color table.

    ; Return value
    ; If the function succeeds, the return value is the number of color table entries that the function sets.
    ; If the function fails, the return value is zero.

    ; Remarks
    ; This function should be called to set the color table for DIBs that use 1, 4, or 8 bpp. The BitCoun
    ; member of a bitmap's associated bitmap information header structure.

   Return DllCall("SetDIBColorTable", "UPtr",hDC, "Int", iStart, "Int", entries, "UPtr",&RGBQUADs)
}

Gdi_CreateCompatibleBitmap(hDC, w, h) {
  ; The CreateCompatibleBitmap function creates a DDB bitmap compatible
  ; with the device that is associated with the specified device context.

  ; Remarks:
  ; The color format of the bitmap created by the CreateCompatibleBitmap
  ; function matches the color format of the device identified by the hdc
  ; parameter. This bitmap can be selected into any memory device context
  ; that is compatible with the original device.

  ; Because memory device contexts allow both color and monochrome bitmaps, 
  ; the format of the bitmap returned by the CreateCompatibleBitmap
  ; function differs when the specified device context is a memory device
  ; context. However, a compatible bitmap that was created for a nonmemory 
  ; device context always possesses the same color format and uses the same
  ; color palette as the specified device context.

  ; Note: When a memory device context is created, it initially has a
  ; 1-by-1 monochrome bitmap selected into it. If this memory device 
  ; context is used in CreateCompatibleBitmap, the bitmap that is created
  ; is a monochrome bitmap. To create a color bitmap, use the HDC that
  ; was used to create the memory device context.

  ; If an application sets the nWidth or nHeight parameters to zero,
  ; CreateCompatibleBitmap returns the handle to a 1-by-1 pixel,
  ; monochrome bitmap.

  ; If a DIB section, which is a bitmap created by the CreateDIBSection
  ; function, is selected into the device context identified by the hdc
  ; parameter, CreateCompatibleBitmap creates a DIB section.

   return DllCall("gdi32\CreateCompatibleBitmap", "UPtr", hDC, "Int", w, "Int", h)
}

Gdi_GetDCEx(hwnd, flags:=0, hrgnClip:=0) {
   ; Device Context extended flags:
   ; DCX_CACHE = 0x2
   ; DCX_CLIPCHILDREN = 0x8
   ; DCX_CLIPSIBLINGS = 0x10
   ; DCX_EXCLUDERGN = 0x40
   ; DCX_EXCLUDEUPDATE = 0x100
   ; DCX_INTERSECTRGN = 0x80
   ; DCX_INTERSECTUPDATE = 0x200
   ; DCX_LOCKWINDOWUPDATE = 0x400
   ; DCX_NORECOMPUTE = 0x100000
   ; DCX_NORESETATTRS = 0x4
   ; DCX_PARENTCLIP = 0x20
   ; DCX_VALIDATE = 0x200000
   ; DCX_WINDOW = 0x1

   return DllCall("user32\GetDCEx", "UPtr", hwnd, "UPtr", hrgnClip, "int", flags)
}

;#####################################################################################
; Function        Gdi_ReleaseDC
; Description     This function releases a device context (DC), freeing it for use by other applications. The effect of ReleaseDC depends on the type of device context
;
; hdc             Handle to the device context to be released
; hwnd            Handle to the window whose device context is to be released
;
; return          1 = released
;                 0 = not released
;
; notes           The application must call the ReleaseDC function for each call to the GetWindowDC function and for each call to the GetDC function that retrieves a common device context
;                 An application cannot use the ReleaseDC function to release a device context that was created by calling the CreateDC function; instead, it must use the DeleteDC function.
;#####################################################################################

Gdi_ReleaseDC(hDC, hwnd:=0) {
   return DllCall("user32\ReleaseDC", "UPtr", hwnd, "UPtr", hDC)
}

Gdi_DeleteDC(hDC) {
  ; The DeleteDC function deletes the specified device context (DC)
  ; If the function succeeds, the return value is nonzero
  return DllCall("gdi32\DeleteDC", "UPtr", hDC)
}

Gdi_CancelDC(hDC) {
  ; The CancelDC function cancels any pending operation on the specified device context (DC).
  ; If the function succeeds, the return value is nonzero
  return DllCall("gdi32\CancelDC", "UPtr", hDC)
}

Gdi_CreateDC(pDriver, pDevice, pDevMode) {
; pDriver
;    A string that specifies either "DISPLAY" or the name of a specific
;    display device. For printing, we recommend that you pass NULL
;    to pDriver because GDI ignores pDriver for printer devices.
;
; pDevice
;    A string that specifies the name of the specific output device being used,
;    as shown by the Print Manager (for example, Epson FX-80). It is not 
;    the printer model name. The pDevice parameter must be used.
;    To obtain valid names for displays, call EnumDisplayDevices.
;
;    If pDriver is DISPLAY or the device name of a specific display device,
;    then pDevice must be NULL or that same device name. If pDevice is NULL,
;    then a DC is created for the primary display device.
;
;    If there are multiple monitors on the system, calling
;    Gdi_CreateDC("DISPLAY", NULL, NULL, NULL) will create a DC covering
;    all the monitors.
;
; pDevMode
;   A pointer to a DEVMODE structure containing device-specific
;   initialization data for the device driver. The DocumentProperties()
;   function retrieves this structure filled in for a specified device.
;   This parameter must be NULL if the device driver is to use
;   the default initialization (if any) specified by the user.
;
;   If pDriver is DISPLAY, pDevMode must be NULL; GDI then uses the
;   display device's current DEVMODE.
;
; Return value
;   If the function succeeds, the return value is the handle to
;   a DC for the specified device.
;   If the function fails, the return value is NULL.

; Remarks
;   Note that the handle to the DC can only be used by a single
;   thread at any one time.

   return DllCall("gdi32\CreateDC", "Str", pDriver, "Str", pDevice, "Ptr", 0, "UPtr", pDevMode)
}

Gdi_SaveDC(hDC) {
   ; The SaveDC function saves the current state of the specified device context (DC)
   ; by copying data describing selected objects and graphic modes (such as the bitmap,
   ; brush, palette, font, pen, region, drawing mode, and mapping mode) to a context stack.
   ; The SaveDC function can be used any number of times to save any number of instances
   ; of the DC state. A saved state can be restored by using the RestoreDC function.
   ; If the function succeeds, the return value is nonzero
   return DllCall("gdi32\SaveDC", "UPtr", hDC)
}

Gdi_RestoreDC(hDC, nSavedDC) {
   ; The RestoreDC function restores a device context (DC) to the specified state. 
   ; The DC is restored by popping state information off a stack created by earlier
   ; calls to the SaveDC function.

   ; nSavedDC
   ; The saved state to be restored. If this parameter is positive, nSavedDC represents
   ; a specific instance of the state to be restored. If this parameter is negative,
   ; nSavedDC represents an instance relative to the current state. For example,
   ; -1 restores the most recently saved state.

   ; If the function succeeds, the return value is nonzero
   return DllCall("gdi32\RestoreDC", "UPtr", hDC, "Int", nSavedDC)
}

Gdi_WindowFromDC(hDC) {
   ; The WindowFromDC function returns a handle to the window associated with
   ; the specified display device context (DC). Output functions that use
   ; the specified device context draw into this window.
   ; If function fails, the return value is NULL.
   return DllCall("user32\WindowFromDC", "UPtr", hDC)
}

Gdi_CopyImage(hBitmap, imgType:=0, w:=0, h:=0, flags:=0) {
   ; IMGTYPE options
   ; IMAGE_BITMAP  = 0
   ; IMAGE_CURSOR  = 2
   ; IMAGE_ICON    = 1
 
   ; W, H
   ; The desired width and height, in pixels, of the image.
   ; Set W=0 and/or H=0 for original dimension(s).

   ; FLAGS options:
   ; LR_COPYDELETEORG = 0x00000008
   ; Deletes the original image after creating the copy.

   ; LR_COPYFROMRESOURCE = 0x00004000
   ; Tries to reload an icon or cursor resource from the original resource file
   ; rather than simply copying the current image. This is useful for creating a
   ; different-sized copy when the resource file contains multiple sizes of the
   ; resource. Without this flag, CopyImage stretches the original image to the
   ; new size. If this flag is set, CopyImage uses the size in the resource file
   ; closest to the desired size. This will succeed only if hImage was loaded by 
   ; LoadIcon or LoadCursor, or by LoadImage with the LR_SHARED flag.

   ; LR_COPYRETURNORG = 0x00000004
   ; Returns the original hImage if it satisfies the criteria for the copy—that is,
   ; correct dimensions and color depth—in which case the LR_COPYDELETEORG flag is
   ; ignored. If this flag is not specified, a new object is always created.

   ; LR_CREATEDIBSECTION = 0x00002000
   ; If this is set and a new bitmap is created, the bitmap is created as a DIB
   ; section. Otherwise, the bitmap image is created as a device-dependent bitmap.
   ; This flag is only valid if Type is IMAGE_BITMAP.

   ; LR_DEFAULTSIZE = 0x00000040
   ; Uses the width or height specified by the system metric values for cursors
   ; or icons, if the cxDesired or cyDesired values are set to zero. If this flag
   ; is not specified and cxDesired and cyDesired are set to zero, the function
   ; uses the actual resource size. If the resource contains multiple images, the
   ; function uses the size of the first image.

   ; LR_MONOCHROME = 0x00000001
   ; Creates a new monochrome image. 

   ; Return value:
   ; If the function succeeds, the return value is the handle to the newly created image.
   ; If the function fails, the return value is NULL. To get extended error information,
   ; call GetLastError.

   return DllCall("user32\CopyImage", "UPtr", hBitmap, "int", imgType, "int", w, "int", h, "uint", flags)
}

Gdi_DeleteObject(hObject) {
   return DllCall("gdi32\DeleteObject", "UPtr", hObject)
}

Gdi_SelectObject(hDC, obj) {
   return DllCall("gdi32\SelectObject", "UPtr", hDC, "UPtr", obj, "UPtr")
}

; ===================================================================================
; Function:             Gdi_UpdateLayeredWindow
; Description:          Updates a layered window with the handle to the DC of a gdi bitmap
;
; hwnd                  Handle of the layered window to update
; hdc                   Handle to the DC of the GDI bitmap to update the window with
; x, y                  x, y coordinates to place the window
; w, h                  Width and height of the window
; Alpha                 Default = 255 : The transparency (0-255) to set the window transparency
;
; return                If the function succeeds, the return value is nonzero
;
; notes                 If x or y are omitted, the layered window will use its current coordinates
;                       If w or h are omitted, the current width and height will be used
; ===================================================================================

Gdi_UpdateLayeredWindow(hwnd, hDC, x:="", y:="", w:="", h:="", Alpha:=255) {
   if ((x != "") && (y != ""))
      VarSetCapacity(pt, 8, 0), NumPut(x, pt, 0, "UInt"), NumPut(y, pt, 4, "UInt")

   if (w = "") || (h = "")
      Gdi_GetWindowRect(hwnd, W, H)

   return DllCall("user32\UpdateLayeredWindow"
               , "UPtr", hwnd
               , "UPtr", 0
               , "UPtr", ((x = "") && (y = "")) ? 0 : &pt
               , "int64*", w|h<<32
               , "UPtr", hDC
               , "int64*", 0
               , "uint", 0
               , "UInt*", Alpha<<16|1<<24
               , "uint", 2)
}

Gdi_GetWindowRect(hwnd, ByRef W, ByRef H) {
   ; function by GeekDude: https://gist.github.com/G33kDude/5b7ba418e685e52c3e6507e5c6972959
   ; W10 compatible function to find a window's visible boundaries
   ; modified by Marius Șucanto return an array
   If !hwnd
      Return

   size := VarSetCapacity(rect, 16, 0)
   er := DllCall("dwmapi\DwmGetWindowAttribute"
      , "UPtr", hWnd  ; HWND  hwnd
      , "UInt", 9     ; DWORD dwAttribute (DWMWA_EXTENDED_FRAME_BOUNDS)
      , "UPtr", &rect ; PVOID pvAttribute
      , "UInt", size  ; DWORD cbAttribute
      , "UInt")       ; HRESULT

   If er
      DllCall("GetWindowRect", "UPtr", hwnd, "UPtr", &rect, "UInt")

   r := []
   r.x1 := NumGet(rect, 0, "Int"), r.y1 := NumGet(rect, 4, "Int")
   r.x2 := NumGet(rect, 8, "Int"), r.y2 := NumGet(rect, 12, "Int")
   r.w := Abs(max(r.x1, r.x2) - min(r.x1, r.x2))
   r.h := Abs(max(r.y1, r.y2) - min(r.y1, r.y2))
   W := r.w
   H := r.h
   ; ToolTip, % r.w " --- " r.h , , , 2
   Return r
}


Gdi_UpdateWindow(hwnd) {
; The UpdateWindow function updates the client area of the specified
; window by sending a WM_PAINT message to the window if the window's
; update region is not empty. The function sends a WM_PAINT message
; directly to the window procedure of the specified window,
; bypassing the application queue. If the update region is empty,
; no message is sent.
; If the function succeeds, the return value is nonzero.

   return DllCall("user32\UpdateWindow", "UPtr", hwnd)
}

Gdi_GetWindowRegionBox(hwnd) {
   ; The GetWindowRgnBox function retrieves the dimensions of the
   ; tightest bounding rectangle for the window region of a window.

   ; The function will return an object.
     ; obju.x1, obju.y1, obju.x2, obju.y2 - the coordinates of two points representing the bounding box
        ; x1, y1 - top, left corner
        ; x2, y2 - bottom, right corner
     ; obj.E - the value returned by the internal API. It defines the region complexity.
        ; 1 = NULLREGION -  Region is empty.
        ; 2 = SIMPLEREGION - Region is a single rectangle.
        ; 3 = COMPLEXREGION - Region is more than one rectangle.
        ; 0 = An error occurred.

   VarSetCapacity(Rect, 16, 0)
   obju.E := DllCall("gdi32\GetWindowRgnBox", "UPtr", hwnd, "UPtr", &Rect)
   obju.x1 := NumGet(Rect, 0, "uint")
   obju.y1 := NumGet(Rect, 4, "uint")
   obju.x2 := NumGet(Rect, 8, "uint")
   obju.y2 := NumGet(Rect, 12, "uint")
   Return obju
}

Gdi_GetWindowRegion(hwnd, ByRef regionType:=0) {
   ; Return value: the handle of the region of the window given [hwnd]

   ; The regionType value specifies the region's complexity and can
   ; be one of the following values:
        ; 1 = NULLREGION -  Region is empty.
        ; 2 = SIMPLEREGION - Region is a single rectangle.
        ; 3 = COMPLEXREGION - Region is more than one rectangle.
        ; 0 = An error occurred.

    hRgn := Gdi_CreateRectRegion(0, 0, 0, 0)
    If hRgn
       regionType := DllCall("GetWindowRgn", "UPtr", hwnd, "UPtr*", hRgn)
    return hRgn
}

Gdi_GetUpdateRegion(hwnd, hRgn, bErase) {
; The GetUpdateRgn function retrieves the update region of a window
; by copying it into the specified region. The coordinates of the update
; region are relative to the upper-left corner of the window (that is,
; they are client coordinates).

; Parameters
; hWnd = Handle to the window with an update region that is to be retrieved.
; hRgn = Handle to the region to receive the update region; you can create one 
; with Gdi_CreateRectRegion()

; bErase = Specifies whether the window background should be erased and whether
; nonclient areas of child windows should be drawn. If this parameter is 0,
; no drawing is done.

; Return value
; The return value indicates the complexity of the resulting region.

    return DllCall("GetUpdateRgn", "UPtr", hwnd, "UPtr*", hRgn, "int", bErase)
}

Gdi_SetWindowRegion(hwnd, hRgn, bRedraw) {
; Return value: if the function succeeds, the return value is nonzero.

; The SetWindowRgn function sets the window region of a window.
; The window region determines the area within the window where
; the system permits drawing. The system does not display any
; portion of a window that lies outside of the window region.

; The coordinates of a window's window region are relative to
; the upper-left corner of the window, not the client area of
; the window.

; Note:
; If the window layout is right-to-left (RTL), the coordinates are relative to the upper-right corner of the window. See Window Layout and Mirroring.
 
; After a successful call to SetWindowRgn, the system owns the region
; specified by the region handle hRgn. The system does not make a
; copy of the region. Thus, you should not make any further function
; calls with this region handle. In particular, do not delete this
; region handle. The system deletes the region handle when it 
; no longer needed.

; To obtain the window region of a window,
; call the Gdi_GetWindowRegion function.

    return DllCall("SetWindowRgn", "UPtr", hwnd, "UPtr", hRgn, "uint", bRedraw)
}

Gdi_ValidateRegion(hwnd, hRgn) {
; Return value: if the function succeeds, the return value is nonzero.

; The ValidateRgn function validates the client area within a
; region by removing the region from the current update region
; of the specified window.

    return DllCall("ValidateRgn", "UPtr", hwnd, "UPtr", hRgn)
}

Gdi_InvalidateRegion(hwnd, hRgn, bErase:=0) {
; Return value: The return value is always nonzero. [how sad]

; The InvalidateRgn function invalidates the client area within
; the specified region by adding it to the current update region
; of a window. The invalidated region, along with all other areas
; in the update region, is marked for painting when the next
; WM_PAINT message occurs.
    return DllCall("InvalidateRgn", "UPtr", hwnd, "UPtr", hRgn, "UInt", bErase)
}

Gdi_InvalidateRect(hwnd, x, y, w, h, bErase:=0) {
; The InvalidateRect function adds a rectangle to the specified
; window's update region. The update region represents 
; the portion of the window's client area that must be redrawn.
; If the function succeeds, the return value is nonzero.

; Remarks
; The invalidated areas accumulate in the update region until
; the region is processed when the next WM_PAINT message occurs
; or until the region is validated by using the ValidateRect or
; ValidateRgn function.

; The system sends a WM_PAINT message to a window whenever its
; update region is not empty and there are no other messages in
; the application queue for that window.

; If the bErase parameter is TRUE for any part of the update
; region, the background is erased in the entire region, not
; just in the specified part.

    VarSetCapacity(Rect, 16, 0)
    NumPut(X, Rect, 0, "UInt")
    NumPut(Y, Rect, 4, "UInt")
    NumPut(X + W, Rect, 8, "UInt")
    NumPut(Y + H, Rect, 12, "UInt")

    return DllCall("InvalidateRect", "UPtr", hwnd, "UPtr", &Rect, "UInt", bErase)
}

Gdi_ValidateRect(hwnd, x, y, w, h) {
; The ValidateRect function validates the client area within a rectangle by
; removing the rectangle from the update region of the specified window.
; If the function succeeds, the return value is nonzero.

; Remarks
; The BeginPaint function automatically validates the entire client area.
; Neither the ValidateRect nor ValidateRgn function should be called 
; if a portion of the update region must be validated before the next
; WM_PAINT message is generated.

; The system continues to generate WM_PAINT messages until the current 
; update region is validated.

    VarSetCapacity(Rect, 16, 0)
    NumPut(X, Rect, 0, "UInt")
    NumPut(Y, Rect, 4, "UInt")
    NumPut(X + W, Rect, 8, "UInt")
    NumPut(Y + H, Rect, 12, "UInt")

    return DllCall("ValidateRect", "UPtr", hwnd, "UPtr", &Rect)
}

Gdi_BeginPaint(hwnd, ByRef PaintStruct, ByRef X, ByRef Y, ByRef W, ByRef H) {
; The BeginPaint function prepares the specified window for painting and
; fills a PAINTSTRUCT structure with information about the painting.

; Return value
; If the function succeeds, the return value is the handle to a display device
; context for the specified window.

; If the function fails, the return value is NULL, indicating that no display
; device context is available.

; Remarks
; The BeginPaint function automatically sets the clipping region of the device
; context to exclude any area outside the update region. The update region 
; is set by the InvalidateRect or InvalidateRgn function and by the system after
; sizing, moving, creating, scrolling, or any other operation that affects the 
; client area. If the update region is marked for erasing, BeginPaint sends
; a WM_ERASEBKGND message to the window.

; An application should not call BeginPaint except in response to a WM_PAINT
; message. Each call to BeginPaint must have a corresponding call to the EndPaint function.

    VarSetCapacity(PaintStruct, A_PtrSize + 60, 0)
    hWindowDC := DllCall("BeginPaint", "UPtr", hwnd, "UPtr", &PaintStruct, "UPtr")
    ;obtain dimensions of update region
    If hWindowDC
    {
       X := NumGet(PaintStruct,A_PtrSize + 4,"UInt")
       Y := NumGet(PaintStruct,A_PtrSize + 8,"UInt")
       W := NumGet(PaintStruct,A_PtrSize + 12,"UInt") - X
       H := NumGet(PaintStruct,A_PtrSize + 16,"UInt") - Y
    } Else PaintStruct := ""

    return hWindowDC
}

Gdi_EndPaint(hwnd, PaintStruct) {
    return DllCall("EndPaint", "UPtr", Hwnd, "UPtr", &PaintStruct)
}

;#####################################################################################
; Function        Gdi_BitBlt
; Description     The function performs a bit-block transfer of the color data corresponding to a rectangle
;                 of pixels from the specified source device context into a destination device context.
;
; dDC             handle to destination DC
; dX, dY          x, y coordinates of the destination upper-left corner
; dW, dH          width and height of the area to copy
; sDC             handle to source DC
; sX, sY          x, y coordinates of the source upper-left corner
; raster          raster operation code
;
; return          If the function succeeds, the return value is nonzero
;
; notes           If no raster operation is specified, then SRCCOPY is used, which copies the source directly to the destination rectangle
;
;                 BitBlt only does clipping on the destination DC. If the color formats of the source and destination device contexts do
;                 not match, the BitBlt function converts the source color format to match the destination format.

; Raster operation codes:
; BLACKNESS          = 0x00000042
; NOTSRCERASE        = 0x001100A6
; NOTSRCCOPY         = 0x00330008
; SRCERASE           = 0x00440328
; DSTINVERT          = 0x00550009
; PATINVERT          = 0x005A0049
; SRCINVERT          = 0x00660046
; SRCAND             = 0x008800C6
; MERGEPAINT         = 0x00BB0226
; MERGECOPY          = 0x00C000CA
; SRCCOPY            = 0x00CC0020
; SRCPAINT           = 0x00EE0086
; PATCOPY            = 0x00F00021
; PATPAINT           = 0x00FB0A09
; WHITENESS          = 0x00FF0062
; CAPTUREBLT         = 0x40000000
; NOMIRRORBITMAP     = 0x80000000

Gdi_BitBlt(dDC, dX, dY, dW, dH, sDC, sX, sY, raster:="") {
   ; This function works only with GDI hBitmaps that are Device-Dependent Bitmaps [DDB].
   return DllCall("gdi32\BitBlt"
               , "UPtr", dDC
               , "int", dX, "int", dY
               , "int", dW, "int", dH
               , "UPtr", sDC
               , "int", sX, "int", sY
               , "uint", Raster ? Raster : 0xCC0020) ; src_copy by defalut
}

Gdi_TransparentBlt(dDC, dX, dY, dW, dH, sDC, sX, sY, sW, sH, Color) {
   ; The TransparentBlt function performs a bit-block transfer of the color
   ; data corresponding to a rectangle of pixels from the specified source
   ; device context into a destination device context.

   ; Color parameter designates the RGB color in the source bitmap to treat as transparent.

   ; Return value
   ; If the function succeeds, the return value is 1. Otherwise, the return value is 0.

   ; Remarks
   ; The function works with compatible bitmaps (DDBs) and supports all formats
   ; of source bitmaps. However, for 32 bpp bitmaps, it just copies the alpha
   ; value over. Use AlphaBlend to specify 32 bits-per-pixel bitmaps
   ; with transparency.

   ; If the source and destination rectangles are not the same size, the source
   ; bitmap is stretched to match the destination rectangle. When the
   ; SetStretchBltMode function is used, the iStretchMode modes of BLACKONWHITE
   ; and WHITEONBLACK are converted to COLORONCOLOR for the TransparentBlt function.

   ; The destination device context specifies the transformation type for the
   ; destination coordinates. The source device context specifies the
   ; transformation type for the source coordinates.

   ; TransparentBlt does not mirror a bitmap if either the width or height,
   ; of either the source or destination, is negative.

   return DllCall("Msimg32\TransparentBlt"
               , "UPtr", dDC
               , "int", dX, "int", dY
               , "int", dW, "int", dH
               , "UPtr", sDC
               , "int", sX, "int", sY
               , "int", sW, "int", sH
               , "uint", Color)
}

Gdi_MaskBlt(dDC, dX, dY, dW, dH, sDC, sX, sY, hbmpMask, mX, mY, raster) {
   ; The MaskBlt function uses device-dependent bitmaps.

   ; sDC
   ; A handle to the device context from which the bitmap is to be copied.
   ; It must be zero if the dwRop parameter specifies a raster operation that
   ; does not include a source.

   ; sX
   ; The x-coordinate, in logical units, of the upper-left corner of the 
   ; source bitmap.

   ; sY
   ; The y-coordinate, in logical units, of the upper-left corner of the
   ; source bitmap.

   ; hbmpMask
   ; A handle to the monochrome mask bitmap combined with the color bitmap
   ; in the source device context.
   ; If the mask bitmap is not a monochrome bitmap, an error occurs.

   ; mX
   ; The horizontal pixel offset for the mask bitmap specified by the hbmpMask parameter.

   ; mY
   ; The vertical pixel offset for the mask bitmap specified by the hbmpMask parameter.

   ; raster
   ; The foreground and background ternary raster operation codes (ROPs) that
   ; the function uses to control the combination of source and destination data.
   ; The background raster operation code is stored in the high-order byte of
   ; the high-order word of this value; the foreground raster operation code is
   ; stored in the low-order byte of the high-order word of this value; the
   ; low-order word of this value is ignored, and should be zero. 

   ; For a discussion of foreground and background in the context of this function,
   ; see the following Remarks section.

   ; Remarks

   ; A value of 1 in the mask specified by hbmpMask indicates that the
   ; foreground raster operation code specified by dwRop should be applied
   ; at that location. A value of 0 in the mask indicates that the
   ; background raster operation code specified by dwRop should be applied
   ; at that location.

   ; If the raster operations require a source, the mask rectangle must cover
   ; the source rectangle. If it does not, the function will fail. If the
   ; raster operations do not require a source, the mask rectangle must
   ; cover the destination rectangle. If it does not, the function will fail.

   ; If a rotation or shear transformation is in effect for the source device
   ; context when this function is called, an error occurs. However, other
   ; types of transformation are allowed.

   ; If the color formats of the source, pattern, and destination bitmaps
   ; differ, this function converts the pattern or source format, or both,
   ; to match the destination format.

   ; When an enhanced metafile is being recorded, an error occurs (and the
   ; function returns FALSE) if the source device context identifies an
   ; enhanced-metafile device context.

   ; Not all devices support the MaskBlt function. An application should
   ; call the GetDeviceCaps function with the nIndex parameter as RC_BITBLT
   ; to determine whether a device supports this function.

   ; If no mask bitmap is supplied, this function behaves exactly like BitBlt,
   ; using the foreground raster operation code.

   ; ICM: No color management is performed when blits occur.
   ; When used in a multiple monitor system, both hdcSrc and hdcDest must
   ; refer to the same device or the function will fail. To transfer data
   ; between DCs for different devices, convert the memory bitmap
   ; (compatible bitmap, or DDB) to a DIB by calling GetDIBits. To display
   ; the DIB to the second device, call SetDIBits or StretchDIBits.

   return DllCall("gdi32\MaskBlt"
               , "UPtr", dDC
               , "int", dX, "int", dY
               , "int", dW, "int", dH
               , "UPtr", sDC
               , "int", sX, "int", sY
               , "UPtr", hbmpMask
               , "int", mX, "int", mY
               , "uint", raster)
}

;#####################################################################################

; Function        Gdi_StretchBlt
; Description     The StretchBlt function copies a bitmap from a source rectangle into a destination rectangle,
;                 stretching or compressing the bitmap to fit the dimensions of the destination rectangle, if necessary.
;                 The system stretches or compresses the bitmap according to the stretching mode currently set in the destination device context.
;
; ddc             handle to destination DC
; dX, dY          x, y coordinates of the destination upper-left corner
; dW, dH          width and height of the destination rectangle
; sdc             handle to source DC
; sX, sY          x, y coordinates of the source upper-left corner
; sW, sH          width and height of the source rectangle
; Raster          raster operation code
;
; return          If the function succeeds, the return value is nonzero
;
; notes           If no raster operation is specified, then SRCCOPY is used. It uses the same raster operations as BitBlt

Gdi_StretchBlt(dDC, dx, dy, dw, dh, sDC, sx, sy, sw, sh, Raster:="") {
   return DllCall("gdi32\StretchBlt"
               , "UPtr", dDC
               , "int", dX, "int", dY
               , "int", dW, "int", dH
               , "UPtr", sdC
               , "int", sX, "int", sY
               , "int", sW, "int", sH
               , "uint", Raster ? Raster : 0x00CC0020)
}

;#####################################################################################

; Function           Gdi_SetStretchBltMode
; Description        The SetStretchBltMode function sets the bitmap stretching mode in the specified device context
;
; hdc                handle to the DC
; iStretchMode       The stretching mode, describing how the target will be stretched
;
; return             If the function succeeds, the return value is the previous stretching mode. If it fails it will return 0
;

Gdi_SetStretchBltMode(hDC, iStretchMode:=4) {
  ; iStretchMode options:
  ; BLACKONWHITE = 1
  ; WHITEONBLACK = 2
  ; COLORONCOLOR = 3
  ; HALFTONE = 4
  ; STRETCH_ANDSCANS = BLACKONWHITE
  ; STRETCH_DELETESCANS = COLORONCOLOR
  ; STRETCH_HALFTONE = HALFTONE
  ; STRETCH_ORSCANS = WHITEONBLACK
  return DllCall("gdi32\SetStretchBltMode", "UPtr", hDC, "int", iStretchMode)
}


;#####################################################################################

; Function           Gdi_BitmapFromHWND
; Description        Uses PrintWindow to get a handle to the specified window and return a bitmap from it
;
; hwnd               handle to the window to get a bitmap from
; clientOnly         capture only the client area of the window, without title bar and border
;
; return             If the function succeeds, the return value is a pointer to a GDI bitmap.

Gdi_BitmapFromHWND(hwnd, clientOnly:=0) {
   ; Restore the window if minimized! Must be visible for capture.
   if DllCall("IsIconic", "UPtr", hwnd)
      DllCall("ShowWindow", "UPtr", hwnd, "int", 4)

   Static Ptr := "UPtr"
   thisFlag := 0
   If (clientOnly=1)
   {
      VarSetCapacity(rc, 16, 0)
      DllCall("GetClientRect", "UPtr", hwnd, "ptr", &rc)
      Width := NumGet(rc, 8, "int")
      Height := NumGet(rc, 12, "int")
      thisFlag := 1
   } Else Gdi_GetWindowRect(hwnd, Width, Height)

   hbm := Gdi_CreateDIBSection(Width, Height)
   hDC := Gdi_CreateCompatibleDC()
   obm := Gdi_SelectObject(hdc, hbm)
   Gdi_PrintWindow(hwnd, hDC, 2 + thisFlag)
   Gdi_SelectObject(hDC, obm)
   Gdi_DeleteDC(hDC)
   return hbm
}


;#####################################################################################

; Function           Gdi_CreateDIBSection
; Description        The CreateDIBSection function creates a DIB (Device Independent Bitmap) that applications can write to directly
;
; w, h               width and height of the bitmap to create
; hdc                a handle to the device context to use the palette from
; bpp                bits per pixel (32 = ARGB)
; ppvBits            A pointer to a variable that receives a pointer to the location of the DIB bit values
;
; return             returns a DIB. A GDI bitmap.
;
; notes              ppvBits will receive the location of the pixels in the DIB

Gdi_CreateDIBSection(w, h, hDC:="", bpp:=32, ByRef ppvBits:=0, Usage:=0, hSection:=0, Offset:=0) {
; A GDI function that creates a new hBitmap,
; a device-independent bitmap [DIB].
; A DIB consists of two distinct parts:
; a BITMAPINFO structure describing the dimensions
; and colors of the bitmap, and an array of bytes
; defining the pixels of the bitmap. 

   hDC2 := hDC ? hDC : Gdi_GetDC()
   VarSetCapacity(bi, 40, 0)
   NumPut(40, bi, 0, "uint")
   NumPut(w, bi, 4, "uint")
   NumPut(h, bi, 8, "uint")
   NumPut(1, bi, 12, "ushort")
   NumPut(bpp, bi, 14, "ushort")
   NumPut(0, bi, 16, "uInt")

   hbm := DllCall("gdi32\CreateDIBSection"
               , "UPtr", hDC2
               , "UPtr", &bi    ; BITMAPINFO
               , "uint", Usage
               , "UPtr*", ppvBits
               , "UPtr", hSection
               , "uint", OffSet, "UPtr")

   if !hdc
      Gdi_ReleaseDC(hdc2)
   return hbm
}

;#####################################################################################

; Function           Gdi_PrintWindow
; Description        The PrintWindow function copies a visual window into the specified device context (DC), typically a printer DC
;
; hwnd               A handle to the window that will be copied
; hdc                A handle to the device context
; Flags              Drawing options
;
; return             If the function succeeds, it returns a nonzero value
;
; PW_CLIENTONLY      = 1

Gdi_PrintWindow(hwnd, hDC, Flags:=2) {
   ; set Flags to 2, to capture hardware accelerated windows
   ; this only applies on Windows 8.1 and later versions.

   return DllCall("user32\PrintWindow", "UPtr", hwnd, "UPtr", hDC, "uint", Flags)
}

Gdi_SetPixel(hDC, x, y, Color, Fast:=0) {
   return DllCall("gdi32\SetPixel" (Fast=1 ? "V" : ""), "UPtr", hDC, "Int", x, "Int", y, "UInt", Color)
}

Gdi_GetPixel(hDC, x, y) {
   ; returns COLORREF
   return DllCall("gdi32\GetPixel", "UPtr", hDC, "Int", x, "Int", y)
}

Gdi_ColorRef2RGB(c) {
  ; c integer must be a COLORREF
  ; returns a RGB HEX value: eg. 22FFAA
  r := c >> 16 & 0xff | c & 0xff00 | (c & 0xff) << 16
  c := "000000"
  DllCall("msvcrt\sprintf", "AStr", c, "AStr", "%06X", "UInt", r, "CDecl")
  Return c
}

Gdi_GetPixelColor(hDC, x, y, Format) {
   c := Gdi_GetPixel(hDC, x, y)
   If (c="")
      Return

   clr := Gdi_ColorRef2RGB(c)
   If (format=1)  ; in RGB [HEX; 00-FF] with 0x prefix
   {
      Return "0x" clr
   } Else If (format=2)  ; in RGB [0-255]
   {
      z := c >> 16 & 0xff | c & 0xff00 | (c & 0xff) << 16
      R := (0xff0000 & z) >> 16
      G := (0x00ff00 & z) >> 8
      B := 0x0000ff & z
      Return [R, G, B]
   } Else If (format=3)  ; in BGR [HEX; 00-FF] with 0x prefix
   {
      Return "0x" SubStr(clr, 5, 2) SubStr(clr, 3, 2) SubStr(clr, 1, 2)
   } Else Return clr
}

;------------------------------
;
; Function: Gdi_AddFontResource
;
; Description:
;   Add one or more fonts from a font file (Ex: "MySpecialFont.ttf") to the
;   system font table.
;
; Parameters:
;
;   fntFile - The full path and name of the font file.
;   isPrivate - If set to TRUE, only the process that called this function can
;       use the added font(s).
;
;   isHidden - If set to TRUE, the added font(s) cannot be enumerated, i.e. not
;       included when any program requests a list of fonts from the OS.  The
;       default is FALSE, i.e. not hidden.
;
; Returns:
;   The number of the fonts added if successful, otherwise FALSE.
;
; Remarks:
;
;   All fonts added using this function are temporary.  If the isPrivate
;   parameter is set to 1, the added font(s) are automatically removed when
;   the process that added the font(s) ends. If isPrivate is set to 0, the font(s)
;   are only available for the current session.  When the system restarts, the
;   font(s) will not be present. If desired, use <Fnt_RemoveFontFile> to remove
;   the font(s) added by this function.
;
; Supported file types:
; .fon -Font resource file.
; .fnt - Raw bitmap font file.
; .ttf - Raw TrueType file.
; .ttc - East Asian Windows: TrueType font collection.
; .fot - TrueType resource file.
; .otf - PostScript OpenType font.
; .mmm - multiple master Type1 font resource file. It must be used with .pfm and .pfb files.
; .pfb - Type 1 font bits file. It is used with a .pfm file.
; .pfm - Type 1 font metrics file. It is used with a .pfb file. 
;
;-------------------------------------------------------------------------------

Gdi_AddFontResource(fntFile, isPrivate, isHidden:=0) {
    Static WM_FONTCHANGE :=0x1D
          ,HWND_BROADCAST:=0xFFFF

    flags := 0
    if isPrivate
       fFlags |= 0x10 ; FR_PRIVATE

    if isHidden
       flags |= 0x20 ; FR_NOT_ENUM

    RC := DllCall("gdi32\AddFontResourceEx", "Str", fntFile, "UInt", flags, "UInt", 0)

    ;-- If one or more fonts were added, notify all top-level windows that the
    ;   pool of font resources has changed.
    if RC
       SendMessage, WM_FONTCHANGE,0,0,,ahk_id %HWND_BROADCAST%,,,,1000
            ;-- Wait up to (but no longer than) 1000 ms for all windows to
            ;   respond to the message.

    Return RC
}

;------------------------------
;
; Function: Gdi_RemoveFontResource
;
; Description:
;   Remove the font(s) added with <Gdi_AddFontResource>.
;
; Parameters:
;   Same parameters as <Gdi_AddFontResource>. Use the same parameter values that
;   were used to add the font(s).
;
; Returns:
;   The number of the fonts removed if successful, otherwise FALSE.
;
; Remarks:
;   See the *Remarks* section of <Gdi_AddFontFile> for more information.
;
;-------------------------------------------------------------------------------
Gdi_RemoveFontResource(fntFile, isPrivate, isHidden:=0) {
    Static WM_FONTCHANGE :=0x1D
          ,HWND_BROADCAST:=0xFFFF

    flags := 0
    if isPrivate
       fFlags |= 0x10 ; FR_PRIVATE

    if isHidden
       flags |= 0x20 ; FR_NOT_ENUM

    RC := DllCall("gdi32\RemoveFontResourceEx", "Str", fntFile, "UInt", flags, "UInt", 0)

    ;-- If one or more fonts were removed, notify all top-level windows that the
    ;   pool of font resources has changed.
    if RC
       SendMessage, WM_FONTCHANGE,0,0,,ahk_id %HWND_BROADCAST%,,,,1000
            ;-- Wait up to 1000 ms for all windows to respond to the message

    Return RC
}

;------------------------------------------------------------------
;
; Function: Gdi_CreateFontByName
;
; Description:
;   Creates a logical font by font name.
;
; Parameters:
    ; - Font quality options:
    ; DEFAULT_QUALITY = 0
    ; DRAFT_QUALITY = 1
    ; PROOF_QUALITY = 2           ;-- AutoHotkey default
    ; NONANTIALIASED_QUALITY = 3
    ; ANTIALIASED_QUALITY = 4
    ; CLEARTYPE_QUALITY = 5

    ; - Font weight options:
    ; FW_DONTCARE = 0
    ; FW_THIN = 100
    ; FW_EXTRALIGHT = 200
    ; FW_LIGHT = 300
    ; FW_NORMAL / REGULAR = 400
    ; FW_MEDIUM = 500
    ; FW_SEMIBOLD = 600
    ; FW_BOLD = 700
    ; FW_EXTRABOLD = 800
    ; FW_HEAVY / BLACK = 900
;
; Returns:
;   A handle to a logical font if succesful.
;
; Remarks:
;   When no longer needed, call <Gdi_DeleteObject> to delete the font.
;
;-------------------------------------------------------------------------------

Gdi_CreateFontByName(fntName, ptsSize, weight:=0, italica:=0, strikeout:=0, underline:=0, quality:=4, angle:=0) {
    fntName := Trim(fntName," `f`n`r`t`v")
    fntHeight := calcFntHeightFromPtsSize(ptsSize)
    hFont := Gdi_CreateFont(fntHeight,,angle,angle,weight,italica,underline,strikeout,,,,quality,,,fntname)
    ; ToolTip, % hFont " = lol" , , , 2
    Return hFont
}

;------------------------------------------------------------------
;
; Function: Gdi_CreateFontByFamily
;
; Description:
;   Creates a logical font specified by family and/or pitch.
;
; Parameters:
    ; - Font family options
    ; DONTCARE = 0
    ; ROMAN = 1
    ; SWISS = 2
    ; MODERN = 3
    ; SCRIPT = 4
    ; DECORATIVE = 5

    ; - Font pitch options
    ; DEFAULT_PITCH = 0 
    ; FIXED_PITCH = 1
    ; VARIABLE_PITCH = 2 
    ; MONO_FONT = 8

    ; - Font quality options:
    ; DEFAULT_QUALITY = 0
    ; DRAFT_QUALITY = 1
    ; PROOF_QUALITY = 2           ;-- AutoHotkey default
    ; NONANTIALIASED_QUALITY = 3
    ; ANTIALIASED_QUALITY = 4
    ; CLEARTYPE_QUALITY = 5

    ; - Font weight options:
    ; FW_DONTCARE = 0
    ; FW_THIN = 100
    ; FW_EXTRALIGHT = 200
    ; FW_LIGHT = 300
    ; FW_NORMAL / REGULAR = 400
    ; FW_MEDIUM = 500
    ; FW_SEMIBOLD = 600
    ; FW_BOLD = 700
    ; FW_EXTRABOLD = 800
    ; FW_HEAVY / BLACK = 900
;
; Returns:
;   A handle to a logical font if succesful.
;
; Remarks:
;   When no longer needed, call <Gdi_DeleteObject> to delete the font.
;
;-------------------------------------------------------------------------------

Gdi_CreateFontByFamily(fntFamily, fntPitch, ptsSize, weight:=0, italica:=0, strikeout:=0, underline:=0, quality:=4, angle:=0) {
    fntName := Trim(fntName," `f`n`r`t`v")
    fntHeight := calcFntHeightFromPtsSize(ptsSize)
    hFont := Gdi_CreateFont(fntHeight,,angle,angle,weight,italica,underline,strikeout,,,,quality,"0x" fntFamily,fntPitch)
    ; ToolTip, % hFont " = lol" , , , 2
    Return hFont
}

Gdi_CreateFont(height:="",width:=0,escapmnt:=0,orient:=0,weight:=0,italica:=0,underline:=0,strikeout:=0,charset:=1,outPrecis:=4,clipPrecis:=0,quality:=4,fntFamily:=0x0,pitch:=0,fntname:="") {
; The output precision [outPrecis] defines how closely the output must match the requested
; font's height, width, character orientation, escapement, pitch, and
; font type. It can be one of the following values.

; outPrecis options:
   ; OUT_DEFAULT_PRECIS = 0
      ; The default font mapper behavior.
   ; OUT_STRING_PRECIS = 1
      ; This value is not used by the font mapper, but it is returned when raster fonts are enumerated.
   ; OUT_CHARACTER_PRECIS = 2
      ; Unused.
   ; OUT_STROKE_PRECIS = 3
      ; This value is not used by the font mapper, but it is returned when TrueType, other outline-based fonts, and vector fonts are enumerated.
   ; OUT_TT_PRECIS = 4
      ; A TrueType font will be chosen when the system has multiple fonts with the same name. 
   ; OUT_DEVICE_PRECIS = 5
      ; A device specific font will be chosen when the system has multiple fonts with the same name.
   ; OUT_RASTER_PRECIS = 6
      ; A raster font will be chosen when the system has multiple fonts with the same name.
   ; OUT_TT_ONLY_PRECIS = 7
      ; The font mapper will choose only TrueType fonts. If there are no TrueType
      ; fonts installed in the system, the font mapper returns to default behavior.
   ; OUT_OUTLINE_PRECIS = 8
      ; The font mapper will choose TrueType and other outline-based fonts.
   ; OUT_SCREEN_OUTLINE_PRECIS = 9
      ; Unknown.
   ; OUT_PS_ONLY_PRECIS = 10
      ; The font mapper will choose only PostScript fonts. If there are no PostScript
      ; fonts installed in the system, the font mapper returns to default behavior.


; The clipping precision [clipPrecis] defines how to clip characters that are partially outside
; the clipping region. It can be one or more of the following values.

; clipPrecis options:
   ; CLIP_DEFAULT_PRECIS = 0
   ; CLIP_CHARACTER_PRECIS = 1
   ; CLIP_STROKE_PRECIS = 2
   ; CLIP_LH_ANGLES = (1<<4)


; charset options:
   ; ANSI_CHARSET = 0
   ; DEFAULT_CHARSET = 1
   ; SYMBOL_CHARSET = 2
   ; MAC_CHARSET = 77
   ; SHIFTJIS_CHARSET = 128
   ; HANGEUL_CHARSET = 129
   ; HANGUL_CHARSET = 129
   ; GB2312_CHARSET = 134
   ; CHINESEBIG5_CHARSET = 136
   ; JOHAB_CHARSET = 130
   ; HEBREW_CHARSET = 177
   ; ARABIC_CHARSET = 178
   ; GREEK_CHARSET = 161
   ; TURKISH_CHARSET = 162
   ; VIETNAMESE_CHARSET = 163
   ; BALTIC_CHARSET = 186
   ; RUSSIAN_CHARSET = 204
   ; THAI_CHARSET = 222
   ; EASTEUROPE_CHARSET = 238
   ; OEM_CHARSET = 255

   hFont := DllCall("gdi32\CreateFont"
         ,"Int",height                      ;-- nHeight [font size]
         ,"Int",width                       ;-- nWidth
         ,"Int",escapmnt                    ;-- nEscapement (0=normal horizontal)
         ,"Int",orient                      ;-- nOrientation
         ,"Int",weight                      ;-- fnWeight
         ,"UInt",italica                    ;-- fdwItalic
         ,"UInt",underline                  ;-- fdwUnderline
         ,"UInt",strikeout                  ;-- fdwStrikeOut
         ,"UInt",charset                    ;-- fdwCharSet
         ,"UInt",outPrecis                  ;-- fdwOutputPrecision
         ,"UInt",clipPrecis                 ;-- fdwClipPrecision
         ,"UInt",quality                    ;-- fdwQuality
         ,"UInt",(fntFamily<<4)|pitch       ;-- fdwPitchAndFamily
         ,"Str",SubStr(fntName,1,31))       ;-- lpszFace
   Return hFont
}

Gdi_CloneFont(hFont) {
   VarSetCapacity(LOGFONT,A_IsUnicode ? 92:60, 0)
   DllCall("GetObject", "UPtr", hFont, "Int", A_IsUnicode ? 92:60, "UPtr",&LOGFONT)
   Return DllCall("gdi32\CreateFontIndirect","UPtr",&LOGFONT, "UPtr")
}

Gdi_GetFontMetrics(hFont) {
    Static TMPF_FIXED_PITCH := 0x1
                ;-- If this bit is set, the font is a variable pitch font.  If
                ;   this bit is clear, the font is a fixed pitch font.  Note
                ;   very carefully that those meanings are the opposite of what
                ;   the constant name implies.
          , TMPF_VECTOR     := 0x2
          , TMPF_TRUETYPE   := 0x4
          , TMPF_DEVICE     := 0x8

    fntInfos := []
    hDC := Gdi_CreateDC("DISPLAY", nulla, none)
    LogPixelsY := Gdi_GetDeviceCaps(hDC, 90) ; LOGPIXELSY
    Gdi_DeleteDC(hDC)

    pTM := Gdi_GetTextMetrics(hFont)

    fntInfos.name := Gdi_GetTextFace(hFont)
    fntInfos.height := NumGet(pTM+0, 0, "Int")
    fntInfos.ascent := NumGet(pTM+0, 4, "Int")
    fntInfos.descent := NumGet(pTM+0, 8, "Int")
    fntInfos.internalLeading := NumGet(pTM+0, 12, "Int")
    fntInfos.externalLeading := NumGet(pTM+0, 16, "Int")
    fntInfos.avgCharWidth := NumGet(pTM+0, 20, "Int")
    fntInfos.maxCharWidth := NumGet(pTM+0, 24, "Int")
    fntInfos.weight := NumGet(pTM+0, 28, "Int")
    fntInfos.overhang := NumGet(pTM+0, 32, "Int")
    fntInfos.itaiic := NumGet(pTM+0, A_IsUnicode ? 52:48, "UChar")
    fntInfos.underline := NumGet(pTM+0, A_IsUnicode ? 53:49, "UChar")
    fntInfos.strikeout := NumGet(pTM+0, A_IsUnicode ? 54:50, "UChar")
    fntInfos.fixedPitch := (NumGet(pTM+0, A_IsUnicode ? 55:51, "UChar") & TMPF_FIXED_PITCH) ? 0 : 1
    fntInfos.vectorFont := (NumGet(pTM+0, A_IsUnicode ? 55:51, "UChar") & TMPF_VECTOR) ? 0 : 1
    fntInfos.trueType := (NumGet(pTM+0, A_IsUnicode ? 55:51, "UChar") & TMPF_TRUETYPE) ? 0 : 1
    fntInfos.deviceFont := (NumGet(pTM+0, A_IsUnicode ? 55:51, "UChar") & TMPF_DEVICE) ? 0 : 1
    fntInfos.charSet := NumGet(pTM+0, A_IsUnicode ? 56:52, "UChar")
    fntInfos.symbolsFont := (fntInfos.charSet=2) ? 1 : 0
    fntInfos.oemFont := (fntInfos.charSet=255) ? 1 : 0
    fntInfos.verticalFont := (SubStr(fntInfos.name, 1, 1)="@") ? 1 : 0
    fntInfos.ptsSize := Round((fntInfos.height - fntInfos.internalLeading)*72/LogPixelsY)
    Return fntInfos
}

Gdi_GetTextMetrics(hFont) {
    hDC := Gdi_GetDC()
    old_hFont := Gdi_SelectObject(hDC, hFont)

    VarSetCapacity(TEXTMETRIC,A_IsUnicode ? 60:56,0)
    DllCall("gdi32\GetTextMetrics","UPtr", hDC, "UPtr", &TEXTMETRIC)

    Gdi_SelectObject(hDC, old_hFont)
    Gdi_ReleaseDC(hDC, 0)
    Return &TEXTMETRIC
}

Gdi_GetTextFace(hFont) {
    Static MAX_LENGTH := 32     ; in TCHARS

    hDC := Gdi_GetDC()
    old_hFont := Gdi_SelectObject(hDC, hFont)

    VarSetCapacity(pFontName, MAX_LENGTH * (A_IsUnicode ? 2:1), 0)
    DllCall("gdi32\GetTextFace", "UPtr", hDC, "Int", MAX_LENGTH, "Str", pFontName)

    Gdi_SelectObject(hDC, old_hFont)
    Gdi_ReleaseDC(hDC, 0)
    Return pFontName
}

;------------------------------
;
; Function: Gdi_TruncateStringToFitWidth
; Description:
;    Returns a string, truncated if necessary, that is less than or equal to a
;    specified maximum width, in pixels.
;
; Parameters:
;   hFont - Handle to a logical font.
;   p_String - The string to process.
;   p_MaxW - The maximum width for the return string, in pixels.
;
; Returns:
;   The function returns an object with several properties.
    ; obju.stringLength    ; the original string length
    ; obju.fitLength       ; the string length that can fit within given p_MaxW
    ; obju.width           ; The width of the original string
    ; obju.height          ; the height of the original string
    ; obju.dll             ; what the DLL call has returned
;
; Remarks:
;   Common control characters like tab, carriage control, and line feed are
;   always counted but they may or may not contain a size (width and height).
;   Every font is different. Tab characters are never expanded by this function.
;
;-------------------------------------------------------------------------------
Gdi_TruncateStringToFitWidth(hFont, p_String, p_MaxW) {
    if (Gdi_GetObjectType(hFont)!="FONT")
       Return

    hDC := Gdi_GetDC()
    old_hFont := Gdi_SelectObject(hDC, hFont)

    E := Gdi_GetTextExtentExPoint(hDC, p_String, p_MaxW, l_Fit, w, h)
    Gdi_SelectObject(hDC, old_hFont)
    Gdi_ReleaseDC(hDC, 0)
    obju := []
    obju.stringLength := StrLen(p_String)
    obju.fitLength := l_Fit
    obju.width := w
    obju.height := h
    obju.dll := E
    Return obju
}

Gdi_GetTextExtentExPoint(hDC, p_String, p_MaxW, ByRef l_Fit, ByRef W, ByRef H) {
   ; The GetTextExtentExPoint function retrieves the number of characters in
   ; a specified string that will fit within a specified space and fills an
   ; array with the text extent for each of those characters.
   ; (A text extent is the distance between the beginning of the space
   ; and a character that will fit in the space.) This information is
   ; useful for word-wrapping calculations.

   ; If the function succeeds, the return value is nonzero.
   ; If the function fails, the return value is zero.

    VarSetCapacity(a_Fit, 4, 0)
    VarSetCapacity(lpSize, 8, 0)
    E := DllCall("gdi32\GetTextExtentExPoint"
                  ,"UPtr",hDC                                      ;-- hdc
                  ,"Str",p_String                                 ;-- lpszStr
                  ,"Int",StrLen(p_String)                         ;-- cchString
                  ,"Int",p_MaxW                                   ;-- nMaxExtent
                  ,"IntP",a_Fit                                   ;-- lpnFit [out]
                  ,"Ptr",0                                        ;-- alpDx [out]
                  ,"Ptr",&lpSize)                                 ;-- lpSize

    w := NumGet(lpSize, 0, "Int")
    h := NumGet(lpSize, 4, "Int")
    l_Fit := a_Fit
    Return E
}

Gdi_GetDeviceCaps(hDC, index) {
   ; The GetDeviceCaps function retrieves device-specific information for the specified hDC.
   ; Device Parameters [general]
     ; DRIVERVERSION  = 0     ; Device driver version
     ; TECHNOLOGY     = 2     ; Device classification
     ; HORZSIZE       = 4     ; Horizontal size in millimeters
     ; VERTSIZE       = 6     ; Vertical size in millimeters
     ; HORZRES        = 8     ; Horizontal width in pixels
     ; VERTRES        = 10    ; Vertical height in pixels
     ; BITSPIXEL      = 12    ; Number of bits per pixel
     ; PLANES         = 14    ; Number of planes
     ; NUMBRUSHES     = 16    ; Number of brushes the device has
     ; NUMPENS        = 18    ; Number of pens the device has
     ; NUMMARKERS     = 20    ; Number of markers the device has
     ; NUMFONTS       = 22    ; Number of fonts the device has
     ; NUMCOLORS      = 24    ; Number of colors the device supports
     ; PDEVICESIZE    = 26    ; Size required for device descriptor
     ; CURVECAPS      = 28    ; Curve capabilities
     ; LINECAPS       = 30    ; Line capabilities
     ; POLYGONALCAPS  = 32    ; Polygonal capabilities
     ; TEXTCAPS       = 34    ; Text capabilities
     ; CLIPCAPS       = 36    ; Clipping capabilities
     ; RASTERCAPS     = 38    ; Bitblt capabilities
     ; ASPECTX        = 40    ; Length of the X leg
     ; ASPECTY        = 42    ; Length of the Y leg
     ; ASPECTXY       = 44    ; Length of the hypotenuse
     ; LOGPIXELSX     = 88    ; Logical pixels/inch in X
     ; LOGPIXELSY     = 90    ; Logical pixels/inch in Y
     ; SIZEPALETTE   = 104    ; Number of entries in physical palette
     ; NUMRESERVED   = 106    ; Number of reserved entries in palette
     ; COLORRES      = 108    ; Actual color resolution

   ; Printing related DeviceCaps
     ; PHYSICALWIDTH    = 110 ; Physical Width in device units
     ; PHYSICALHEIGHT   = 111 ; Physical Height in device units
     ; PHYSICALOFFSETX  = 112 ; Physical Printable Area x margin
     ; PHYSICALOFFSETY  = 113 ; Physical Printable Area y margin
     ; SCALINGFACTORX   = 114 ; Scaling factor x
     ; SCALINGFACTORY   = 115 ; Scaling factor y

   ; Display driver specific
     ; VREFRESH         = 116  ; Current vertical refresh rate of the display device (for displays only) in Hz
     ; DESKTOPVERTRES   = 117  ; Horizontal width of entire desktop in pixels
     ; DESKTOPHORZRES   = 118  ; Vertical height of entire desktop in pixels
     ; BLTALIGNMENT     = 119  ; Preferred blt alignment

   ; Device Technologies [return values when index = 2]
     ; DT_PLOTTER           = 0   ; Vector plotter
     ; DT_RASDISPLAY        = 1   ; Raster display
     ; DT_RASPRINTER        = 2   ; Raster printer
     ; DT_RASCAMERA         = 3   ; Raster camera
     ; DT_CHARSTREAM        = 4   ; Character-stream, PLP
     ; DT_METAFILE          = 5   ; Metafile, VDM
     ; DT_DISPFILE          = 6   ; Display-file

   ; many other types of information can be retrieved
   ; see https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-getdevicecaps

   return DllCall("gdi32\GetDeviceCaps", "UPtr", hDC, "int", index)
}

Gdi_SetTextCharSpacing(hDC, extra) {
   return DllCall("gdi32\SetTextCharacterExtra", "UPtr", hDC, "Int", extra)
}

Gdi_SetMapMode(hDC, mode) {
   ; The SetMapMode function sets the mapping mode of the specified device
   ; context. The mapping mode defines the unit of measure used to transform
   ; page-space units into device-space units, and also defines the 
   ; orientation of the device's x and y axes.
 
   ; mode options:
   ; MM_TEXT = 1
   ;    Each logical unit is mapped to one device pixel. Positive x is to the right; positive y is down.
   ; MM_LOMETRIC = 2
   ;    Each logical unit is mapped to 0.1 millimeter. Positive x is to the right; positive y is up.
   ; MM_HIMETRIC = 3
   ;    Each logical unit is mapped to 0.01 millimeter. Positive x is to the right; positive y is up.
   ; MM_LOENGLISH = 4
   ;    Each logical unit is mapped to 0.01 inch. Positive x is to the right; positive y is up.
   ; MM_HIENGLISH = 5
   ;    Each logical unit is mapped to 0.001 inch. Positive x is to the right; positive y is up.
   ; MM_TWIPS = 6
   ;    Each logical unit is mapped to one twentieth of a printer's point (1/1440 inch, also called a twip). Positive x is to the right; positive y is up. 
   ; MM_ISOTROPIC = 7
   ;    Logical units are mapped to arbitrary units with equally scaled axes; that is, one unit along the x-axis is equal to one unit along the y-axis. Use the SetWindowExtEx and SetViewportExtEx functions to specify the units and the orientation of the axes. Graphics device interface (GDI) makes adjustments as necessary to ensure the x and y units remain the same size (When the window extent is set, the viewport will be adjusted to keep the units isotropic).
   ; MM_ANISOTROPIC = 8
   ;    Logical units are mapped to arbitrary units with arbitrarily scaled axes. Use the SetWindowExtEx and SetViewportExtEx functions to specify the units, orientation, and scaling.

   return DllCall("Gdi32\SetMapMode", "UPtr", hDC, "Int", mode)
}

Gdi_SetGraphicsMode(hDC, mode) {
   ; modes:
   ; GM_COMPATIBLE = 1
   ;    Sets the graphics mode that is compatible with 16-bit Windows.
   ;    This is the default mode. If this value is specified, the
   ;    application can only modify the world-to-device transform by
   ;    calling functions that set window and viewport extents and
   ;    origins, but not by using SetWorldTransform or 
   ;    ModifyWorldTransform; calls to those functions will fail.
   ;    Examples of functions that set window and viewport extents and
   ;    origins are SetViewportExtEx and SetWindowExtEx.
   ;
   ; GM_ADVANCED = 2
   ;    Sets the advanced graphics mode that allows world transformations.
   ;    This value must be specified if the application will set or modify
   ;    the world transformation for the specified device context. In this
   ;    mode all graphics, including text output, fully conform to the
   ;    world-to-device transformation specified in the device context. 

   return DllCall("gdi32\SetGraphicsMode", "UPtr", hDC, "Int", mode)
}

Gdi_GetBitmapInfo(hBitmap) {
   ; from TheArkive; returns an object with properties

   oi_size := DllCall("GetObject", "UPtr", hBitmap, "Int", 0, "UPtr", 0)  ; get size of struct
   size := VarSetCapacity(oi, (A_PtrSize = 8) ? 104 : 84, 0)           ; always use max size of struct
   DllCall("GetObject", "UPtr", hBitmap, "Int", size, "UPtr", &oi)        ; finally, call GetObject and get data

   obj := []
   ; Main BITMAP struct
   obj.Size := oi_size
   obj.Type := NumGet(oi,0,"UInt")
   obj.Width := NumGet(oi, 4, "UInt")
   obj.Height := NumGet(oi, 8, "UInt")
   obj.Stride := NumGet(oi, 12, "UInt")
   obj.Planes := NumGet(oi, 16, "UShort")
   obj.Bpp := NumGet(oi, 18, "UShort")
   obj.hPtr := NumGet(oi, 24, "UPtr")

   ; BITMAPINFOHEADER struct / DIBSECTION struct
   obj.Struct := NumGet(oi,32,"UInt")
   obj.Width := NumGet(oi, 36, "UInt")
   obj.Height := NumGet(oi, 40, "UInt")
   obj.Planes := NumGet(oi, 44, "UShort")
   obj.Bpp := NumGet(oi, 46, "UShort")
   obj.Comp := NumGet(oi, 48, "UInt")
   obj.Size := NumGet(oi, 52, "UInt")
   obj.ClrUsed := NumGet(oi, 60, "UInt")
   obj.ClrImportant := NumGet(oi, 64, "UInt")
   Return obj
}

Gdi_GetStockObject(stockIndex){
   ; If the function fails, NULL is returned.
   ; Stock Logical Objects
     ; WHITE_BRUSH = 0
     ; LTGRAY_BRUSH = 1
     ; GRAY_BRUSH = 2
     ; DKGRAY_BRUSH = 3
     ; BLACK_BRUSH = 4
     ; NULL_BRUSH = 5
     ; WHITE_PEN = 6
     ; BLACK_PEN = 7
     ; NULL_PEN = 8
     ; OEM_FIXED_FONT = 10
     ; ANSI_FIXED_FONT = 11
     ; ANSI_VAR_FONT = 12
     ; SYSTEM_FONT = 13
     ; DEVICE_DEFAULT_FONT = 14
     ; DEFAULT_PALETTE = 15
     ; SYSTEM_FIXED_FONT = 16
     ; DEFAULT_GUI_FONT = 17
     ; DC_BRUSH = 18
     ; DC_PEN = 19
     ; DC_COLORSPACE = 20
     ; DC_BITMAP = 21
   Return DllCall("gdi32\GetStockObject", "Int", stockIndex)
}

Gdi_ScreenToClient(hWnd, vPosX, vPosY, ByRef vPosX2, ByRef vPosY2) {
; function by jeeswg found on:
; https://autohotkey.com/boards/viewtopic.php?t=38472
  VarSetCapacity(POINT, 8, 0)
  NumPut(vPosX, &POINT, 0, "Int")
  NumPut(vPosY, &POINT, 4, "Int")
  DllCall("user32\ScreenToClient", "UPtr", hWnd, "UPtr", &POINT)
  vPosX2 := NumGet(&POINT, 0, "Int")
  vPosY2 := NumGet(&POINT, 4, "Int")
}

Gdi_ClientToScreen(hWnd, vPosX, vPosY, ByRef vPosX2, ByRef vPosY2) {
; function by jeeswg found on:
; https://autohotkey.com/boards/viewtopic.php?t=38472

  VarSetCapacity(POINT, 8, 0)
  NumPut(vPosX, &POINT, 0, "Int")
  NumPut(vPosY, &POINT, 4, "Int")
  DllCall("user32\ClientToScreen", "UPtr", hWnd, "UPtr", &POINT)
  vPosX2 := NumGet(&POINT, 0, "Int")
  vPosY2 := NumGet(&POINT, 4, "Int")
}

rgb2bgr(rgbHex) {
   ; input must be like AA00FF
   ; output is FF00AA
   r := SubStr(rgbHex, 1, 2)
   g := SubStr(rgbHex, 3, 2)
   b := SubStr(rgbHex, 5, 2)
   Return b . g . r
}

calcFntHeightFromPtsSize(ptsSize) {
   hDC := Gdi_CreateDC("DISPLAY", nulla, none)
   ydpi := Gdi_GetDeviceCaps(hDC, 90) ; LOGPIXELSY
   fntHeight := Round((ptsSize*ydpi)/72)
   Gdi_DeleteDC(hDC)
   ; ToolTip, % ptsSize "==" fntHeight "==" ydpi , , , 2
   Return fntHeight
}

Gdi_GetLibVersion() {
  return 1.30 ; samedi 28 mai 2022 ; 28/05/2022
}


/*
A selection of API functions left to implement from GDI.
===========================================================

6 Region functions:
  CreateEllipticRgnIndirect
  CreatePolygonRgn [priority]
  CreatePolyPolygonRgn
  CreateRectRgnIndirect
  ExtCreateRegion
  GetRegionData

11 Rectangle functions:
  CopyRect
  EqualRect
  InflateRect
  IsRectEmpty
  OffsetRect
  SetRect
  SetRectEmpty
  PtInRect
  IntersectRect
  SubtractRect
  UnionRect

4 Pen, Path functions:
  CreatePenIndirect
  ExtCreatePen
  GetMiterLimit
  GetPath

2 Filled shapes functions:
  Polygon [priority]
  PolyPolygon

4 Brush functions:
  CreateBrushIndirect
  CreateDIBPatternBrushPt
  GetBrushOrgEx
  GetSysColorBrush

17 Color functions:
  AnimatePalette
  CreateHalftonePalette
  CreatePalette
  GetColorAdjustment
  GetNearestColor
  GetNearestPaletteIndex
  GetPaletteEntries
  GetSystemPaletteEntries
  GetSystemPaletteUse
  RealizePalette
  ResizePalette
  SelectPalette
  SetColorAdjustment
  SetPaletteEntries
  SetSystemPaletteUse
  UnrealizeObject
  UpdateColors

12 Bitmap functions:
  CreateBitmapIndirect
  GetBitmapDimensionEx
  GetDIBColorTable
  GetStretchBltMode
  GradientFill
  LoadBitmap / LoadImage [priority]
  PlgBlt
  SetBitmapDimensionEx
  SetDIBits [priority]
  *GetDIBits
  *CreateDIBitmap
  AlphaBlend [priority]
 
10 Line and curve functions:
  GetArcDirection
  LineDDA
  LineDDAProc
  PolyBezier
  PolyBezierTo
  PolyDraw
  Polyline [priority]
  PolylineTo
  PolyPolyline
  SetArcDirection

32 Fonts and text functions:
  AddFontMemResourceEx
  AddFontResource
  CreateFontIndirectEx
  DrawTextEx
  EnumFontFamExProc
  EnumFontFamiliesEx
  ExtTextOut
  GetAspectRatioFilterEx
  GetCharABCWidths
  GetCharABCWidthsFloat
  GetCharABCWidthsI
  GetCharacterPlacement
  GetCharWidth32
  GetCharWidthFloat
  GetCharWidthI
  GetFontData
  GetFontLanguageInfo
  GetFontUnicodeRanges
  GetGlyphIndices
  GetGlyphOutline
  GetKerningPairs
  GetOutlineTextMetrics
  GetRasterizerCaps
  GetTabbedTextExtent
  GetTextAlign
  GetTextCharacterExtra
  GetTextColor
  PolyTextOut
  RemoveFontMemResourceEx
  RemoveFontResource
  SetMapperFlags
  TabbedTextOut

18 Device context functions:
  ChangeDisplaySettings
  ChangeDisplaySettingsEx
  CreateIC
  DeviceCapabilities
  DrawEscape
  EnumDisplayDevices
  EnumDisplaySettings
  EnumDisplaySettingsEx
  EnumObjects
  EnumObjectsProc
  GetCurrentObject
  GetDCBrushColor
  GetDCOrgEx
  GetDCPenColor
  GetLayout
  *GetObject
  ResetDC
  SetLayout

24 Coordinate space and transformation functions:
  CombineTransform
  DPtoLP
  GetCurrentPositionEx
  GetDisplayAutoRotationPreferences
  GetGraphicsMode
  GetMapMode
  GetViewportExtEx
  GetViewportOrgEx
  GetWindowExtEx
  GetWindowOrgEx
  GetWorldTransform
  SetDisplayAutoRotationPreferences
  LPtoDP
  MapWindowPoints
  ModifyWorldTransform
  OffsetViewportOrgEx
  OffsetWindowOrgEx
  ScaleViewportExtEx
  ScaleWindowExtEx
  SetWorldTransform
  SetViewportExtEx
  SetViewportOrgEx
  SetWindowExtEx
  SetWindowOrgEx
        
18 Painting and drawing functions:
  DrawAnimatedRects
  DrawCaption
  DrawEdge
  DrawFocusRect
  DrawFrameControl
  DrawState
  DrawStateProc
  ExcludeUpdateRgn
  GdiFlush
  GdiGetBatchLimit
  GdiSetBatchLimit
  GetBkColor
  GetBkMode
  GrayString
  LockWindowUpdate
  OutputProc
  PaintDesktop
  RedrawWindow

  - priority
  SetBoundsRect
  GetBoundsRect
  GetUpdateRect
                

  https://docs.microsoft.com/en-us/windows/win32/gdi/about-bitmaps
*/
