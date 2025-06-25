﻿; Gdip_All.ahk - GDI+ library compilation of user contributed GDI+ functions
; made by Marius Șucan: https://github.com/marius-sucan/AHK-GDIp-Library-Compilation
; a fork from: https://github.com/mmikeww/AHKv2-Gdip
; based on https://github.com/tariqporter/Gdip
; Supports: AHK_L / AHK_H Unicode/ANSI x86/x64 and AHK v2 alpha
; This file is the AHK v2 edition; for AHK v1.1 compatible edition, please see the repository.
;
; NOTES: The drawing of GDI+ Bitmaps is limited to a size
; of 32767 pixels in either direction (width, height).
; To calculate the largest bitmap you can create:
;    The maximum object size is 2GB = 2,147,483,648 bytes
;    Default bitmap is 32bpp (4 bytes), the largest area we can have is 2GB / 4 = 536,870,912 bytes
;    If we want a square, the largest we can get is sqrt(2GB/4) = 23,170 pixels
;
; Gdip standard library versions:
; by Marius Șucan - gathered user-contributed functions and implemented hundreds of new functions
; - v1.84 on 05/06/2020
; - v1.83 on 24/05/2020
; - v1.82 on 11/03/2020
; - v1.81 on 25/02/2020
; - v1.80 on 11/01/2019
; - v1.79 on 10/28/2019
; - v1.78 on 10/27/2019
; - v1.77 on 10/06/2019
; - v1.76 on 09/27/2019
; - v1.75 on 09/23/2019
; - v1.74 on 09/19/2019
; - v1.73 on 09/17/2019
; - v1.72 on 09/16/2019
; - v1.71 on 09/15/2019
; - v1.70 on 09/13/2019
; - v1.69 on 09/12/2019
; - v1.68 on 09/11/2019
; - v1.67 on 09/10/2019
; - v1.66 on 09/09/2019
; - v1.65 on 09/08/2019
; - v1.64 on 09/07/2019
; - v1.63 on 09/06/2019
; - v1.62 on 09/05/2019
; - v1.61 on 09/04/2019
; - v1.60 on 09/03/2019
; - v1.59 on 09/01/2019
; - v1.58 on 08/29/2019
; - v1.57 on 08/23/2019
; - v1.56 on 08/21/2019
; - v1.55 on 08/14/2019
;
; bug fixes and AHK v2 compatibility by mmikeww and others
; - v1.54 on 11/15/2017
; - v1.53 on 06/19/2017
; - v1.52 on 06/11/2017
; - v1.51 on 01/27/2017
; - v1.50 on 11/20/2016
;
; - v1.47 on 02/20/2014 [?]
;
; modified by Rseding91 using fincs 64 bit compatible
; - v1.45 on 05/01/2013 
;
; by tic (Tariq Porter)
; - v1.45 on 07/09/2011 
; - v1.01 on 31/05/2008
;
; Detailed history:
; - 05/06/2020 = Synchronized with mmikeww's repository and fixed a few bugs
; - 24/05/2020 = Added a few more functions and fixed or improved already exiting functions
; - 11/02/2020 = Imported updated MDMF functions from mmikeww, and AHK v2 examples, and other minor changes
; - 25/02/2020 = Added several new functions, including for color conversions [from Tidbit], improved/fixed several functions
; - 11/01/2019 = Implemented support for a private font file for Gdip_AddPathStringSimplified()
; - 10/28/2019 = Added 7 new GDI+ functions and fixes related to Gdip_CreateFontFamilyFromFile()
; - 10/27/2019 = Added 5 new GDI+ functions and bug fixes for Gdip_TestBitmapUniformity(), Gdip_RotateBitmapAtCenter() and Gdip_ResizeBitmap()
; - 10/06/2019 = Added more parameters to Gdip_GraphicsFromImage/HDC/HWND and added Gdip_GetPixelColor()
; - 09/27/2019 = bug fixes...
; - 09/23/2019 = Added 4 new functions and improved Gdip_CreateBitmap() [ Marius Șucan ]
; - 09/19/2019 = Added 4 new functions and improved Gdip_RotateBitmapAtCenter() [ Marius Șucan ]
; - 09/17/2019 = Added 6 new GDI+ functions and renamed curve related functions [ Marius Șucan ]
; - 09/16/2019 = Added 10 new GDI+ functions [ Marius Șucan ]
; - 09/15/2019 = Added 3 new GDI+ functions and improved Gdip_DrawStringAlongPolygon() [ Marius Șucan ]
; - 09/13/2019 = Added 10 new GDI+ functions [ Marius Șucan ]
; - 09/12/2019 = Added 6 new GDI+ functions [ Marius Șucan ]
; - 09/11/2019 = Added 10 new GDI+ functions [ Marius Șucan ]
; - 09/10/2019 = Added 17 new GDI+ functions [ Marius Șucan ]
; - 09/09/2019 = Added 14 new GDI+ functions [ Marius Șucan ]
; - 09/08/2019 = Added 3 new functions and fixed Gdip_SetPenDashArray() [ Marius Șucan ]
; - 09/07/2019 = Added 12 new functions [ Marius Șucan ]
; - 09/06/2019 = Added 14 new GDI+ functions [ Marius Șucan ]
; - 09/05/2019 = Added 27 new GDI+ functions [ Marius Șucan ]
; - 09/04/2019 = Added 36 new GDI+ functions [ Marius Șucan ]
; - 09/03/2019 = Added about 37 new GDI+ functions [ Marius Șucan ]
; - 08/29/2019 = Fixed Gdip_GetPropertyTagName() [on AHK v2], Gdip_GetPenColor() and Gdip_GetSolidFillColor(), added Gdip_LoadImageFromFile()
; - 08/23/2019 = Added Gdip_FillRoundedRectangle2() and Gdip_DrawRoundedRectangle2(); extracted from Gdip2 by Tariq [tic] and corrected functions names
; - 08/21/2019 = Added GenerateColorMatrix() by Marius Șucan
; - 08/19/2019 = Added 12 functions. Extracted from a class wrapper for GDI+ written by nnnik in 2017.
; - 08/18/2019 = Added Gdip_AddPathRectangle() and eight PathGradient related functions by JustMe
; - 08/16/2019 = Added Gdip_DrawImageFX(), Gdip_CreateEffect() and other related functions [ Marius Șucan ]
; - 08/15/2019 = Added Gdip_DrawRoundedLine() by DevX and Rabiator
; - 08/15/2019 = Added 11 GraphicsPath related functions by "Learning one" and updated by Marius Șucan
; - 08/14/2019 = Added Gdip_IsVisiblePathPoint() and RotateAtCenter() by RazorHalo
; - 08/08/2019 = Added Gdi_GetDIBits() and Gdi_CreateDIBitmap() by Marius Șucan
; - 07/19/2019 = Added Gdip_GetHistogram() by swagfag and GetProperty GDI+ functions by JustMe
; - 11/15/2017 = compatibility with both AHK v2 and v1, restored by nnnik
; - 06/19/2017 = Fixed few bugs from old syntax by Bartlomiej Uliasz
; - 06/11/2017 = made code compatible with new AHK v2.0-a079-be5df98 by Bartlomiej Uliasz
; - 01/27/2017 = fixed some bugs and made #Warn All compatible by Bartlomiej Uliasz
; - 11/20/2016 = fixed Gdip_BitmapFromBRA() by 'just me'
; - 11/18/2016 = backward compatible support for both AHK v1.1 and AHK v2
; - 11/15/2016 = initial AHK v2 support by guest3456
; - 02/20/2014 = fixed Gdip_CreateRegion() and Gdip_GetClipRegion() on AHK Unicode x86
; - 05/13/2013 = fixed Gdip_SetBitmapToClipboard() on AHK Unicode x64
; - 07/09/2011 = v1.45 release by tic (Tariq Porter)
; - 31/05/2008 = v1.01 release by tic (Tariq Porter)
;
;#####################################################################################
; STATUS ENUMERATION
; Return values for functions specified to have status enumerated return type
;#####################################################################################
;
; Ok =                  = 0
; GenericError          = 1
; InvalidParameter      = 2
; OutOfMemory           = 3
; ObjectBusy            = 4
; InsufficientBuffer    = 5
; NotImplemented        = 6
; Win32Error            = 7
; WrongState            = 8
; Aborted               = 9
; FileNotFound          = 10
; ValueOverflow         = 11
; AccessDenied          = 12
; UnknownImageFormat    = 13
; FontFamilyNotFound    = 14
; FontStyleNotFound     = 15
; NotTrueTypeFont       = 16
; UnsupportedGdiplusVersion= 17
; GdiplusNotInitialized    = 18
; PropertyNotFound         = 19
; PropertyNotSupported     = 20
; ProfileNotFound          = 21
;
;#####################################################################################
; FUNCTIONS LIST
; See functions-list.txt file.
;#####################################################################################

; Function:             UpdateLayeredWindow
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

UpdateLayeredWindow(hwnd, hdc, x:="", y:="", w:="", h:="", Alpha:=255) {
   Ptr := "UPtr"
   if ((x != "") && (y != ""))
      VarSetCapacity(pt, 8), NumPut(x, pt, 0, "UInt"), NumPut(y, pt, 4, "UInt")

   if (w = "") || (h = "")
      GetWindowRect(hwnd, W, H)

   return DllCall("UpdateLayeredWindow"
               , Ptr, hwnd
               , Ptr, 0
               , Ptr, ((x = "") && (y = "")) ? 0 : &pt
               , "int64*", w|h<<32
               , Ptr, hdc
               , "int64*", 0
               , "uint", 0
               , "UInt*", Alpha<<16|1<<24
               , "uint", 2)
}

;#####################################################################################

; Function        BitBlt
; Description     The BitBlt function performs a bit-block transfer of the color data corresponding to a rectangle
;                 of pixels from the specified source device context into a destination device context.
;
; dDC             handle to destination DC
; dX, dY          x, y coordinates of the destination upper-left corner
; dW, dH          width and height of the area to copy
; sDC             handle to source DC
; sX, sY          x, y coordinates of the source upper-left corner
; Raster          raster operation code
;
; return          If the function succeeds, the return value is nonzero
;
; notes           If no raster operation is specified, then SRCCOPY is used, which copies the source directly to the destination rectangle
;
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

BitBlt(ddc, dx, dy, dw, dh, sdc, sx, sy, raster:="") {
; This function works only with GDI hBitmaps that 
; are Device-Dependent Bitmaps [DDB].

   Ptr := "UPtr"
   return DllCall("gdi32\BitBlt"
               , Ptr, dDC
               , "int", dX, "int", dY
               , "int", dW, "int", dH
               , Ptr, sDC
               , "int", sX, "int", sY
               , "uint", Raster ? Raster : 0x00CC0020)
}

;#####################################################################################

; Function        StretchBlt
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

StretchBlt(ddc, dx, dy, dw, dh, sdc, sx, sy, sw, sh, Raster:="") {
   Ptr := "UPtr"

   return DllCall("gdi32\StretchBlt"
               , Ptr, ddc
               , "int", dX, "int", dY
               , "int", dW, "int", dH
               , Ptr, sdc
               , "int", sX, "int", sY
               , "int", sW, "int", sH
               , "uint", Raster ? Raster : 0x00CC0020)
}

;#####################################################################################

; Function           SetStretchBltMode
; Description        The SetStretchBltMode function sets the bitmap stretching mode in the specified device context
;
; hdc                handle to the DC
; iStretchMode       The stretching mode, describing how the target will be stretched
;
; return             If the function succeeds, the return value is the previous stretching mode. If it fails it will return 0
;

SetStretchBltMode(hdc, iStretchMode:=4) {
; iStretchMode options:
; BLACKONWHITE = 1
; COLORONCOLOR = 3
; HALFTONE = 4
; WHITEONBLACK = 2
; STRETCH_ANDSCANS = BLACKONWHITE
; STRETCH_DELETESCANS = COLORONCOLOR
; STRETCH_HALFTONE = HALFTONE
; STRETCH_ORSCANS = WHITEONBLACK

   return DllCall("gdi32\SetStretchBltMode"
               , "UPtr", hdc
               , "int", iStretchMode)
}

;#####################################################################################

; Function           SetImage
; Description        Associates a new image with a static control
;
; hwnd               handle of the control to update
; hBitmap            a gdi bitmap to associate the static control with
;
; return             If the function succeeds, the return value is nonzero

SetImage(hwnd, hBitmap) {
; STM_SETIMAGE = 0x172
; Example: Gui, Add, Text, 0xE w500 h300 hwndhPic          ; SS_Bitmap    = 0xE

   Ptr := "UPtr"
   E := DllCall("SendMessage", Ptr, hwnd, "UInt", 0x172, "UInt", 0x0, Ptr, hBitmap )
   DeleteObject(E)
   return E
}

;#####################################################################################

; Function           SetSysColorToControl
; Description        Sets a solid colour to a control
;
; hwnd               handle of the control to update
; SysColor           A system colour to set to the control
;
; return             If the function succeeds, the return value is zero
;
; notes              A control must have the 0xE style set to it so it is recognised as a bitmap
;                    By default SysColor=15 is used which is COLOR_3DFACE. This is the standard background for a control

SetSysColorToControl(hwnd, SysColor:=15) {
; SysColor options:
; 3DDKSHADOW = 21
; 3DFACE = 15
; 3DHIGHLIGHT = 20
; 3DHILIGHT = 20
; 3DLIGHT = 22
; 3DSHADOW = 16
; ACTIVEBORDER = 10
; ACTIVECAPTION = 2
; APPWORKSPACE = 12
; BACKGROUND = 1
; BTNFACE = 15
; BTNHIGHLIGHT = 20
; BTNHILIGHT = 20
; BTNSHADOW = 16
; BTNTEXT = 18
; CAPTIONTEXT = 9
; DESKTOP = 1
; GRADIENTACTIVECAPTION  27
; GRADIENTINACTIVECAPTION = 28
; GRAYTEXT = 17
; HIGHLIGHT = 13
; HIGHLIGHTTEXT = 14
; HOTLIGHT = 26
; INACTIVEBORDER = 11
; INACTIVECAPTION = 3
; INACTIVECAPTIONTEXT = 19
; INFOBK = 24
; INFOTEXT = 23
; MENU = 4
; MENUHILIGHT = 29
; MENUBAR = 30
; MENUTEXT = 7
; SCROLLBAR = 0
; WINDOW = 5
; WINDOWFRAME = 6
; WINDOWTEXT = 8
   Ptr := "UPtr"
   GetWindowRect(hwnd, W, H)
   bc := DllCall("GetSysColor", "Int", SysColor, "UInt")
   pBrushClear := Gdip_BrushCreateSolid(0xff000000 | (bc >> 16 | bc & 0xff00 | (bc & 0xff) << 16))
   pBitmap := Gdip_CreateBitmap(w, h)
   G := Gdip_GraphicsFromImage(pBitmap)
   Gdip_FillRectangle(G, pBrushClear, 0, 0, w, h)
   hBitmap := Gdip_CreateHBITMAPFromBitmap(pBitmap)
   SetImage(hwnd, hBitmap)
   Gdip_DeleteBrush(pBrushClear)
   Gdip_DeleteGraphics(G)
   Gdip_DisposeImage(pBitmap)
   DeleteObject(hBitmap)
   return 0
}

;#####################################################################################

; Function        Gdip_BitmapFromScreen
; Description     Gets a gdi+ bitmap from the screen
;
; Screen          0 = All screens
;                 Any numerical value = Just that screen
;                 x|y|w|h = Take specific coordinates with a width and height
; Raster          raster operation code
;
; return          If the function succeeds, the return value is a pointer to a gdi+ bitmap
;                 -1: one or more of x,y,w,h parameters were not passed properly
;
; notes           If no raster operation is specified, then SRCCOPY is used to the returned bitmap

Gdip_BitmapFromScreen(Screen:=0, Raster:="") {
   hhdc := 0
   Ptr := "UPtr"
   if (Screen = 0)
   {
      _x := DllCall("GetSystemMetrics", "Int", 76 )
      _y := DllCall("GetSystemMetrics", "Int", 77 )
      _w := DllCall("GetSystemMetrics", "Int", 78 )
      _h := DllCall("GetSystemMetrics", "Int", 79 )
   } else if (SubStr(Screen, 1, 5) = "hwnd:")
   {
      hwnd := SubStr(Screen, 6)
      if !WinExist("ahk_id " hwnd)
         return -2

      GetWindowRect(hwnd, _w, _h)
      _x := _y := 0
      hhdc := GetDCEx(hwnd, 3)
   } else if IsInteger(Screen)
   {
      M := GetMonitorInfo(Screen)
      _x := M.Left, _y := M.Top, _w := M.Right-M.Left, _h := M.Bottom-M.Top
   } else
   {
      S := StrSplit(Screen, "|")
      _x := S[1], _y := S[2], _w := S[3], _h := S[4]
   }

   if (_x = "") || (_y = "") || (_w = "") || (_h = "")
      return -1

   chdc := CreateCompatibleDC(), hbm := CreateDIBSection(_w, _h, chdc)
   obm := SelectObject(chdc, hbm), hhdc := hhdc ? hhdc : GetDC()
   BitBlt(chdc, 0, 0, _w, _h, hhdc, _x, _y, Raster)
   ReleaseDC(hhdc)

   pBitmap := Gdip_CreateBitmapFromHBITMAP(hbm)
   SelectObject(chdc, obm), DeleteObject(hbm), DeleteDC(hhdc), DeleteDC(chdc)
   return pBitmap
}

;#####################################################################################

; Function           Gdip_BitmapFromHWND
; Description        Uses PrintWindow to get a handle to the specified window and return a bitmap from it
;
; hwnd               handle to the window to get a bitmap from
; clientOnly         capture only the client area of the window, without title bar and border
;
; return             If the function succeeds, the return value is a pointer to a gdi+ bitmap

Gdip_BitmapFromHWND(hwnd, clientOnly:=0) {
   ; Restore the window if minimized! Must be visible for capture.
   if DllCall("IsIconic", "ptr", hwnd)
      DllCall("ShowWindow", "ptr", hwnd, "int", 4)

   Ptr := "UPtr"
   thisFlag := 0
   If (clientOnly=1)
   {
      VarSetCapacity(rc, 16, 0)
      DllCall("GetClientRect", "ptr", hwnd, "ptr", &rc)
      Width := NumGet(rc, 8, "int")
      Height := NumGet(rc, 12, "int")
      thisFlag := 1
   } Else GetWindowRect(hwnd, Width, Height)

   hbm := CreateDIBSection(Width, Height)
   hdc := CreateCompatibleDC(), obm := SelectObject(hdc, hbm)
   PrintWindow(hwnd, hdc, 2 + thisFlag)
   pBitmap := Gdip_CreateBitmapFromHBITMAP(hbm)
   SelectObject(hdc, obm), DeleteObject(hbm), DeleteDC(hdc)
   return pBitmap
}

;#####################################################################################

; Function           CreateRectF
; Description        Creates a RectF object, containing a the coordinates and dimensions of a rectangle
;
; RectF              Name to call the RectF object
; x, y               x, y coordinates of the upper left corner of the rectangle
; w, h               Width and height of the rectangle
;
; return             No return value

CreateRectF(ByRef RectF, x, y, w, h) {
   VarSetCapacity(RectF, 16)
   NumPut(x, RectF, 0, "float"), NumPut(y, RectF, 4, "float")
   NumPut(w, RectF, 8, "float"), NumPut(h, RectF, 12, "float")
}

;#####################################################################################

; Function           CreateRect
; Description        Creates a Rect object, containing a the coordinates and dimensions of a rectangle
;
; Rect               Name to call the Rect object
; x, y               x, y coordinates of the upper left corner of the rectangle
; x2, y2             x, y coordinates of the bottom right corner of the rectangle

; return             No return value

CreateRect(ByRef Rect, x, y, x2, y2) {
; modified by Marius Șucan according to dangerdogL2121
; found on https://autohotkey.com/board/topic/29449-gdi-standard-library-145-by-tic/page-93

   VarSetCapacity(Rect, 16)
   NumPut(x, Rect, 0, "uint"), NumPut(y, Rect, 4, "uint")
   NumPut(x2, Rect, 8, "uint"), NumPut(y2, Rect, 12, "uint")
}
;#####################################################################################

; Function           CreateSizeF
; Description        Creates a SizeF object, containing an 2 values
;
; SizeF              Name to call the SizeF object
; w, h               width and height values for the SizeF object
;
; return             No Return value

CreateSizeF(ByRef SizeF, w, h) {
   VarSetCapacity(SizeF, 8)
   NumPut(w, SizeF, 0, "float")
   NumPut(h, SizeF, 4, "float")
}

;#####################################################################################

; Function           CreatePointF
; Description        Creates a SizeF object, containing two values
;
; SizeF              Name to call the SizeF object
; x, y               x, y values for the SizeF object
;
; return             No Return value

CreatePointF(ByRef PointF, x, y) {
   VarSetCapacity(PointF, 8)
   NumPut(x, PointF, 0, "float")
   NumPut(y, PointF, 4, "float")
}

CreatePointsF(ByRef PointsF, inPoints) {
   Points := StrSplit(inPoints, "|")
   PointsCount := Points.Length
   VarSetCapacity(PointsF, 8 * PointsCount, 0)
   for eachPoint, Point in Points
   {
       Coord := StrSplit(Point, ",")
       NumPut(Coord[1], &PointsF, 8*(A_Index-1), "float")
       NumPut(Coord[2], &PointsF, (8*(A_Index-1))+4, "float")
   }
   Return PointsCount
}

;#####################################################################################

; Function           CreateDIBSection
; Description        The CreateDIBSection function creates a DIB (Device Independent Bitmap) that applications can write to directly
;
; w, h               width and height of the bitmap to create
; hdc                a handle to the device context to use the palette from
; bpp                bits per pixel (32 = ARGB)
; ppvBits            A pointer to a variable that receives a pointer to the location of the DIB bit values
;
; return             returns a DIB. A gdi bitmap
;
; notes              ppvBits will receive the location of the pixels in the DIB

CreateDIBSection(w, h, hdc:="", bpp:=32, ByRef ppvBits:=0, Usage:=0, hSection:=0, Offset:=0) {
; A GDI function that creates a new hBitmap,
; a device-independent bitmap [DIB].
; A DIB consists of two distinct parts:
; a BITMAPINFO structure describing the dimensions
; and colors of the bitmap, and an array of bytes
; defining the pixels of the bitmap. 

   Ptr := "UPtr"
   hdc2 := hdc ? hdc : GetDC()
   VarSetCapacity(bi, 40, 0)
   NumPut(40, bi, 0, "uint")
   NumPut(w, bi, 4, "uint")
   NumPut(h, bi, 8, "uint")
   NumPut(1, bi, 12, "ushort")
   NumPut(bpp, bi, 14, "ushort")
   NumPut(0, bi, 16, "uInt")

   hbm := DllCall("CreateDIBSection"
               , Ptr, hdc2
               , Ptr, &bi    ; BITMAPINFO
               , "uint", Usage
               , "UPtr*", ppvBits
               , Ptr, hSection
               , "uint", OffSet, Ptr)

   if !hdc
      ReleaseDC(hdc2)
   return hbm
}

;#####################################################################################

; Function           PrintWindow
; Description        The PrintWindow function copies a visual window into the specified device context (DC), typically a printer DC
;
; hwnd               A handle to the window that will be copied
; hdc                A handle to the device context
; Flags              Drawing options
;
; return             If the function succeeds, it returns a nonzero value
;
; PW_CLIENTONLY      = 1

PrintWindow(hwnd, hdc, Flags:=2) {
; set Flags to 2, to capture hardware accelerated windows
; this only applies on Windows 8.1 and later versions.

   Ptr := "UPtr"
   return DllCall("PrintWindow", Ptr, hwnd, Ptr, hdc, "uint", Flags)
}

;#####################################################################################

; Function           DestroyIcon
; Description        Destroys an icon and frees any memory the icon occupied
;
; hIcon              Handle to the icon to be destroyed. The icon must not be in use
;
; return             If the function succeeds, the return value is nonzero

DestroyIcon(hIcon) {
   return DllCall("DestroyIcon", "UPtr", hIcon)
}

;#####################################################################################

; Function:          GetIconDimensions
; Description:       Retrieves a given icon/cursor's width and height 
;
; hIcon              Pointer to an icon or cursor
; Width, Height      ByRef variables. These variables are set to the icon's width and height
;
; return             If the function succeeds, the return value is zero, otherwise:
;                    -1 = Could not retrieve the icon's info. Check A_LastError for extended information
;                    -2 = Could not delete the icon's bitmask bitmap
;                    -3 = Could not delete the icon's color bitmap

GetIconDimensions(hIcon, ByRef Width, ByRef Height) {
   Ptr := "UPtr"
   Width := Height := 0

   VarSetCapacity(ICONINFO, size := 16 + 2 * A_PtrSize, 0)
   if !DllCall("user32\GetIconInfo", Ptr, hIcon, Ptr, &ICONINFO)
      return -1
   
   hbmMask := NumGet(&ICONINFO, 16, Ptr)
   hbmColor := NumGet(&ICONINFO, 16 + A_PtrSize, Ptr)
   VarSetCapacity(BITMAP, size, 0)

   if DllCall("gdi32\GetObject", Ptr, hbmColor, "Int", size, Ptr, &BITMAP)
   {
      Width := NumGet(&BITMAP, 4, "Int")
      Height := NumGet(&BITMAP, 8, "Int")
   }

   if !DeleteObject(hbmMask)
      return -2
   
   if !DeleteObject(hbmColor)
      return -3

   return 0
}

PaintDesktop(hdc) {
   return DllCall("PaintDesktop", "UPtr", hdc)
}

;#####################################################################################

; Function        CreateCompatibleDC
; Description     This function creates a memory device context (DC) compatible with the specified device
;
; hdc             Handle to an existing device context
;
; return          returns the handle to a device context or 0 on failure
;
; notes           If this handle is 0 (by default), the function creates a memory device context compatible with the application's current screen

CreateCompatibleDC(hdc:=0) {
   return DllCall("CreateCompatibleDC", "UPtr", hdc)
}

;#####################################################################################

; Function        SelectObject
; Description     The SelectObject function selects an object into the specified device context (DC). The new object replaces the previous object of the same type
;
; hdc             Handle to a DC
; hgdiobj         A handle to the object to be selected into the DC
;
; return          If the selected object is not a region and the function succeeds, the return value is a handle to the object being replaced
;
; notes           The specified object must have been created by using one of the following functions
;                 Bitmap - CreateBitmap, CreateBitmapIndirect, CreateCompatibleBitmap, CreateDIBitmap, CreateDIBSection (A single bitmap cannot be selected into more than one DC at the same time)
;                 Brush - CreateBrushIndirect, CreateDIBPatternBrush, CreateDIBPatternBrushPt, CreateHatchBrush, CreatePatternBrush, CreateSolidBrush
;                 Font - CreateFont, CreateFontIndirect
;                 Pen - CreatePen, CreatePenIndirect
;                 Region - CombineRgn, CreateEllipticRgn, CreateEllipticRgnIndirect, CreatePolygonRgn, CreateRectRgn, CreateRectRgnIndirect
;
; notes           If the selected object is a region and the function succeeds, the return value is one of the following value
;
; SIMPLEREGION    = 2 Region consists of a single rectangle
; COMPLEXREGION   = 3 Region consists of more than one rectangle
; NULLREGION      = 1 Region is empty

SelectObject(hdc, hgdiobj) {
   Ptr := "UPtr"
   return DllCall("SelectObject", Ptr, hdc, Ptr, hgdiobj)
}

;#####################################################################################

; Function           DeleteObject
; Description        This function deletes a logical pen, brush, font, bitmap, region, or palette, freeing all system resources associated with the object
;                    After the object is deleted, the specified handle is no longer valid
;
; hObject            Handle to a logical pen, brush, font, bitmap, region, or palette to delete
;
; return             Nonzero indicates success. Zero indicates that the specified handle is not valid or that the handle is currently selected into a device context

DeleteObject(hObject) {
   return DllCall("DeleteObject", "UPtr", hObject)
}

;#####################################################################################

; Function           GetDC
; Description        This function retrieves a handle to a display device context (DC) for the client area of the specified window.
;                    The display device context can be used in subsequent graphics display interface (GDI) functions to draw in the client area of the window.
;
; hwnd               Handle to the window whose device context is to be retrieved. If this value is NULL, GetDC retrieves the device context for the entire screen
;
; return             The handle the device context for the specified window's client area indicates success. NULL indicates failure

GetDC(hwnd:=0) {
   return DllCall("GetDC", "UPtr", hwnd)
}

GetDCEx(hwnd, flags:=0, hrgnClip:=0) {
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

   Ptr := "UPtr"
   return DllCall("GetDCEx", Ptr, hwnd, Ptr, hrgnClip, "int", flags)
}

;#####################################################################################

; Function        ReleaseDC
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

ReleaseDC(hdc, hwnd:=0) {
   Ptr := "UPtr"
   return DllCall("ReleaseDC", Ptr, hwnd, Ptr, hdc)
}

;#####################################################################################

; Function           DeleteDC
; Description        The DeleteDC function deletes the specified device context (DC)
;
; hdc                A handle to the device context
;
; return             If the function succeeds, the return value is nonzero
;
; notes              An application must not delete a DC whose handle was obtained by calling the GetDC function. Instead, it must call the ReleaseDC function to free the DC

DeleteDC(hdc) {
   return DllCall("DeleteDC", "UPtr", hdc)
}

;#####################################################################################

; Function           Gdip_LibraryVersion
; Description        Get the current library version
;
; return             the library version
;
; notes              This is useful for non compiled programs to ensure that a person doesn't run an old version when testing your scripts

Gdip_LibraryVersion() {
   return 1.45
}

;#####################################################################################

; Function        Gdip_LibrarySubVersion
; Description     Get the current library sub version
;
; return          the library sub version
;
; notes           This is the sub-version currently maintained by Rseding91
;                 Updated by guest3456 preliminary AHK v2 support
;                 Updated by Marius Șucan reflecting the work on Gdip_all extended compilation

Gdip_LibrarySubVersion() {
   return 1.84
}

;#####################################################################################

; Function:          Gdip_BitmapFromBRA
; Description:       Gets a pointer to a gdi+ bitmap from a BRA file
;
; BRAFromMemIn       The variable for a BRA file read to memory
; File               The name of the file, or its number that you would like (This depends on alternate parameter)
; Alternate          Changes whether the File parameter is the file name or its number
;
; return             If the function succeeds, the return value is a pointer to a gdi+ bitmap
;                    -1 = The BRA variable is empty
;                    -2 = The BRA has an incorrect header
;                    -3 = The BRA has information missing
;                    -4 = Could not find file inside the BRA

Gdip_BitmapFromBRA(ByRef BRAFromMemIn, File, Alternate := 0) {
   pBitmap := 0
   pStream := 0

   If !(BRAFromMemIn)
      Return -1
   Headers := StrSplit(StrGet(&BRAFromMemIn, 256, "CP0"), "`n")
   Header := StrSplit(Headers.1, "|")
   If (Header.Length != 4) || (Header.2 != "BRA!")
      Return -2
   _Info := StrSplit(Headers.2, "|")
   If (_Info.Length != 3)
      Return -3
   OffsetTOC := StrPut(Headers.1, "CP0") + StrPut(Headers.2, "CP0") ;  + 2
   OffsetData := _Info.2
   SearchIndex := Alternate ? 1 : 2
   TOC := StrGet(&BRAFromMemIn + OffsetTOC, OffsetData - OffsetTOC - 1, "CP0")
   RX1 := A_AhkVersion < "2" ? "mi`nO)^" : "mi`n)^"
   Offset := Size := 0
   If RegExMatch(TOC, RX1 . (Alternate ? File "\|.+?" : "\d+\|" . File) . "\|(\d+)\|(\d+)$", FileInfo) {
      Offset := OffsetData + FileInfo.1
      Size := FileInfo.2
   }
   If (Size=0)
      Return -4
   hData := DllCall("GlobalAlloc", "UInt", 2, "UInt", Size, "UPtr")
   pData := DllCall("GlobalLock", "Ptr", hData, "UPtr")
   DllCall("RtlMoveMemory", "Ptr", pData, "Ptr", &BRAFromMemIn + Offset, "Ptr", Size)
   DllCall("GlobalUnlock", "Ptr", hData)
   DllCall("Ole32.dll\CreateStreamOnHGlobal", "Ptr", hData, "Int", 1, "PtrP", pStream)
   DllCall("gdiplus\GdipCreateBitmapFromStream", "Ptr", pStream, "PtrP", pBitmap)
   ObjRelease(pStream)
   Return pBitmap
}

;#####################################################################################

; Function:        Gdip_BitmapFromBase64
; Description:     Creates a bitmap from a Base64 encoded string
;
; Base64           ByRef variable. Base64 encoded string. Immutable, ByRef to avoid performance overhead of passing long strings.
;
; return           If the function succeeds, the return value is a pointer to a bitmap, otherwise:
;                 -1 = Could not calculate the length of the required buffer
;                 -2 = Could not decode the Base64 encoded string
;                 -3 = Could not create a memory stream

Gdip_BitmapFromBase64(ByRef Base64) {
   Ptr := "UPtr"
   pBitmap := 0
   DecLen := 0

   ; calculate the length of the buffer needed
   if !(DllCall("crypt32\CryptStringToBinary", Ptr, &Base64, "UInt", 0, "UInt", 0x01, Ptr, 0, "UIntP", DecLen, Ptr, 0, Ptr, 0))
      return -1

   VarSetCapacity(Dec, DecLen, 0)

   ; decode the Base64 encoded string
   if !(DllCall("crypt32\CryptStringToBinary", Ptr, &Base64, "UInt", 0, "UInt", 0x01, Ptr, &Dec, "UIntP", DecLen, Ptr, 0, Ptr, 0))
      return -2

   ; create a memory stream
   if !(pStream := DllCall("shlwapi\SHCreateMemStream", Ptr, &Dec, "UInt", DecLen, "UPtr"))
      return -3

   DllCall("gdiplus\GdipCreateBitmapFromStreamICM", Ptr, pStream, "PtrP", pBitmap)
   ObjRelease(pStream)

   return pBitmap
}

;#####################################################################################

; Function           Gdip_DrawRectangle
; Description        This function uses a pen to draw the outline of a rectangle into the Graphics of a bitmap
;
; pGraphics          Pointer to the Graphics of a bitmap
; pPen               Pointer to a pen
; x, y               x, y coordinates of the top left of the rectangle
; w, h               width and height of the rectangle
;
; return             status enumeration. 0 = success
;
; notes              as all coordinates are taken from the top left of each pixel, then the entire width/height should be specified as subtracting the pen width

Gdip_DrawRectangle(pGraphics, pPen, x, y, w, h) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipDrawRectangle", Ptr, pGraphics, Ptr, pPen, "float", x, "float", y, "float", w, "float", h)
}

;#####################################################################################

; Function           Gdip_DrawRoundedRectangle
; Description        This function uses a pen to draw the outline of a rounded rectangle into the Graphics of a bitmap
;
; pGraphics          Pointer to the Graphics of a bitmap
; pPen               Pointer to a pen
; x, y               x, y coordinates of the top left of the rounded rectangle
; w, h               width and height of the rectanlge
; r                  radius of the rounded corners
;
; return             status enumeration. 0 = success
;
; notes              as all coordinates are taken from the top left of each pixel, then the entire width/height should be specified as subtracting the pen width

Gdip_DrawRoundedRectangle(pGraphics, pPen, x, y, w, h, r) {
   Gdip_SetClipRect(pGraphics, x-r, y-r, 2*r, 2*r, 4)
   Gdip_SetClipRect(pGraphics, x+w-r, y-r, 2*r, 2*r, 4)
   Gdip_SetClipRect(pGraphics, x-r, y+h-r, 2*r, 2*r, 4)
   Gdip_SetClipRect(pGraphics, x+w-r, y+h-r, 2*r, 2*r, 4)
   _E := Gdip_DrawRectangle(pGraphics, pPen, x, y, w, h)
   Gdip_ResetClip(pGraphics)
   Gdip_SetClipRect(pGraphics, x-(2*r), y+r, w+(4*r), h-(2*r), 4)
   Gdip_SetClipRect(pGraphics, x+r, y-(2*r), w-(2*r), h+(4*r), 4)
   Gdip_DrawEllipse(pGraphics, pPen, x, y, 2*r, 2*r)
   Gdip_DrawEllipse(pGraphics, pPen, x+w-(2*r), y, 2*r, 2*r)
   Gdip_DrawEllipse(pGraphics, pPen, x, y+h-(2*r), 2*r, 2*r)
   Gdip_DrawEllipse(pGraphics, pPen, x+w-(2*r), y+h-(2*r), 2*r, 2*r)
   Gdip_ResetClip(pGraphics)
   return _E
}

Gdip_DrawRoundedRectangle2(pGraphics, pPen, x, y, w, h, r, Angle:=0) {
; extracted from: https://github.com/tariqporter/Gdip2/blob/master/lib/Object.ahk
; and adapted by Marius Șucan

   penWidth := Gdip_GetPenWidth(pPen)
   pw := penWidth / 2
   if (w <= h && (r + pw > w / 2))
   {
      r := (w / 2 > pw) ? w / 2 - pw : 0
   } else if (h < w && r + pw > h / 2)
   {
      r := (h / 2 > pw) ? h / 2 - pw : 0
   } else if (r < pw / 2)
   {
      r := pw / 2
   }

   r2 := r * 2
   path1 := Gdip_CreatePath(0)
   Gdip_AddPathArc(path1, x + pw, y + pw, r2, r2, 180, 90)
   Gdip_AddPathLine(path1, x + pw + r, y + pw, x + w - r - pw, y + pw)
   Gdip_AddPathArc(path1, x + w - r2 - pw, y + pw, r2, r2, 270, 90)
   Gdip_AddPathLine(path1, x + w - pw, y + r + pw, x + w - pw, y + h - r - pw)
   Gdip_AddPathArc(path1, x + w - r2 - pw, y + h - r2 - pw, r2, r2, 0, 90)
   Gdip_AddPathLine(path1, x + w - r - pw, y + h - pw, x + r + pw, y + h - pw)
   Gdip_AddPathArc(path1, x + pw, y + h - r2 - pw, r2, r2, 90, 90)
   Gdip_AddPathLine(path1, x + pw, y + h - r - pw, x + pw, y + r + pw)
   Gdip_ClosePathFigure(path1)
   If (Angle>0)
      Gdip_RotatePathAtCenter(path1, Angle)
   _E := Gdip_DrawPath(pGraphics, pPen, path1)
   Gdip_DeletePath(path1)
   return _E
}

;#####################################################################################

; Function           Gdip_DrawEllipse
; Description        This function uses a pen to draw the outline of an ellipse into the Graphics of a bitmap
;
; pGraphics          Pointer to the Graphics of a bitmap
; pPen               Pointer to a pen
; x, y               x, y coordinates of the top left of the rectangle the ellipse will be drawn into
; w, h               width and height of the ellipse
;
; return             status enumeration. 0 = success
;
; notes              as all coordinates are taken from the top left of each pixel, then the entire width/height should be specified as subtracting the pen width

Gdip_DrawEllipse(pGraphics, pPen, x, y, w, h) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipDrawEllipse", Ptr, pGraphics, Ptr, pPen, "float", x, "float", y, "float", w, "float", h)
}

;#####################################################################################

; Function        Gdip_DrawBezier
; Description     This function uses a pen to draw the outline of a bezier (a weighted curve) into the Graphics of a bitmap
; A Bezier spline does not pass through its control points. The control points act as magnets, pulling the curve
; in certain directions to influence the way the spline bends.

; pGraphics       Pointer to the Graphics of a bitmap
; pPen            Pointer to a pen
; x1, y1          x, y coordinates of the start of the bezier
; x2, y2          x, y coordinates of the first arc of the bezier
; x3, y3          x, y coordinates of the second arc of the bezier
; x4, y4          x, y coordinates of the end of the bezier
;
; return          status enumeration. 0 = success
;
; notes           as all coordinates are taken from the top left of each pixel, then the entire width/height should be specified as subtracting the pen width

Gdip_DrawBezier(pGraphics, pPen, x1, y1, x2, y2, x3, y3, x4, y4) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipDrawBezier"
               , Ptr, pGraphics
               , Ptr, pPen
               , "float", x1
               , "float", y1
               , "float", x2
               , "float", y2
               , "float", x3
               , "float", y3
               , "float", x4
               , "float", y4)
}

;#####################################################################################

; Function           Gdip_DrawBezierCurve
; Description        This function uses a pen to draw beziers
; Parameters:
; pGraphics          Pointer to the Graphics of a bitmap
; pPen               Pointer to a pen
; Points
;   An array of starting and control points of a Bezier line
;   A single Bezier line consists of 4 points a starting point 2 control
;   points and an end point.
;   The line never actually goes through the control points.
;   The control points define the tangent in the starting and end points and their
;   distance controls how strongly the curve follows there.
;
; Return: status enumeration. 0 = success
;
; This function was extracted and modified by Marius Șucan from
; a class based wrapper around the GDI+ API made by nnnik.
; Source: https://github.com/nnnik/classGDIp
;
; Points array format:
; Points := "x1,y1|x2,y2|x3,y3|x4,y4" [... and so on]

Gdip_DrawBezierCurve(pGraphics, pPen, Points) {
   iCount := CreatePointsF(PointsF, Points)
   return DllCall("gdiplus\GdipDrawBeziers", "UPtr", pGraphics, "UPtr", pPen, "UPtr", &PointsF, "UInt", iCount)
}

Gdip_DrawClosedCurve(pGraphics, pPen, Points, Tension:="") {
; Draws a closed cardinal spline on a pGraphics object using a pPen object.
; A cardinal spline is a curve that passes through each point in the array.

; Tension: Non-negative real number that controls the length of the curve and how the curve bends. A value of
; zero specifies that the spline is a sequence of straight lines. As the value increases, the curve becomes fuller.
; Number that specifies how tightly the curve bends through the coordinates of the closed cardinal spline.

; Example points array:
; Points := "x1,y1|x2,y2|x3,y3" [and so on]
; At least three points must be defined.

   iCount := CreatePointsF(PointsF, Points)
   If Tension
      return DllCall("gdiplus\GdipDrawClosedCurve2", "UPtr", pGraphics, "UPtr", pPen, "UPtr", &PointsF, "UInt", iCount, "float", Tension)
   Else
      return DllCall("gdiplus\GdipDrawClosedCurve", "UPtr", pGraphics, "UPtr", pPen, "UPtr", &PointsF, "UInt", iCount)
}

Gdip_DrawCurve(pGraphics, pPen, Points, Tension:="") {
; Draws an open spline on a pGraphics object using a pPen object.
; A cardinal spline is a curve that passes through each point in the array.

; Tension: Non-negative real number that controls the length of the curve and how the curve bends. A value of
; zero specifies that the spline is a sequence of straight lines. As the value increases, the curve becomes fuller.
; Number that specifies how tightly the curve bends through the coordinates of the closed cardinal spline.

; Example points array:
; Points := "x1,y1|x2,y2|x3,y3" [and so on]
; At least three points must be defined.

   iCount := CreatePointsF(PointsF, Points)
   If Tension
      return DllCall("gdiplus\GdipDrawCurve2", "UPtr", pGraphics, "UPtr", pPen, "UPtr", &PointsF, "UInt", iCount, "float", Tension)
   Else
      return DllCall("gdiplus\GdipDrawCurve", "UPtr", pGraphics, "UPtr", pPen, "UPtr", &PointsF, "UInt", iCount)
}

Gdip_DrawPolygon(pGraphics, pPen, Points) {
; Draws a closed polygonal line on a pGraphics object using a pPen object.
;
; Example points array:
; Points := "x1,y1|x2,y2|x3,y3" [and so on]

   iCount := CreatePointsF(PointsF, Points)
   return DllCall("gdiplus\GdipDrawPolygon", "UPtr", pGraphics, "UPtr", pPen, "UPtr", &PointsF, "UInt", iCount)
}

;#####################################################################################

; Function           Gdip_DrawArc
; Description        This function uses a pen to draw the outline of an arc into the Graphics of a bitmap
;
; pGraphics          Pointer to the Graphics of a bitmap
; pPen               Pointer to a pen
; x, y               x, y coordinates of the start of the arc
; w, h               width and height of the arc
; StartAngle         specifies the angle between the x-axis and the starting point of the arc
; SweepAngle         specifies the angle between the starting and ending points of the arc
;
; return             status enumeration. 0 = success
;
; notes              as all coordinates are taken from the top left of each pixel, then the entire width/height should be specified as subtracting the pen width

Gdip_DrawArc(pGraphics, pPen, x, y, w, h, StartAngle, SweepAngle) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipDrawArc"
               , Ptr, pGraphics
               , Ptr, pPen
               , "float", x, "float", y
               , "float", w, "float", h
               , "float", StartAngle
               , "float", SweepAngle)
}

;#####################################################################################

; Function           Gdip_DrawPie
; Description        This function uses a pen to draw the outline of a pie into the Graphics of a bitmap
;
; pGraphics          Pointer to the Graphics of a bitmap
; pPen               Pointer to a pen
; x, y               x, y coordinates of the start of the pie
; w, h               width and height of the pie
; StartAngle         specifies the angle between the x-axis and the starting point of the pie
; SweepAngle         specifies the angle between the starting and ending points of the pie
;
; return             status enumeration. 0 = success
;
; notes              as all coordinates are taken from the top left of each pixel, then the entire width/height should be specified as subtracting the pen width

Gdip_DrawPie(pGraphics, pPen, x, y, w, h, StartAngle, SweepAngle) {
   Ptr := "UPtr"

   return DllCall("gdiplus\GdipDrawPie", Ptr, pGraphics, Ptr, pPen, "float", x, "float", y, "float", w, "float", h, "float", StartAngle, "float", SweepAngle)
}

;#####################################################################################

; Function        Gdip_DrawLine
; Description     This function uses a pen to draw a line into the Graphics of a bitmap
;
; pGraphics       Pointer to the Graphics of a bitmap
; pPen            Pointer to a pen
; x1, y1          x, y coordinates of the start of the line
; x2, y2          x, y coordinates of the end of the line
;
; return          status enumeration. 0 = success

Gdip_DrawLine(pGraphics, pPen, x1, y1, x2, y2) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipDrawLine"
               , Ptr, pGraphics
               , Ptr, pPen
               , "float", x1, "float", y1
               , "float", x2, "float", y2)
}

;#####################################################################################

; Function           Gdip_DrawLines
; Description        This function uses a pen to draw a series of joined lines into the Graphics of a bitmap
;
; pGraphics          Pointer to the Graphics of a bitmap
; pPen               Pointer to a pen
; Points             the coordinates of all the points passed as x1,y1|x2,y2|x3,y3.....
;
; return             status enumeration. 0 = success

Gdip_DrawLines(pGraphics, pPen, Points) {
   Ptr := "UPtr"
   iCount := CreatePointsF(PointsF, Points)
   return DllCall("gdiplus\GdipDrawLines", Ptr, pGraphics, Ptr, pPen, Ptr, &PointsF, "int", iCount)
}

;#####################################################################################

; Function           Gdip_FillRectangle
; Description        This function uses a brush to fill a rectangle in the Graphics of a bitmap
;
; pGraphics          Pointer to the Graphics of a bitmap
; pBrush             Pointer to a brush
; x, y               x, y coordinates of the top left of the rectangle
; w, h               width and height of the rectangle
;
; return             status enumeration. 0 = success

Gdip_FillRectangle(pGraphics, pBrush, x, y, w, h) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipFillRectangle"
               , Ptr, pGraphics
               , Ptr, pBrush
               , "float", x, "float", y
               , "float", w, "float", h)
}

;#####################################################################################

; Function           Gdip_FillRoundedRectangle
; Description        This function uses a brush to fill a rounded rectangle in the Graphics of a bitmap
;
; pGraphics          Pointer to the Graphics of a bitmap
; pBrush             Pointer to a brush
; x, y               x, y coordinates of the top left of the rounded rectangle
; w, h               width and height of the rectanlge
; r                  radius of the rounded corners
;
; return             status enumeration. 0 = success

Gdip_FillRoundedRectangle2(pGraphics, pBrush, x, y, w, h, r) {
; extracted from: https://github.com/tariqporter/Gdip2/blob/master/lib/Object.ahk
; and adapted by Marius Șucan

   r := (w <= h) ? (r < w // 2) ? r : w // 2 : (r < h // 2) ? r : h // 2
   path1 := Gdip_CreatePath(0)
   Gdip_AddPathRectangle(path1, x+r, y, w-(2*r), r)
   Gdip_AddPathRectangle(path1, x+r, y+h-r, w-(2*r), r)
   Gdip_AddPathRectangle(path1, x, y+r, r, h-(2*r))
   Gdip_AddPathRectangle(path1, x+w-r, y+r, r, h-(2*r))
   Gdip_AddPathRectangle(path1, x+r, y+r, w-(2*r), h-(2*r))
   Gdip_AddPathPie(path1, x, y, 2*r, 2*r, 180, 90)
   Gdip_AddPathPie(path1, x+w-(2*r), y, 2*r, 2*r, 270, 90)
   Gdip_AddPathPie(path1, x, y+h-(2*r), 2*r, 2*r, 90, 90)
   Gdip_AddPathPie(path1, x+w-(2*r), y+h-(2*r), 2*r, 2*r, 0, 90)
   E := Gdip_FillPath(pGraphics, pBrush, path1)
   Gdip_DeletePath(path1)
   return E
}

Gdip_FillRoundedRectangle(pGraphics, pBrush, x, y, w, h, r) {
   Region := Gdip_GetClipRegion(pGraphics)
   Gdip_SetClipRect(pGraphics, x-r, y-r, 2*r, 2*r, 4)
   Gdip_SetClipRect(pGraphics, x+w-r, y-r, 2*r, 2*r, 4)
   Gdip_SetClipRect(pGraphics, x-r, y+h-r, 2*r, 2*r, 4)
   Gdip_SetClipRect(pGraphics, x+w-r, y+h-r, 2*r, 2*r, 4)
   _E := Gdip_FillRectangle(pGraphics, pBrush, x, y, w, h)
   Gdip_SetClipRegion(pGraphics, Region, 0)
   Gdip_SetClipRect(pGraphics, x-(2*r), y+r, w+(4*r), h-(2*r), 4)
   Gdip_SetClipRect(pGraphics, x+r, y-(2*r), w-(2*r), h+(4*r), 4)
   Gdip_FillEllipse(pGraphics, pBrush, x, y, 2*r, 2*r)
   Gdip_FillEllipse(pGraphics, pBrush, x+w-(2*r), y, 2*r, 2*r)
   Gdip_FillEllipse(pGraphics, pBrush, x, y+h-(2*r), 2*r, 2*r)
   Gdip_FillEllipse(pGraphics, pBrush, x+w-(2*r), y+h-(2*r), 2*r, 2*r)
   Gdip_SetClipRegion(pGraphics, Region, 0)
   Gdip_DeleteRegion(Region)
   return _E
}

;#####################################################################################

; Function           Gdip_FillPolygon
; Description        This function uses a brush to fill a polygon in the Graphics of a bitmap
;
; pGraphics          Pointer to the Graphics of a bitmap
; pBrush             Pointer to a brush
; Points             the coordinates of all the points passed as x1,y1|x2,y2|x3,y3.....
;
; return             status enumeration. 0 = success
;
; notes              Alternate will fill the polygon as a whole, wheras winding will fill each new "segment"
; Alternate          = 0
; Winding            = 1

Gdip_FillPolygon(pGraphics, pBrush, Points, FillMode:=0) {
   Ptr := "UPtr"
   iCount := CreatePointsF(PointsF, Points)
   return DllCall("gdiplus\GdipFillPolygon", Ptr, pGraphics, Ptr, pBrush, Ptr, &PointsF, "int", iCount, "int", FillMode)
}

;#####################################################################################

; Function           Gdip_FillPie
; Description        This function uses a brush to fill a pie in the Graphics of a bitmap
;
; pGraphics          Pointer to the Graphics of a bitmap
; pBrush             Pointer to a brush
; x, y               x, y coordinates of the top left of the pie
; w, h               width and height of the pie
; StartAngle         specifies the angle between the x-axis and the starting point of the pie
; SweepAngle         specifies the angle between the starting and ending points of the pie
;
; return             status enumeration. 0 = success

Gdip_FillPie(pGraphics, pBrush, x, y, w, h, StartAngle, SweepAngle) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipFillPie"
               , Ptr, pGraphics
               , Ptr, pBrush
               , "float", x
               , "float", y
               , "float", w
               , "float", h
               , "float", StartAngle
               , "float", SweepAngle)
}

;#####################################################################################

; Function           Gdip_FillEllipse
; Description        This function uses a brush to fill an ellipse in the Graphics of a bitmap
;
; pGraphics          Pointer to the Graphics of a bitmap
; pBrush             Pointer to a brush
; x, y               x, y coordinates of the top left of the ellipse
; w, h               width and height of the ellipse
;
; return             status enumeration. 0 = success

Gdip_FillEllipse(pGraphics, pBrush, x, y, w, h) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipFillEllipse", Ptr, pGraphics, Ptr, pBrush, "float", x, "float", y, "float", w, "float", h)
}

;#####################################################################################

; Function        Gdip_FillRegion
; Description     This function uses a brush to fill a region in the Graphics of a bitmap
;
; pGraphics       Pointer to the Graphics of a bitmap
; pBrush          Pointer to a brush
; Region          Pointer to a Region
;
; return          status enumeration. 0 = success
;
; notes           You can create a region Gdip_CreateRegion() and then add to this

Gdip_FillRegion(pGraphics, pBrush, Region) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipFillRegion", Ptr, pGraphics, Ptr, pBrush, Ptr, Region)
}

;#####################################################################################

; Function        Gdip_FillPath
; Description     This function uses a brush to fill a path in the Graphics of a bitmap
;
; pGraphics       Pointer to the Graphics of a bitmap
; pBrush          Pointer to a brush
; Region          Pointer to a Path
;
; return          status enumeration. 0 = success

Gdip_FillPath(pGraphics, pBrush, pPath) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipFillPath", Ptr, pGraphics, Ptr, pBrush, Ptr, pPath)
}
;#####################################################################################

; Function        Gdip_FillClosedCurve
; Description     This function fills a closed cardinal spline on a pGraphics object
;                 using a pBrush object.
;                 A cardinal spline is a curve that passes through each point in the array.
;
; pGraphics       Pointer to the Graphics of a bitmap
; pBrush          Pointer to a brush
;
; Points array format:
; Points := "x1,y1|x2,y2|x3,y3|x4,y4" [... and so on]
;
; Tension         Non-negative real number that controls the length of the curve and how the curve bends. A value of
;                 zero specifies that the spline is a sequence of straight lines. As the value increases, the curve becomes fuller.
;                 Number that specifies how tightly the curve bends through the coordinates of the closed cardinal spline.
;
; Fill mode:      0 - [Alternate] The areas are filled according to the even-odd parity rule
;                 1 - [Winding] The areas are filled according to the non-zero winding rule
;
; return          status enumeration. 0 = success

Gdip_FillClosedCurve(pGraphics, pBrush, Points, Tension:="", FillMode:=0) {
   Ptr := "UPtr"
   iCount := CreatePointsF(PointsF, Points)
   If Tension
      Return DllCall("gdiplus\GdipFillClosedCurve2", Ptr, pGraphics, Ptr, pBrush, "UPtr", &PointsF, "int", iCount, "float", Tension, "int", FillMode)
   Else
      Return DllCall("gdiplus\GdipFillClosedCurve", Ptr, pGraphics, Ptr, pBrush, "UPtr", &PointsF, "int", iCount)
}

;#####################################################################################

; Function        Gdip_DrawImagePointsRect
; Description     This function draws a bitmap into the Graphics of another bitmap and skews it
;
; pGraphics       Pointer to the Graphics of a bitmap
; pBitmap         Pointer to a bitmap to be drawn
; Points          Points passed as x1,y1|x2,y2|x3,y3 (3 points: top left, top right, bottom left) describing the drawing of the bitmap
; sX, sY          x, y coordinates of the source upper-left corner
; sW, sH          width and height of the source rectangle
; Matrix          a color matrix used to alter image attributes when drawing
; Unit            see Gdip_DrawImage()
; Return          status enumeration. 0 = success
;
; Notes           If sx, sy, sw, sh are omitted the entire source bitmap will be used.
;                 Matrix can be omitted to just draw with no alteration to ARGB.
;                 Matrix may be passed as a digit from 0 - 1 to change just transparency.
;                 Matrix can be passed as a matrix with "|" delimiter.
;                 To generate a color matrix using user-friendly parameters,
;                 use GenerateColorMatrix()

Gdip_DrawImagePointsRect(pGraphics, pBitmap, Points, sx:="", sy:="", sw:="", sh:="", Matrix:=1, Unit:=2, ImageAttr:=0) {
   Ptr := "UPtr"
   If !ImageAttr
   {
      if !IsNumber(Matrix)
         ImageAttr := Gdip_SetImageAttributesColorMatrix(Matrix)
      else if (Matrix != 1)
         ImageAttr := Gdip_SetImageAttributesColorMatrix("1|0|0|0|0|0|1|0|0|0|0|0|1|0|0|0|0|0|" Matrix "|0|0|0|0|0|1")
   } Else usrImageAttr := 1

   if (sx="" && sy="" && sw="" && sh="")
   {
      sx := sy := 0
      Gdip_GetImageDimensions(pBitmap, sw, sh)
   }

   iCount := CreatePointsF(PointsF, Points)
   _E := DllCall("gdiplus\GdipDrawImagePointsRect"
            , Ptr, pGraphics
            , Ptr, pBitmap
            , Ptr, &PointsF
            , "int", iCount
            , "float", sX
            , "float", sY
            , "float", sW
            , "float", sH
            , "int", Unit
            , Ptr, ImageAttr ? ImageAttr : 0
            , Ptr, 0
            , Ptr, 0)

   if (ImageAttr && usrImageAttr!=1)
      Gdip_DisposeImageAttributes(ImageAttr)

   return _E
}

;#####################################################################################

; Function        Gdip_DrawImage
; Description     This function draws a bitmap into the Graphics of another bitmap
;
; pGraphics       Pointer to the Graphics of a bitmap
; pBitmap         Pointer to a bitmap to be drawn
; dX, dY          x, y coordinates of the destination upper-left corner
; dW, dH          width and height of the destination image
; sX, sY          x, y coordinates of the source upper-left corner
; sW, sH          width and height of the source image
; Matrix          a color matrix used to alter image attributes when drawing
; Unit            Unit of measurement:
;                 0 - World coordinates, a nonphysical unit
;                 1 - Display units
;                 2 - A unit is 1 pixel
;                 3 - A unit is 1 point or 1/72 inch
;                 4 - A unit is 1 inch
;                 5 - A unit is 1/300 inch
;                 6 - A unit is 1 millimeter
;
; return          status enumeration. 0 = success
;
; notes           When sx,sy,sw,sh are omitted the entire source bitmap will be used
;                 Gdip_DrawImage performs faster.
;                 Matrix can be omitted to just draw with no alteration to ARGB
;                 Matrix may be passed as a digit from 0.0 - 1.0 to change just transparency
;                 Matrix can be passed as a matrix with "|" as delimiter. For example:
;                 MatrixBright=
;                 (
;                 1.5   |0    |0    |0    |0
;                 0     |1.5  |0    |0    |0
;                 0     |0    |1.5  |0    |0
;                 0     |0    |0    |1    |0
;                 0.05  |0.05 |0.05 |0    |1
;                 )
;
; example color matrix:
;                 MatrixBright = 1.5|0|0|0|0|0|1.5|0|0|0|0|0|1.5|0|0|0|0|0|1|0|0.05|0.05|0.05|0|1
;                 MatrixGreyScale = 0.299|0.299|0.299|0|0|0.587|0.587|0.587|0|0|0.114|0.114|0.114|0|0|0|0|0|1|0|0|0|0|0|1
;                 MatrixNegative = -1|0|0|0|0|0|-1|0|0|0|0|0|-1|0|0|0|0|0|1|0|1|1|1|0|1
;                 To generate a color matrix using user-friendly parameters,
;                 use GenerateColorMatrix()

Gdip_DrawImage(pGraphics, pBitmap, dx:="", dy:="", dw:="", dh:="", sx:="", sy:="", sw:="", sh:="", Matrix:=1, Unit:=2, ImageAttr:=0) {
   Ptr := "UPtr"
   If !ImageAttr
   {
      if !IsNumber(Matrix)
         ImageAttr := Gdip_SetImageAttributesColorMatrix(Matrix)
      else if (Matrix!=1)
         ImageAttr := Gdip_SetImageAttributesColorMatrix("1|0|0|0|0|0|1|0|0|0|0|0|1|0|0|0|0|0|" Matrix "|0|0|0|0|0|1")
   } Else usrImageAttr := 1

   If (dx!="" && dy!="" && dw="" && dh="" && sx="" && sy="" && sw="" && sh="")
   {
      sx := sy := 0
      sw := dw := Gdip_GetImageWidth(pBitmap)
      sh := dh := Gdip_GetImageHeight(pBitmap)
   } Else If (sx="" && sy="" && sw="" && sh="")
   {
      If (dx="" && dy="" && dw="" && dh="")
      {
         sx := dx := 0, sy := dy := 0
         sw := dw := Gdip_GetImageWidth(pBitmap)
         sh := dh := Gdip_GetImageHeight(pBitmap)
      } Else
      {
         sx := sy := 0
         Gdip_GetImageDimensions(pBitmap, sw, sh)
      }
   }

   _E := DllCall("gdiplus\GdipDrawImageRectRect"
            , Ptr, pGraphics
            , Ptr, pBitmap
            , "float", dX, "float", dY
            , "float", dW, "float", dH
            , "float", sX, "float", sY
            , "float", sW, "float", sH
            , "int", Unit
            , Ptr, ImageAttr ? ImageAttr : 0
            , Ptr, 0, Ptr, 0)

   if (ImageAttr && usrImageAttr!=1)
      Gdip_DisposeImageAttributes(ImageAttr)

   return _E
}

Gdip_DrawImageFast(pGraphics, pBitmap, X:=0, Y:=0) {
; This function performs faster than Gdip_DrawImage().
; X, Y - the coordinates of the destination upper-left corner
; where the pBitmap will be drawn.

   Ptr := "UPtr"
   _E := DllCall("gdiplus\GdipDrawImage"
            , Ptr, pGraphics
            , Ptr, pBitmap
            , "float", X
            , "float", Y)
   return _E
}

Gdip_DrawImageRect(pGraphics, pBitmap, X, Y, W, H) {
; X, Y - the coordinates of the destination upper-left corner
; where the pBitmap will be drawn.
; W, H - the width and height of the destination rectangle, where the pBitmap will be drawn.

   Ptr := "UPtr"
   _E := DllCall("gdiplus\GdipDrawImageRect"
            , Ptr, pGraphics
            , Ptr, pBitmap
            , "float", X, "float", Y
            , "float", W, "float", H)
   return _E
}

;#####################################################################################

; Function        Gdip_SetImageAttributesColorMatrix
; Description     This function creates an image color matrix ready for drawing if no ImageAttr is given.
;                 It can set or clear the color and/or grayscale-adjustment matrices for a specified ImageAttr object.
;
; clrMatrix       A color-adjustment matrix used to alter image attributes when drawing
;                 passed with "|" as delimeter.
; grayMatrix      A grayscale-adjustment matrix used to alter image attributes when drawing
;                 passed with "|" as delimeter. This applies only when ColorMatrixFlag=2.
;
; ColorAdjustType The category for which the color and grayscale-adjustment matrices are set or cleared.
;                 0 - adjustments apply to all categories that do not have adjustment settings of their own
;                 1 - adjustments apply to bitmapped images
;                 2 - adjustments apply to brush operations in metafiles
;                 3 - adjustments apply to pen operations in metafiles
;                 4 - adjustments apply to text drawn in metafiles
;
; fEnable         If True, the specified matrices (color, grayscale or both) adjustments for the specified
;                 category are applied; otherwise the category is cleared
;
; ColorMatrixFlag Type of image and color that will be affected by the adjustment matrices:
;                 0 - All color values (including grays) are adjusted by the same color-adjustment matrix.
;                 1 - Colors are adjusted but gray shades are not adjusted.
;                     A gray shade is any color that has the same value for its red, green, and blue components.
;                 2 - Colors are adjusted by one matrix and gray shades are adjusted by another matrix.

; ImageAttr       A pointer to an ImageAttributes object.
;                 If this parameter is omitted, a new one is created.

; return          It return 0 on success, if an ImageAttr object was given,
;                 otherwise, it returns the handle of a new ImageAttr object [if succesful].
;
; notes           MatrixBright = 1.5|0|0|0|0|0|1.5|0|0|0|0|0|1.5|0|0|0|0|0|1|0|0.05|0.05|0.05|0|1
;                 MatrixGreyScale = 0.299|0.299|0.299|0|0|0.587|0.587|0.587|0|0|0.114|0.114|0.114|0|0|0|0|0|1|0|0|0|0|0|1
;                 MatrixNegative = -1|0|0|0|0|0|-1|0|0|0|0|0|-1|0|0|0|0|0|1|0|1|1|1|0|1
;                 To generate a color matrix using user-friendly parameters,
;                 use GenerateColorMatrix()
; additional remarks:
; In my tests, it seems that the grayscale matrix is not functioning properly.
; Grayscale images are rendered invisible [with zero opacity] for some reason...
; TO DO: fix this?

Gdip_SetImageAttributesColorMatrix(clrMatrix, ImageAttr:=0, grayMatrix:=0, ColorAdjustType:=1, fEnable:=1, ColorMatrixFlag:=0) {
   Ptr := "UPtr"
   If (StrLen(clrMatrix)<5 && ImageAttr)
      Return -1

   If StrLen(clrMatrix)<5
      Return

   VarSetCapacity(ColourMatrix, 100, 0)
   Matrix := RegExReplace(RegExReplace(clrMatrix, "^[^\d-\.]+([\d\.])", "$1", , 1), "[^\d-\.]+", "|")
   Matrix := StrSplit(Matrix, "|")
   Loop (25)
   {
      M := (Matrix[A_Index] != "") ? Matrix[A_Index] : Mod(A_Index-1, 6) ? 0 : 1
      NumPut(M, ColourMatrix, (A_Index-1)*4, "float")
   }

   Matrix := ""
   Matrix := RegExReplace(RegExReplace(grayMatrix, "^[^\d-\.]+([\d\.])", "$1", , 1), "[^\d-\.]+", "|")
   Matrix := StrSplit(Matrix, "|")
   If (StrLen(Matrix)>2 && ColorMatrixFlag=2)
   {
      VarSetCapacity(GrayscaleMatrix, 100, 0)
      Loop (25)
      {
         M := (Matrix[A_Index] != "") ? Matrix[A_Index] : Mod(A_Index-1, 6) ? 0 : 1
         NumPut(M, GrayscaleMatrix, (A_Index-1)*4, "float")
      }
   }

   If !ImageAttr
   {
      created := 1
      ImageAttr := Gdip_CreateImageAttributes()
   }

   E := DllCall("gdiplus\GdipSetImageAttributesColorMatrix"
         , Ptr, ImageAttr
         , "int", ColorAdjustType
         , "int", fEnable
         , Ptr, &ColourMatrix
         , Ptr, &GrayscaleMatrix
         , "int", ColorMatrixFlag)

   E := created=1 ? ImageAttr : E
   return E
}

Gdip_CreateImageAttributes() {
   ImageAttr := 0
   DllCall("gdiplus\GdipCreateImageAttributes", "UPtr*", ImageAttr)
   return ImageAttr
}

Gdip_CloneImageAttributes(ImageAttr) {
   Ptr := "UPtr"
   newImageAttr := 0
   DllCall("gdiplus\GdipCloneImageAttributes", Ptr, ImageAttr, "UPtr*", newImageAttr)
   return newImageAttr
}

Gdip_SetImageAttributesThreshold(ImageAttr, Threshold, ColorAdjustType:=1, fEnable:=1) {
; Sets or clears the threshold (transparency range) for a specified category by ColorAdjustType
; The threshold is a value from 0 through 1 that specifies a cutoff point for each color component. For example,
; suppose the threshold is set to 0.7, and suppose you are rendering a color whose red, green, and blue
; components are 230, 50, and 220. The red component, 230, is greater than 0.7ª255, so the red component will
; be changed to 255 (full intensity). The green component, 50, is less than 0.7ª255, so the green component will
; be changed to 0. The blue component, 220, is greater than 0.7ª255, so the blue component will be changed to 255.

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetImageAttributesThreshold", Ptr, ImageAttr, "int", ColorAdjustType, "int", fEnable, "float", Threshold)
}

Gdip_SetImageAttributesResetMatrix(ImageAttr, ColorAdjustType) {
; Sets the color-adjustment matrix of a specified category to the identity matrix.

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetImageAttributesToIdentity", Ptr, ImageAttr, "int", ColorAdjustType)
}

Gdip_SetImageAttributesGamma(ImageAttr, Gamma, ColorAdjustType:=1, fEnable:=1) {
; Gamma from 0.1 to 5.0

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetImageAttributesGamma", Ptr, ImageAttr, "int", ColorAdjustType, "int", fEnable, "float", Gamma)
}

Gdip_SetImageAttributesToggle(ImageAttr, ColorAdjustType, fEnable) {
; Turns on or off color adjustment for a specified category defined by ColorAdjustType
; fEnable - 0 or 1

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetImageAttributesNoOp", Ptr, ImageAttr, "int", ColorAdjustType, "int", fEnable)
}

Gdip_SetImageAttributesOutputChannel(ImageAttr, ColorChannelFlags, ColorAdjustType:=1, fEnable:=1) {
; ColorChannelFlags - The output channel, can be any combination:
; 0 - Cyan color channel
; 1 - Magenta color channel
; 2 - Yellow color channel
; 3 - Black color channel
; 4 - The previous selected channel

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetImageAttributesOutputChannel", Ptr, ImageAttr, "int", ColorAdjustType, "int", fEnable, "int", ColorChannelFlags)
}

Gdip_SetImageAttributesColorKeys(ImageAttr, ARGBLow, ARGBHigh, ColorAdjustType:=1, fEnable:=1) {
; initial tests of this function lead to a crash of the application ...

   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipSetImageAttributesColorKeys", Ptr, ImageAttr, "int", ColorAdjustType, "int", fEnable, "uint", ARGBLow, "uint", ARGBHigh)
}

Gdip_SetImageAttributesWrapMode(ImageAttr, WrapMode, ARGB) {
; ImageAttr - Pointer to an ImageAttribute object
; WrapMode  - Specifies how repeated copies of an image are used to tile an area:
;             0 - Tile - Tiling without flipping
;             1 - TileFlipX - Tiles are flipped horizontally as you move from one tile to the next in a row
;             2 - TileFlipY - Tiles are flipped vertically as you move from one tile to the next in a column
;             3 - TileFlipXY - Tiles are flipped horizontally as you move along a row and flipped vertically as you move along a column
;             4 - Clamp - No tiling takes place
; ARGB      - Alpha, Red, Green and Blue components of the color of pixels outside of a rendered image.
;             This color is visible if the wrap mode is set to 4 and the source rectangle of the image is greater than the
;             image itself.

   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipSetImageAttributesWrapMode", Ptr, ImageAttr, "int", WrapMode, "uint", ARGB, "int", 0)
}

Gdip_ResetImageAttributes(ImageAttr, ColorAdjustType) {
; Clears all color and grayscale-adjustment settings for a specified category defined by ColorAdjustType.
;
; ImageAttr - a pointer to an ImageAttributes object.
; ColorAdjustType - The category for which color adjustment is reset:
; see Gdip_SetImageAttributesColorMatrix() for details.

   Ptr := "UPtr"
   DllCall("gdiplus\GdipResetImageAttributes", Ptr, ImageAttr, "int", ColorAdjustType)
}

;#####################################################################################

; Function           Gdip_GraphicsFromImage
; Description        This function gets the graphics for a bitmap used for drawing functions
;
; pBitmap            Pointer to a bitmap to get the pointer to its graphics
;
; return             returns a pointer to the graphics of a bitmap
;
; notes              a bitmap can be drawn into the graphics of another bitmap

Gdip_GraphicsFromImage(pBitmap, InterpolationMode:="", SmoothingMode:="", PageUnit:="", CompositingQuality:="") {
   pGraphics := 0
   DllCall("gdiplus\GdipGetImageGraphicsContext", "UPtr", pBitmap, "UPtr*", pGraphics)
   If pGraphics
   {
      If (InterpolationMode!="")
         Gdip_SetInterpolationMode(pGraphics, InterpolationMode)
      If (SmoothingMode!="")
         Gdip_SetSmoothingMode(pGraphics, SmoothingMode)
      If (PageUnit!="")
         Gdip_SetPageUnit(pGraphics, PageUnit)
      If (CompositingQuality!="")
         Gdip_SetCompositingMode(pGraphics, CompositingQuality)
   }
   return pGraphics
}

;#####################################################################################

; Function           Gdip_GraphicsFromHDC
; Description        This function gets the graphics from the handle of a device context.
;
; hDC                The handle to the device context.
; hDevice            Handle to a device that will be associated with the new Graphics object.
;
; return             A pointer to the graphics of a bitmap.
;
; notes              You can draw a bitmap into the graphics of another bitmap.

Gdip_GraphicsFromHDC(hDC, hDevice:="", InterpolationMode:="", SmoothingMode:="", PageUnit:="", CompositingQuality:="") {
   pGraphics := 0
   If hDevice
      DllCall("Gdiplus\GdipCreateFromHDC2", "UPtr", hDC, "UPtr", hDevice, "UPtr*", pGraphics)
   Else
      DllCall("gdiplus\GdipCreateFromHDC", "UPtr", hdc, "UPtr*", pGraphics)

   If pGraphics
   {
      If (InterpolationMode!="")
         Gdip_SetInterpolationMode(pGraphics, InterpolationMode)
      If (SmoothingMode!="")
         Gdip_SetSmoothingMode(pGraphics, SmoothingMode)
      If (PageUnit!="")
         Gdip_SetPageUnit(pGraphics, PageUnit)
      If (CompositingQuality!="")
         Gdip_SetCompositingMode(pGraphics, CompositingQuality)
   }

   return pGraphics
}

Gdip_GraphicsFromHWND(HWND, useICM:=0, InterpolationMode:="", SmoothingMode:="", PageUnit:="", CompositingQuality:="") {
; Creates a pGraphics object that is associated with a specified window handle [HWND]
; If useICM=1, the created graphics uses ICM [color management - (International Color Consortium = ICC)].
   pGraphics := 0
   function2call := (useICM=1) ? "GdipCreateFromHWNDICM" : "GdipCreateFromHWND"
   DllCall("gdiplus\" function2call, "UPtr", HWND, "UPtr*", pGraphics)

   If pGraphics
   {
      If (InterpolationMode!="")
         Gdip_SetInterpolationMode(pGraphics, InterpolationMode)
      If (SmoothingMode!="")
         Gdip_SetSmoothingMode(pGraphics, SmoothingMode)
      If (PageUnit!="")
         Gdip_SetPageUnit(pGraphics, PageUnit)
      If (CompositingQuality!="")
         Gdip_SetCompositingMode(pGraphics, CompositingQuality)
   }
   return pGraphics
}

;#####################################################################################

; Function           Gdip_GetDC
; Description        This function gets the device context of the passed Graphics
;
; hDC                This is the handle to the device context
;
; return             returns the device context for the graphics of a bitmap

Gdip_GetDC(pGraphics) {
   hDC := 0
   DllCall("gdiplus\GdipGetDC", "UPtr", pGraphics, "UPtr*", hDC)
   return hDC
}

;#####################################################################################

; Function           Gdip_ReleaseDC
; Description        This function releases a device context from use for further use
;
; pGraphics          Pointer to the graphics of a bitmap
; hdc                This is the handle to the device context
;
; return             status enumeration. 0 = success

Gdip_ReleaseDC(pGraphics, hdc) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipReleaseDC", Ptr, pGraphics, Ptr, hdc)
}

;#####################################################################################

; Function           Gdip_GraphicsClear
; Description        Clears the graphics of a bitmap ready for further drawing
;
; pGraphics          Pointer to the graphics of a bitmap
; ARGB               The colour to clear the graphics to
;
; return             status enumeration. 0 = success
;
; notes              By default this will make the background invisible
;                    Using clipping regions you can clear a particular area on the graphics rather than clearing the entire graphics

Gdip_GraphicsClear(pGraphics, ARGB:=0x00ffffff) {
   return DllCall("gdiplus\GdipGraphicsClear", "UPtr", pGraphics, "int", ARGB)
}

Gdip_GraphicsFlush(pGraphics, intent) {
; intent - Specifies whether the method returns immediately or waits for any existing operations to finish:
; 0 - Flush all batched rendering operations and return immediately
; 1 - Flush all batched rendering operations and wait for them to complete

   return DllCall("gdiplus\GdipFlush", "UPtr", pGraphics, "int", intent)
}

;#####################################################################################

; Function           Gdip_BlurBitmap
; Description        Gives a pointer to a blurred bitmap from a pointer to a bitmap
;
; pBitmap            Pointer to a bitmap to be blurred
; BlurAmount         The Amount to blur a bitmap by from 1 (least blur) to 100 (most blur)
;
; return             If the function succeeds, the return value is a pointer to the new blurred bitmap
;                    -1 = The blur parameter is outside the range 1-100
;
; notes              This function will not dispose of the original bitmap

Gdip_BlurBitmap(pBitmap, BlurAmount) {
   if (BlurAmount > 100) || (BlurAmount < 1)
      return -1

   Gdip_GetImageDimensions(pBitmap, sWidth, sHeight)
   dWidth := sWidth//BlurAmount
   dHeight := sHeight//BlurAmount

   pBitmap1 := Gdip_CreateBitmap(dWidth, dHeight)
   G1 := Gdip_GraphicsFromImage(pBitmap1)
   Gdip_SetInterpolationMode(G1, 7)
   Gdip_DrawImage(G1, pBitmap, 0, 0, dWidth, dHeight, 0, 0, sWidth, sHeight)

   Gdip_DeleteGraphics(G1)
   pBitmap2 := Gdip_CreateBitmap(sWidth, sHeight)
   G2 := Gdip_GraphicsFromImage(pBitmap2)
   Gdip_SetInterpolationMode(G2, 7)
   Gdip_DrawImage(G2, pBitmap1, 0, 0, sWidth, sHeight, 0, 0, dWidth, dHeight)

   Gdip_DeleteGraphics(G2)
   Gdip_DisposeImage(pBitmap1)
   return pBitmap2
}

;#####################################################################################

; Function:        Gdip_SaveBitmapToFile
; Description:     Saves a bitmap to a file in any supported format onto disk
;
; pBitmap          Pointer to a bitmap
; sOutput          The name of the file that the bitmap will be saved to. Supported extensions are: .BMP,.DIB,.RLE,.JPG,.JPEG,.JPE,.JFIF,.GIF,.TIF,.TIFF,.PNG
; Quality          If saving as jpg (.JPG,.JPEG,.JPE,.JFIF) then quality can be 1-100 with default at maximum quality
; toBase64         If set to 1, instead of saving the file to disk, the function will return on success the base64 data
;                  A "base64" string is the binary image data encoded into text using only 64 characters.
;                  To convert it back into an image use: Gdip_BitmapFromBase64()
;
; return           If the function succeeds, the return value is zero, otherwise:
;                 -1 = Extension supplied is not a supported file format
;                 -2 = Could not get a list of encoders on system
;                 -3 = Could not find matching encoder for specified file format
;                 -4 = Could not get WideChar name of output file
;                 -5 = Could not save file to disk
;                 -6 = Could not save image to stream [for base64]
;                 -7 = Could not convert to base64
;
; notes            This function will use the extension supplied from the sOutput parameter to determine the output format

Gdip_SaveBitmapToFile(pBitmap, sOutput, Quality:=75, toBase64:=0) {
   Ptr := "UPtr"
   nCount := 0
   nSize := 0
   _p := 0

   SplitPath sOutput,,, Extension
   If !RegExMatch(Extension, "^(?i:BMP|DIB|RLE|JPG|JPEG|JPE|JFIF|GIF|TIF|TIFF|PNG)$")
      Return -1

   Extension := "." Extension
   DllCall("gdiplus\GdipGetImageEncodersSize", "uint*", nCount, "uint*", nSize)
   VarSetCapacity(ci, nSize)
   DllCall("gdiplus\GdipGetImageEncoders", "uint", nCount, "uint", nSize, Ptr, &ci)
   If !(nCount && nSize)
      Return -2

   If (A_IsUnicode)
   {
      StrGet_Name := "StrGet"
      Loop (nCount)
      {
         sString := %StrGet_Name%(NumGet(ci, (idx := (48+7*A_PtrSize)*(A_Index-1))+32+3*A_PtrSize), "UTF-16")
         If !InStr(sString, "*" Extension)
            Continue

         pCodec := &ci+idx
         Break
      }
   } Else
   {
      Loop (nCount)
      {
         Location := NumGet(ci, 76*(A_Index-1)+44)
         nSize := DllCall("WideCharToMultiByte", "uint", 0, "uint", 0, "uint", Location, "int", -1, "uint", 0, "int",  0, "uint", 0, "uint", 0)
         VarSetCapacity(sString, nSize)
         DllCall("WideCharToMultiByte", "uint", 0, "uint", 0, "uint", Location, "int", -1, "str", sString, "int", nSize, "uint", 0, "uint", 0)
         If !InStr(sString, "*" Extension)
            Continue

         pCodec := &ci+76*(A_Index-1)
         Break
      }
   }

   If !pCodec
      Return -3

   If (Quality!=75)
   {
      Quality := (Quality < 0) ? 0 : (Quality > 100) ? 100 : Quality
      If (quality>90 && toBase64=1)
         Quality := 90

      If RegExMatch(Extension, "^\.(?i:JPG|JPEG|JPE|JFIF)$")
      {
         DllCall("gdiplus\GdipGetEncoderParameterListSize", Ptr, pBitmap, Ptr, pCodec, "uint*", nSize)
         VarSetCapacity(EncoderParameters, nSize, 0)
         DllCall("gdiplus\GdipGetEncoderParameterList", Ptr, pBitmap, Ptr, pCodec, "uint", nSize, Ptr, &EncoderParameters)
         nCount := NumGet(EncoderParameters, "UInt")
         Loop (nCount)
         {
            elem := (24+A_PtrSize)*(A_Index-1) + 4 + (pad := A_PtrSize = 8 ? 4 : 0)
            If (NumGet(EncoderParameters, elem+16, "UInt") = 1) && (NumGet(EncoderParameters, elem+20, "UInt") = 6)
            {
               _p := elem+&EncoderParameters-pad-4
               NumPut(Quality, NumGet(NumPut(4, NumPut(1, _p+0)+20, "UInt")), "UInt")
               Break
            }
         }
      }
   }

   If (toBase64=1)
   {
      ; part of the function extracted from ImagePut by iseahound
      ; https://www.autohotkey.com/boards/viewtopic.php?f=6&t=76301&sid=bfb7c648736849c3c53f08ea6b0b1309
      DllCall("ole32\CreateStreamOnHGlobal", "ptr",0, "int",true, "ptr*",pStream)
      _E := DllCall("gdiplus\GdipSaveImageToStream", "ptr",pBitmap, "ptr",pStream, "ptr",pCodec, "uint", _p ? _p : 0)
      If _E
         Return -6

      DllCall("ole32\GetHGlobalFromStream", "ptr",pStream, "uint*",hData)
      pData := DllCall("GlobalLock", "ptr",hData, "ptr")
      nSize := DllCall("GlobalSize", "uint",pData)

      VarSetCapacity(bin, nSize, 0)
      DllCall("RtlMoveMemory", "ptr",&bin, "ptr",pData, "uptr",nSize)
      DllCall("GlobalUnlock", "ptr",hData)
      ObjRelease(pStream)
      DllCall("GlobalFree", "ptr",hData)

      ; Using CryptBinaryToStringA saves about 2MB in memory.
      DllCall("Crypt32.dll\CryptBinaryToStringA", "ptr",&bin, "uint",nSize, "uint",0x40000001, "ptr",0, "uint*",base64Length)
      VarSetCapacity(base64, base64Length, 0)
      _E := DllCall("Crypt32.dll\CryptBinaryToStringA", "ptr",&bin, "uint",nSize, "uint",0x40000001, "ptr",&base64, "uint*",base64Length)
      If !_E
         Return -7

      VarSetCapacity(bin, 0)
      Return StrGet(&base64, base64Length, "CP0")
   }

   If (!A_IsUnicode)
   {
      nSize := DllCall("MultiByteToWideChar", "uint", 0, "uint", 0, Ptr, &sOutput, "int", -1, Ptr, 0, "int", 0)
      VarSetCapacity(wOutput, nSize*2)
      DllCall("MultiByteToWideChar", "uint", 0, "uint", 0, Ptr, &sOutput, "int", -1, Ptr, &wOutput, "int", nSize)
      VarSetCapacity(wOutput, -1)
      If !VarSetCapacity(wOutput)
         Return -4
      _E := DllCall("gdiplus\GdipSaveImageToFile", Ptr, pBitmap, Ptr, &wOutput, Ptr, pCodec, "uint", _p ? _p : 0)
   } Else
      _E := DllCall("gdiplus\GdipSaveImageToFile", Ptr, pBitmap, Ptr, &sOutput, Ptr, pCodec, "uint", _p ? _p : 0)

   Return _E ? -5 : 0
}

;#####################################################################################

; Function           Gdip_GetPixel
; Description        Gets the ARGB of a pixel in a bitmap
;
; pBitmap            Pointer to a bitmap
; x, y               x, y coordinates of the pixel
;
; return             Returns the ARGB value of the pixel

Gdip_GetPixel(pBitmap, x, y) {
   ARGB := 0
   DllCall("gdiplus\GdipBitmapGetPixel", "UPtr", pBitmap, "int", x, "int", y, "uint*", ARGB)
   return ARGB
   ; should use Format("{1:#x}", ARGB)
}

Gdip_GetPixelColor(pBitmap, x, y, Format) {
   ARGBdec := Gdip_GetPixel(pBitmap, x, y)
   If (format=1)  ; in ARGB [HEX; 00-FF] with 0x prefix
   {
      Return Format("{1:#x}", ARGBdec)
   } Else If (format=2)  ; in RGBA [0-255]
   {
      Gdip_FromARGB(ARGBdec, A, R, G, B)
      Return R "," G "," B "," A
   } Else If (format=3)  ; in BGR [HEX; 00-FF] with 0x prefix
   {
      clr := Format("{1:#x}", ARGBdec)
      Return "0x" SubStr(clr, -1) SubStr(clr, 7, 2) SubStr(clr, 5, 2)
   } Else If (format=4)  ; in RGB [HEX; 00-FF] with no prefix
   {
      Return SubStr(Format("{1:#x}", ARGBdec), 5)
   } Else Return ARGBdec
}

;#####################################################################################

; Function           Gdip_SetPixel
; Description        Sets the ARGB of a pixel in a bitmap
;
; pBitmap            Pointer to a bitmap
; x, y               x, y coordinates of the pixel
;
; return             status enumeration. 0 = success

Gdip_SetPixel(pBitmap, x, y, ARGB) {
   return DllCall("gdiplus\GdipBitmapSetPixel", "UPtr", pBitmap, "int", x, "int", y, "int", ARGB)
}

;#####################################################################################

; Function           Gdip_GetImageWidth
; Description        Gives the width of a bitmap
;
; pBitmap            Pointer to a bitmap
;
; return             Returns the width in pixels of the supplied bitmap

Gdip_GetImageWidth(pBitmap) {
   Width := 0
   DllCall("gdiplus\GdipGetImageWidth", "UPtr", pBitmap, "uint*", Width)
   return Width
}

;#####################################################################################

; Function           Gdip_GetImageHeight
; Description        Gives the height of a bitmap
;
; pBitmap            Pointer to a bitmap
;
; return             Returns the height in pixels of the supplied bitmap

Gdip_GetImageHeight(pBitmap) {
   Height := 0
   DllCall("gdiplus\GdipGetImageHeight", "UPtr", pBitmap, "uint*", Height)
   return Height
}

;#####################################################################################

; Function           Gdip_GetImageDimensions
; Description        Gives the width and height of a bitmap
;
; pBitmap            Pointer to a bitmap
; Width              ByRef variable. This variable will be set to the width of the bitmap
; Height             ByRef variable. This variable will be set to the height of the bitmap
;
; return             GDI+ status enumeration return value

Gdip_GetImageDimensions(pBitmap, ByRef Width, ByRef Height) {
   If StrLen(pBitmap)<3
      Return -1

   Width := 0, Height := 0
   E := Gdip_GetImageDimension(pBitmap, Width, Height)
   Width := Round(Width)
   Height := Round(Height)
   return E
}

Gdip_GetImageDimension(pBitmap, ByRef w, ByRef h) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipGetImageDimension", Ptr, pBitmap, "float*", w, "float*", h)
}

Gdip_GetImageBounds(pBitmap) {
  Ptr := "UPtr"
  rData := {}

  VarSetCapacity(RectF, 16, 0)
  status := DllCall("gdiplus\GdipGetImageBounds", Ptr, pBitmap, Ptr, &RectF, "Int*", 0)

  If (!status) {
        rData.x := NumGet(&RectF, 0, "float")
      , rData.y := NumGet(&RectF, 4, "float")
      , rData.w := NumGet(&RectF, 8, "float")
      , rData.h := NumGet(&RectF, 12, "float")
  } Else {
    Return status
  }

  return rData
}

Gdip_GetImageFlags(pBitmap) {
; Gets a set of flags that indicate certain attributes of this Image object.
; Returns an element of the ImageFlags Enumeration that holds a set of single-bit flags.
; ImageFlags enumeration
   ; None              := 0x0000  ; Specifies no format information.
   ; ; Low-word: shared with SINKFLAG_x:
   ; Scalable          := 0x00001  ; the image can be scaled.
   ; HasAlpha          := 0x00002  ; the pixel data contains alpha values.
   ; HasTranslucent    := 0x00004  ; the pixel data has alpha values other than 0 (transparent) and 255 (opaque).
   ; PartiallyScalable := 0x00008  ; the pixel data is partially scalable with some limitations.
   ; ; Low-word: color space definition:
   ; ColorSpaceRGB     := 0x00010  ; the image is stored using an RGB color space.
   ; ColorSpaceCMYK    := 0x00020  ; the image is stored using a CMYK color space.
   ; ColorSpaceGRAY    := 0x00040  ; the image is a grayscale image.
   ; ColorSpaceYCBCR   := 0x00080  ; the image is stored using a YCBCR color space.
   ; ColorSpaceYCCK    := 0x00100  ; the image is stored using a YCCK color space.
   ; ; Low-word: image size info:
   ; HasRealDPI        := 0x01000  ; dots per inch information is stored in the image.
   ; HasRealPixelSize  := 0x02000  ; the pixel size is stored in the image.
   ; ; High-word:
   ; ReadOnly          := 0x10000  ; the pixel data is read-only.
   ; Caching           := 0x20000  ; the pixel data can be cached for faster access.
; function extracted from : https://github.com/flipeador/Library-AutoHotkey/tree/master/graphics
; by flipeador

   Flags := 0
   DllCall("Gdiplus.dll\GdipGetImageFlags", "Ptr", pBitmap, "UInt*", Flags)
   Return Flags
}

Gdip_GetImageRawFormat(pBitmap) {
; retrieves the pBitmap [file] format

  Static RawFormatsList := {"{B96B3CA9-0728-11D3-9D7B-0000F81EF32E}":"Undefined", "{B96B3CAA-0728-11D3-9D7B-0000F81EF32E}":"MemoryBMP", "{B96B3CAB-0728-11D3-9D7B-0000F81EF32E}":"BMP", "{B96B3CAC-0728-11D3-9D7B-0000F81EF32E}":"EMF", "{B96B3CAD-0728-11D3-9D7B-0000F81EF32E}":"WMF", "{B96B3CAE-0728-11D3-9D7B-0000F81EF32E}":"JPEG", "{B96B3CAF-0728-11D3-9D7B-0000F81EF32E}":"PNG", "{B96B3CB0-0728-11D3-9D7B-0000F81EF32E}":"GIF", "{B96B3CB1-0728-11D3-9D7B-0000F81EF32E}":"TIFF", "{B96B3CB2-0728-11D3-9D7B-0000F81EF32E}":"EXIF", "{B96B3CB5-0728-11D3-9D7B-0000F81EF32E}":"Icon"}
  Ptr := "UPtr"
  VarSetCapacity(pGuid, 16, 0)
  E1 := DllCall("gdiplus\GdipGetImageRawFormat", Ptr, pBitmap, "Ptr", &pGuid)

  size := VarSetCapacity(sguid, (38 << !!A_IsUnicode) + 1, 0)
  E2 := DllCall("ole32.dll\StringFromGUID2", "ptr", &pguid, "ptr", &sguid, "int", size)
  R1 := E2 ? StrGet(&sguid) : E2
  R2 := RawFormatsList[R1]
  Return R2 ? R2 : R1
}

Gdip_GetImagePixelFormat(pBitmap, mode:=0) {
; Mode options 
; 0 - in decimal
; 1 - in hex
; 2 - in human readable format
;
; PXF01INDEXED = 0x00030101  ; 1 bpp, indexed
; PXF04INDEXED = 0x00030402  ; 4 bpp, indexed
; PXF08INDEXED = 0x00030803  ; 8 bpp, indexed
; PXF16GRAYSCALE = 0x00101004; 16 bpp, grayscale
; PXF16RGB555 = 0x00021005   ; 16 bpp; 5 bits for each RGB
; PXF16RGB565 = 0x00021006   ; 16 bpp; 5 bits red, 6 bits green, and 5 bits blue
; PXF16ARGB1555 = 0x00061007 ; 16 bpp; 1 bit for alpha and 5 bits for each RGB component
; PXF24RGB = 0x00021808   ; 24 bpp; 8 bits for each RGB
; PXF32RGB = 0x00022009   ; 32 bpp; 8 bits for each RGB, no alpha.
; PXF32ARGB = 0x0026200A  ; 32 bpp; 8 bits for each RGB and alpha
; PXF32PARGB = 0x000E200B ; 32 bpp; 8 bits for each RGB and alpha, pre-mulitiplied
; PXF48RGB = 0x0010300C   ; 48 bpp; 16 bits for each RGB
; PXF64ARGB = 0x0034400D  ; 64 bpp; 16 bits for each RGB and alpha
; PXF64PARGB = 0x001A400E ; 64 bpp; 16 bits for each RGB and alpha, pre-multiplied

; INDEXED [1-bits, 4-bits and 8-bits] pixel formats rely on color palettes.
; The color information for the pixels is stored in palettes.
; Indexed images always contain a palette - a special table of colors.
; Each pixel is an index in this table. Usually a palette contains 256
; or less entries. That's why the maximum depth of an indexed pixel is 8 bpp.
; Using palettes is a common practice when working with small color depths.

; modified by Marius Șucan

   Static PixelFormatsList := {0x30101:"1-INDEXED", 0x30402:"4-INDEXED", 0x30803:"8-INDEXED", 0x101004:"16-GRAYSCALE", 0x021005:"16-RGB555", 0x21006:"16-RGB565", 0x61007:"16-ARGB1555", 0x21808:"24-RGB", 0x22009:"32-RGB", 0x26200A:"32-ARGB", 0xE200B:"32-PARGB", 0x10300C:"48-RGB", 0x34400D:"64-ARGB", 0x1A400E:"64-PARGB"}
   PixelFormat := 0
   E := DllCall("gdiplus\GdipGetImagePixelFormat", "UPtr", pBitmap, "UPtr*", PixelFormat)
   If E
      Return -1

   If (mode=0)
      Return PixelFormat

   inHEX := Format("{1:#x}", PixelFormat)
   If (PixelFormatsList.Haskey(inHEX) && mode=2)
      result := PixelFormatsList[inHEX]
   Else
      result := inHEX
   return result
}

Gdip_GetImageType(pBitmap) {
; RETURN VALUES:
; UNKNOWN = 0
; BITMAP = 1
; METAFILE = 2
; ERROR = -1
   result := 0
   E := DllCall("gdiplus\GdipGetImageType", Ptr, pBitmap, "int*", result)
   If E
      Return -1
   Return result
}

Gdip_GetDPI(pGraphics, ByRef DpiX, ByRef DpiY) {
   DpiX := Gdip_GetDpiX(pGraphics)
   DpiY := Gdip_GetDpiY(pGraphics)
}

Gdip_GetDpiX(pGraphics) {
   dpix := 0
   DllCall("gdiplus\GdipGetDpiX", "UPtr", pGraphics, "float*", dpix)
   return Round(dpix)
}

Gdip_GetDpiY(pGraphics) {
   dpiy := 0
   DllCall("gdiplus\GdipGetDpiY", "UPtr", pGraphics, "float*", dpiy)
   return Round(dpiy)
}

Gdip_GetImageHorizontalResolution(pBitmap) {
   dpix := 0
   DllCall("gdiplus\GdipGetImageHorizontalResolution", "UPtr", pBitmap, "float*", dpix)
   return Round(dpix)
}

Gdip_GetImageVerticalResolution(pBitmap) {
   dpiy := 0
   DllCall("gdiplus\GdipGetImageVerticalResolution", "UPtr", pBitmap, "float*", dpiy)
   return Round(dpiy)
}

Gdip_BitmapSetResolution(pBitmap, dpix, dpiy) {
   return DllCall("gdiplus\GdipBitmapSetResolution", "UPtr", pBitmap, "float", dpix, "float", dpiy)
}

Gdip_BitmapGetDPIResolution(pBitmap, ByRef dpix, ByRef dpiy) {
   dpix := dpiy := 0
   If StrLen(pBitmap)<3
      Return

   dpix := Gdip_GetImageHorizontalResolution(pBitmap)
   dpiy := Gdip_GetImageVerticalResolution(pBitmap)
}

Gdip_CreateBitmapFromGraphics(pGraphics, Width, Height) {
  Ptr := "UPtr"
  PtrA := "UPtr*"
  pBitmap := 0
  DllCall("gdiplus\GdipCreateBitmapFromGraphics", "int", Width, "int", Height, Ptr, pGraphics, PtrA, pBitmap)
  Return pBitmap
}

Gdip_CreateBitmapFromFile(sFile, IconNumber:=1, IconSize:="", useICM:=0) {
   Ptr := "UPtr"
   PtrA := "UPtr*"
   pBitmap := 0
   pBitmapOld := 0
   hIcon := 0

   SplitPath sFile,,, Extension
   if RegExMatch(Extension, "^(?i:exe|dll)$")
   {
      Sizes := IconSize ? IconSize : 256 "|" 128 "|" 64 "|" 48 "|" 32 "|" 16
      BufSize := 16 + (2*A_PtrSize)

      VarSetCapacity(buf, BufSize, 0)
      For eachSize, Size in StrSplit( Sizes, "|" )
      {
         DllCall("PrivateExtractIcons", "str", sFile, "int", IconNumber-1, "int", Size, "int", Size, PtrA, hIcon, PtrA, 0, "uint", 1, "uint", 0)
         if !hIcon
            continue

         if !DllCall("GetIconInfo", Ptr, hIcon, Ptr, &buf)
         {
            DestroyIcon(hIcon)
            continue
         }

         hbmMask  := NumGet(buf, 12 + (A_PtrSize - 4))
         hbmColor := NumGet(buf, 12 + (A_PtrSize - 4) + A_PtrSize)
         if !(hbmColor && DllCall("GetObject", Ptr, hbmColor, "int", BufSize, Ptr, &buf))
         {
            DestroyIcon(hIcon)
            continue
         }
         break
      }
      if !hIcon
         return -1

      Width := NumGet(buf, 4, "int"), Height := NumGet(buf, 8, "int")
      hbm := CreateDIBSection(Width, -Height), hdc := CreateCompatibleDC(), obm := SelectObject(hdc, hbm)
      if !DllCall("DrawIconEx", Ptr, hdc, "int", 0, "int", 0, Ptr, hIcon, "uint", Width, "uint", Height, "uint", 0, Ptr, 0, "uint", 3)
      {
         DestroyIcon(hIcon)
         return -2
      }

      VarSetCapacity(dib, 104)
      DllCall("GetObject", Ptr, hbm, "int", A_PtrSize = 8 ? 104 : 84, Ptr, &dib) ; sizeof(DIBSECTION) = 76+2*(A_PtrSize=8?4:0)+2*A_PtrSize
      Stride := NumGet(dib, 12, "Int")
      Bits := NumGet(dib, 20 + (A_PtrSize = 8 ? 4 : 0)) ; padding
      pBitmapOld := Gdip_CreateBitmap(Width, Height, 0, Stride, Bits)
      pBitmap := Gdip_CreateBitmap(Width, Height)
      _G := Gdip_GraphicsFromImage(pBitmap)
      Gdip_DrawImage(_G, pBitmapOld, 0, 0, Width, Height, 0, 0, Width, Height)
      SelectObject(hdc, obm), DeleteObject(hbm), DeleteDC(hdc)
      Gdip_DeleteGraphics(_G), Gdip_DisposeImage(pBitmapOld)
      DestroyIcon(hIcon)
   } else
   {
      function2call := (useICM=1) ? "GdipCreateBitmapFromFileICM" : "GdipCreateBitmapFromFile"
      if (!A_IsUnicode)
      {
         VarSetCapacity(wFile, 1024)
         DllCall("kernel32\MultiByteToWideChar", "uint", 0, "uint", 0, Ptr, &sFile, "int", -1, Ptr, &wFile, "int", 512)
         DllCall("gdiplus\" function2call, Ptr, &wFile, PtrA, pBitmap)
      } else
         DllCall("gdiplus\" function2call, Ptr, &sFile, PtrA, pBitmap)
   }

   return pBitmap
}

Gdip_CreateARGBBitmapFromHBITMAP(hImage) {
; function by iseahound found on:
; https://www.autohotkey.com/boards/viewtopic.php?f=6&t=63345
; part of https://github.com/iseahound/Graphics/blob/master/lib/Graphics.ahk

   ; struct BITMAP - https://docs.microsoft.com/en-us/windows/desktop/api/wingdi/ns-wingdi-tagbitmap
   DllCall("GetObject"
            , "ptr", hImage
            , "int", VarSetCapacity(dib, 76+2*(A_PtrSize=8?4:0)+2*A_PtrSize)
            , "ptr", &dib) ; sizeof(DIBSECTION) = x86:84, x64:104
   width  := NumGet(dib, 4, "uint")
   height := NumGet(dib, 8, "uint")
   bpp    := NumGet(dib, 18, "ushort")

   ; Fallback to built-in method if pixels are not ARGB.
   if (bpp!=32)
      return Gdip_CreateBitmapFromHBITMAP(hImage)

   ; Create a handle to a device context and associate the hImage.
   hdc := CreateCompatibleDC()
   obm := SelectObject(hdc, hImage)

   ; Buffer the hImage with a top-down device independent bitmap via negative height.
   ; Note that a DIB is an hBitmap, pixels are formatted as pARGB, and has a pointer to the bits.
   cdc := CreateCompatibleDC(hdc)
   hbm := CreateDIBSection(width, -height, hdc, 32, pBits)
   ob2 := SelectObject(cdc, hbm)

   ; Create a new Bitmap (different from an hBitmap) which holds ARGB pixel values.
   pBitmap := Gdip_CreateBitmap(width, height)

   ; Create a Scan0 buffer pointing to pBits. The buffer has pixel format pARGB.
   CreateRect(Rect, 0, 0, width, height)
   VarSetCapacity(BitmapData, 16+2*A_PtrSize, 0)
      , NumPut(       width, BitmapData,  0,  "uint") ; Width
      , NumPut(      height, BitmapData,  4,  "uint") ; Height
      , NumPut(   4 * width, BitmapData,  8,   "int") ; Stride
      , NumPut(     0xE200B, BitmapData, 12,   "int") ; PixelFormat
      , NumPut(       pBits, BitmapData, 16,   "ptr") ; Scan0
   DllCall("gdiplus\GdipBitmapLockBits"
            ,   "ptr", pBitmap
            ,   "ptr", &Rect
            ,  "uint", 6            ; ImageLockMode.UserInputBuffer | ImageLockMode.WriteOnly
            ,   "int", 0xE200B      ; Format32bppPArgb
            ,   "ptr", &BitmapData)

   ; Ensure that our hBitmap (hImage) is top-down by copying it to a top-down bitmap.
   BitBlt(cdc, 0, 0, width, height, hdc, 0, 0)

   ; Convert the pARGB pixels copied into the device independent bitmap (hbm) to ARGB.
   DllCall("gdiplus\GdipBitmapUnlockBits", "ptr",pBitmap, "ptr",&BitmapData)

   ; Cleanup the buffer and device contexts.
   SelectObject(cdc, ob2)
   DeleteObject(hbm), DeleteDC(cdc)
   SelectObject(hdc, obm), DeleteDC(hdc)

   return pBitmap
}

Gdip_CreateBitmapFromHBITMAP(hBitmap, hPalette:=0) {
; Creates a Bitmap GDI+ object from a GDI bitmap handle.
; hPalette - Handle to a GDI palette used to define the bitmap colors
; if the hBitmap is a device-dependent bitmap [DDB].

   Ptr := "UPtr"
   pBitmap := 0
   DllCall("gdiplus\GdipCreateBitmapFromHBITMAP", Ptr, hBitmap, Ptr, hPalette, "UPtr*", pBitmap)
   return pBitmap
}

Gdip_CreateHBITMAPFromBitmap(pBitmap, Background:=0xffffffff) {
; background should be zero, to not alter alpha channel of the image
   hBitmap := 0
   DllCall("gdiplus\GdipCreateHBITMAPFromBitmap", "UPtr", pBitmap, "UPtr*", hBitmap, "int", Background)
   return hBitmap
}

Gdip_CreateARGBHBITMAPFromBitmap(ByRef pBitmap) {
  ; function by iseahound ; source: https://github.com/mmikeww/AHKv2-Gdip
  ; modified to rely on already present functions [within the library]

  ; This version is about 25% faster than Gdip_CreateHBITMAPFromBitmap().
  ; Get Bitmap width and height.

  Gdip_GetImageDimensions(pBitmap, Width, Height)

  ; Convert the source pBitmap into a hBitmap manually.
  ; struct BITMAPINFOHEADER - https://docs.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapinfoheader
  hdc := CreateCompatibleDC()
  hbm := CreateDIBSection(width, -height, hdc, 32, pBits)
  obm := SelectObject(hdc, hbm)

  ; Transfer data from source pBitmap to an hBitmap manually.
  CreateRect(Rect, 0, 0, width, height)
  VarSetCapacity(BitmapData, 16+2*A_PtrSize, 0)     ; sizeof(BitmapData) = 24, 32
    , NumPut(     width, BitmapData,  0,   "uint") ; Width
    , NumPut(    height, BitmapData,  4,   "uint") ; Height
    , NumPut( 4 * width, BitmapData,  8,    "int") ; Stride
    , NumPut(   0xE200B, BitmapData, 12,    "int") ; PixelFormat
    , NumPut(     pBits, BitmapData, 16,    "ptr") ; Scan0
  DllCall("gdiplus\GdipBitmapLockBits"
        ,    "ptr", pBitmap
        ,    "ptr", &Rect
        ,   "uint", 5            ; ImageLockMode.UserInputBuffer | ImageLockMode.ReadOnly
        ,    "int", 0xE200B      ; Format32bppPArgb
        ,    "ptr", &BitmapData) ; Contains the pointer (pBits) to the hbm.
  DllCall("gdiplus\GdipBitmapUnlockBits", "ptr", pBitmap, "ptr", &BitmapData)

  ; Cleanup the hBitmap and device contexts.
  SelectObject(hdc, obm)
  DeleteObject(hdc)
  return hbm
}

Gdip_CreateBitmapFromHICON(hIcon) {
   pBitmap := 0
   DllCall("gdiplus\GdipCreateBitmapFromHICON", "UPtr", hIcon, "UPtr*", pBitmap)
   return pBitmap
}

Gdip_CreateHICONFromBitmap(pBitmap) {
   hIcon := 0
   DllCall("gdiplus\GdipCreateHICONFromBitmap", "UPtr", pBitmap, "UPtr*", hIcon)
   return hIcon
}

Gdip_CreateBitmap(Width, Height, PixelFormat:=0, Stride:=0, Scan0:=0) {
; By default, this function creates a new 32-ARGB bitmap.
; modified by Marius Șucan

   pBitmap := 0
   If !PixelFormat
      PixelFormat := 0x26200A  ; 32-ARGB

   DllCall("gdiplus\GdipCreateBitmapFromScan0"
      , "int", Width
      , "int", Height
      , "int", Stride
      , "int", PixelFormat
      , "UPtr", Scan0
      , "UPtr*", pBitmap)
   Return pBitmap
}

Gdip_CreateBitmapFromClipboard() {
; modified by Marius Șucan

   Ptr := "UPtr"
   pid := DllCall("GetCurrentProcessId","uint")
   hwnd := WinExist("ahk_pid " . pid)
   if !DllCall("IsClipboardFormatAvailable", "uint", 8)  ; CF_DIB = 8
   {
      if DllCall("IsClipboardFormatAvailable", "uint", 2)  ; CF_BITMAP = 2
      {
         if !DllCall("OpenClipboard", Ptr, hwnd)
            return -1

         hData := DllCall("User32.dll\GetClipboardData", "UInt", 2, "UPtr")
         hBitmap := DllCall("User32.dll\CopyImage", "UPtr", hData, "UInt", 0, "Int", 0, "Int", 0, "UInt", 0x2000, "Ptr")
         DllCall("CloseClipboard")
         return hBitmap
      }
      return -2
   }

   if !DllCall("OpenClipboard", Ptr, hwnd)
      return -1

   hBitmap := DllCall("GetClipboardData", "uint", 2, Ptr)
   if !hBitmap
   {
      DllCall("CloseClipboard")
      return -3
   }

   DllCall("CloseClipboard")
   pBitmap := Gdip_CreateARGBBitmapFromHBITMAP(hBitmap)
   If hBitmap
      DeleteObject(hBitmap)

   if !pBitmap
      return -4

   return pBitmap
}

Gdip_SetBitmapToClipboard(pBitmap) {
; modified by Marius Șucan to have this function report errors

   Ptr := "UPtr"
   off1 := A_PtrSize = 8 ? 52 : 44
   off2 := A_PtrSize = 8 ? 32 : 24

   pid := DllCall("GetCurrentProcessId","uint")
   hwnd := WinExist("ahk_pid " . pid)
   r1 := DllCall("OpenClipboard", Ptr, hwnd)
   If !r1
      Return -1

   hBitmap := Gdip_CreateHBITMAPFromBitmap(pBitmap, 0)
   If !hBitmap
   {
      DllCall("CloseClipboard")
      Return -3
   }

   r2 := DllCall("EmptyClipboard")
   If !r2
   {
      DeleteObject(hBitmap)
      DllCall("CloseClipboard")
      Return -2
   }

   DllCall("GetObject", Ptr, hBitmap, "int", VarSetCapacity(oi, A_PtrSize = 8 ? 104 : 84, 0), Ptr, &oi)
   hdib := DllCall("GlobalAlloc", "uint", 2, Ptr, 40+NumGet(oi, off1, "UInt"), Ptr)
   pdib := DllCall("GlobalLock", Ptr, hdib, Ptr)
   DllCall("RtlMoveMemory", Ptr, pdib, Ptr, &oi+off2, Ptr, 40)
   DllCall("RtlMoveMemory", Ptr, pdib+40, Ptr, NumGet(oi, off2 - A_PtrSize, Ptr), Ptr, NumGet(oi, off1, "UInt"))
   DllCall("GlobalUnlock", Ptr, hdib)
   DeleteObject(hBitmap)
   r3 := DllCall("SetClipboardData", "uint", 8, Ptr, hdib) ; CF_DIB = 8
   DllCall("CloseClipboard")
   DllCall("GlobalFree", Ptr, hdib)
   E := r3 ? 0 : -4    ; 0 - success
   Return E
}

Gdip_CloneBitmapArea(pBitmap, x:="", y:="", w:=0, h:=0, PixelFormat:=0, KeepPixelFormat:=0) {
; The new pBitmap is by default in the 32-ARGB PixelFormat.
;
; If the specified coordinates exceed the boundaries of pBitmap
; the resulted pBitmap is erroneuous / defective.
   pBitmapDest := 0
   If !PixelFormat
      PixelFormat := 0x26200A    ; 32-ARGB

   If (KeepPixelFormat=1)
      PixelFormat := Gdip_GetImagePixelFormat(pBitmap, 1)

   If (y="")
      y := 0

   If (x="")
      x := 0

   If (!w && !h)
      Gdip_GetImageDimensions(pBitmap, w, h)

   E := DllCall("gdiplus\GdipCloneBitmapArea"
               , "float", x, "float", y
               , "float", w, "float", h
               , "int", PixelFormat
               , "UPtr", pBitmap
               , "UPtr*", pBitmapDest)
   return pBitmapDest
}

Gdip_CloneBitmap(pBitmap) {
; the new pBitmap will have the same PixelFormat, unchanged.

   pBitmapDest := 0
   E := DllCall("gdiplus\GdipCloneImage"
               , "UPtr", pBitmap
               , "UPtr*", pBitmapDest)
   return pBitmapDest
}

Gdip_BitmapSelectActiveFrame(pBitmap, FrameIndex) {
; Selects as the active frame the given FrameIndex
; within an animated GIF or a multi-paged TIFF.
; On succes, it returns the frames count.
; On fail, the return value is -1.

    Countu := 0
    CountFrames := 0
    Ptr := "UPtr"
    DllCall("gdiplus\GdipImageGetFrameDimensionsCount", Ptr, pBitmap, "UInt*", Countu)
    VarSetCapacity(dIDs, 16, 0)
    DllCall("gdiplus\GdipImageGetFrameDimensionsList", Ptr, pBitmap, "Uint", &dIDs, "UInt", Countu)
    DllCall("gdiplus\GdipImageGetFrameCount", Ptr, pBitmap, "Uint", &dIDs, "UInt*", CountFrames)
    If (FrameIndex>CountFrames)
       FrameIndex := CountFrames
    Else If (FrameIndex<1)
       FrameIndex := 0

    E := DllCall("gdiplus\GdipImageSelectActiveFrame", Ptr, pBitmap, Ptr, &dIDs, "uint", FrameIndex)
    If E
       Return -1
    Return CountFrames
}

Gdip_GetBitmapFramesCount(pBitmap) {
; The function returns the number of frames or pages a given pBitmap has.
; GDI+ only supports multi-frames/pages for GIFs and TIFFs.
; Function written by SBC in September 2010 and
; extracted from his «Picture Viewer» script.
; https://autohotkey.com/board/topic/58226-ahk-picture-viewer/

    Countu := 0
    CountFrames := 0
    Ptr := "UPtr"
    DllCall("gdiplus\GdipImageGetFrameDimensionsCount", Ptr, pBitmap, "UInt*", Countu)
    VarSetCapacity(dIDs, 16, 0)
    DllCall("gdiplus\GdipImageGetFrameDimensionsList", Ptr, pBitmap, "Uint", &dIDs, "UInt", Countu)
    DllCall("gdiplus\GdipImageGetFrameCount", Ptr, pBitmap, "Uint", &dIDs, "UInt*", CountFrames)
    Return CountFrames
}

Gdip_CreateCachedBitmap(pBitmap, pGraphics) {
; Creates a CachedBitmap object based on a Bitmap object and a pGraphics object. The cached bitmap takes
; the pixel data from the Bitmap object and stores it in a format that is optimized for the display device
; associated with the pGraphics object.

   pCachedBitmap := 0
   Ptr := "UPtr"
   E := DllCall("gdiplus\GdipCreateCachedBitmap", Ptr, pBitmap, Ptr, pGraphics, "Ptr*", pCachedBitmap)
   return pCachedBitmap
}

Gdip_DeleteCachedBitmap(pCachedBitmap) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipDeleteCachedBitmap", Ptr, pCachedBitmap)
}

Gdip_DrawCachedBitmap(pGraphics, pCachedBitmap, X, Y) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipDrawCachedBitmap", Ptr, pGraphics, Ptr, pCachedBitmap, "int", X, "int", Y)
}

Gdip_ImageRotateFlip(pBitmap, RotateFlipType:=1) {
; RotateFlipType options:
; RotateNoneFlipNone   = 0
; Rotate90FlipNone     = 1
; Rotate180FlipNone    = 2
; Rotate270FlipNone    = 3
; RotateNoneFlipX      = 4
; Rotate90FlipX        = 5
; Rotate180FlipX       = 6
; Rotate270FlipX       = 7
; RotateNoneFlipY      = Rotate180FlipX
; Rotate90FlipY        = Rotate270FlipX
; Rotate180FlipY       = RotateNoneFlipX
; Rotate270FlipY       = Rotate90FlipX
; RotateNoneFlipXY     = Rotate180FlipNone
; Rotate90FlipXY       = Rotate270FlipNone
; Rotate180FlipXY      = RotateNoneFlipNone
; Rotate270FlipXY      = Rotate90FlipNone

   return DllCall("gdiplus\GdipImageRotateFlip", "UPtr", pBitmap, "int", RotateFlipType)
}

Gdip_RotateBitmapAtCenter(pBitmap, Angle, pBrush:=0, InterpolationMode:=7, PixelFormat:=0) {
; the pBrush will be used to fill the background of the image
; by default, it is black.
; It returns the pointer to a new pBitmap.

    If !Angle
    {
       newBitmap := Gdip_CloneBitmap(pBitmap)
       Return newBitmap
    }

    Gdip_GetImageDimensions(pBitmap, Width, Height)
    Gdip_GetRotatedDimensions(Width, Height, Angle, RWidth, RHeight)
    Gdip_GetRotatedTranslation(Width, Height, Angle, xTranslation, yTranslation)
    If (RWidth*RHeight>536848912) || (Rwidth>32100) || (RHeight>32100)
       Return

    If (pBrush=0)
    {
       pBrush := Gdip_BrushCreateSolid("0xFF000000")
       defaultBrush := 1
    }

    PixelFormatReadable := Gdip_GetImagePixelFormat(pBitmap, 2)
    If InStr(PixelFormatReadable, "indexed")
    {
       hbm := CreateDIBSection(RWidth, RHeight,,24)
       hdc := CreateCompatibleDC()
       obm := SelectObject(hdc, hbm)
       G := Gdip_GraphicsFromHDC(hdc)
       indexedMode := 1
    } Else
    {
       newBitmap := Gdip_CreateBitmap(RWidth, RHeight, PixelFormat)
       G := Gdip_GraphicsFromImage(newBitmap)
    }

    Gdip_SetInterpolationMode(G, InterpolationMode)
    Gdip_SetSmoothingMode(G, 4)
    If StrLen(pBrush)>1
       Gdip_FillRectangle(G, pBrush, 0, 0, RWidth, RHeight)
    Gdip_TranslateWorldTransform(G, xTranslation, yTranslation)
    Gdip_RotateWorldTransform(G, Angle)
    Gdip_DrawImageRect(G, pBitmap, 0, 0, Width, Height)

    If (indexedMode=1)
    {
       newBitmap := Gdip_CreateBitmapFromHBITMAP(hbm)
       SelectObject(hdc, obm)
       DeleteObject(hbm)
       DeleteDC(hdc)
    }

    Gdip_DeleteGraphics(G)
    If (defaultBrush=1)
       Gdip_DeleteBrush(pBrush)

    Return newBitmap
}

Gdip_ResizeBitmap(pBitmap, givenW, givenH, KeepRatio, InterpolationMode:="", KeepPixelFormat:=0, checkTooLarge:=0) {
; KeepPixelFormat can receive a specific PixelFormat.
; The function returns a pointer to a new pBitmap.
; Default is 0 = 32-ARGB

    Gdip_GetImageDimensions(pBitmap, Width, Height)
    If (KeepRatio=1)
    {
       calcIMGdimensions(Width, Height, givenW, givenH, ResizedW, ResizedH)
    } Else
    {
       ResizedW := givenW
       ResizedH := givenH
    }

    If (((ResizedW*ResizedH>536848912) || (ResizedW>32100) || (ResizedH>32100)) && checkTooLarge=1)
       Return

    PixelFormatReadable := Gdip_GetImagePixelFormat(pBitmap, 2)
    If (KeepPixelFormat=1)
       PixelFormat := Gdip_GetImagePixelFormat(pBitmap, 1)
    If Strlen(KeepPixelFormat)>3
       PixelFormat := KeepPixelFormat

    If InStr(PixelFormatReadable, "indexed")
    {
       hbm := CreateDIBSection(ResizedW, ResizedH,,24)
       hdc := CreateCompatibleDC()
       obm := SelectObject(hdc, hbm)
       G := Gdip_GraphicsFromHDC(hdc, InterpolationMode, 4)
       Gdip_DrawImageRect(G, pBitmap, 0, 0, ResizedW, ResizedH)
       newBitmap := Gdip_CreateBitmapFromHBITMAP(hbm)
       If (KeepPixelFormat=1)
          Gdip_BitmapSetColorDepth(newBitmap, SubStr(PixelFormatReadable, 1, 1), 1)
       SelectObject(hdc, obm)
       DeleteObject(hbm)
       DeleteDC(hdc)
       Gdip_DeleteGraphics(G)
    } Else
    {
       newBitmap := Gdip_CreateBitmap(ResizedW, ResizedH, PixelFormat)
       G := Gdip_GraphicsFromImage(newBitmap, InterpolationMode)
       Gdip_DrawImageRect(G, pBitmap, 0, 0, ResizedW, ResizedH)
       Gdip_DeleteGraphics(G)
    }

    Return newBitmap
}

;#####################################################################################
; pPen functions
; With Gdip_SetPenBrushFill() or Gdip_CreatePenFromBrush() functions,
; pPen objects can have gradients or textures.
;#####################################################################################

Gdip_CreatePen(ARGB, w, Unit:=2) {
   pPen := 0
   E := DllCall("gdiplus\GdipCreatePen1", "UInt", ARGB, "float", w, "int", Unit, "UPtr*", pPen)
   return pPen
}

Gdip_CreatePenFromBrush(pBrush, w, Unit:=2) {
; Unit  - Unit of measurement for the pen size:
; 0 - World coordinates, a non-physical unit
; 1 - Display units
; 2 - A unit is 1 pixel [default]
; 3 - A unit is 1 point or 1/72 inch
; 4 - A unit is 1 inch
; 5 - A unit is 1/300 inch
; 6 - A unit is 1 millimeter

   pPen := 0
   E := DllCall("gdiplus\GdipCreatePen2", "UPtr", pBrush, "float", w, "int", 2, "UPtr*", pPen, "int", Unit)
   return pPen
}

Gdip_SetPenWidth(pPen, width) {
   return DllCall("gdiplus\GdipSetPenWidth", "UPtr", pPen, "float", width)
}

Gdip_GetPenWidth(pPen) {
   width := 0
   E := DllCall("gdiplus\GdipGetPenWidth", "UPtr", pPen, "float*", width)
   If E
      return -1
   return width
}

Gdip_GetPenDashStyle(pPen) {
   DashStyle := 0
   E := DllCall("gdiplus\GdipGetPenDashStyle", "UPtr", pPen, "float*", DashStyle)
   If E
      return -1
   return DashStyle
}

Gdip_SetPenColor(pPen, ARGB) {
   return DllCall("gdiplus\GdipSetPenColor", "UPtr", pPen, "UInt", ARGB)
}

Gdip_GetPenColor(pPen) {
   ARGB := 0
   E := DllCall("gdiplus\GdipGetPenColor", "UPtr", pPen, "UInt*", ARGB)
   If E
      return -1
   return Format("{1:#x}", ARGB)
}

Gdip_SetPenBrushFill(pPen, pBrush) {
   return DllCall("gdiplus\GdipSetPenBrushFill", "UPtr", pPen, "UPtr", pBrush)
}

Gdip_ResetPenTransform(pPen) {
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipResetPenTransform", Ptr, pPen)
}

Gdip_MultiplyPenTransform(pPen, hMatrix, matrixOrder:=0) {
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipMultiplyPenTransform", Ptr, pPen, Ptr, hMatrix, "int", matrixOrder)
}

Gdip_RotatePenTransform(pPen, Angle, matrixOrder:=0) {
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipRotatePenTransform", Ptr, pPen, "float", Angle, "int", matrixOrder)
}

Gdip_ScalePenTransform(pPen, ScaleX, ScaleY, matrixOrder:=0) {
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipScalePenTransform", Ptr, pPen, "float", ScaleX, "float", ScaleY, "int", matrixOrder)
}

Gdip_TranslatePenTransform(pPen, X, Y, matrixOrder:=0) {
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipTranslatePenTransform", Ptr, pPen, "float", X, "float", Y, "int", matrixOrder)
}

Gdip_SetPenTransform(pPen, pMatrix) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetPenTransform", Ptr, pPen, Ptr, pMatrix)
}

Gdip_GetPenTransform(pPen) {
   Ptr := "UPtr"
   pMatrix := 0
   DllCall("gdiplus\GdipGetPenTransform", Ptr, pPen, "UPtr*", pMatrix)
   Return pMatrix
}

Gdip_GetPenBrushFill(pPen) {
; Gets the pBrush object that is currently set for the pPen object
   Ptr := "UPtr"
   pBrush := 0
   E := DllCall("gdiplus\GdipGetPenBrushFill", Ptr, pPen, "int*", pBrush)
   Return pBrush
}

Gdip_GetPenFillType(pPen) {
; Description: Gets the type of brush fill currently set for a Pen object
; Return values:
; 0  - The pen draws with a solid color
; 1  - The pen draws with a hatch pattern that is specified by a HatchBrush object
; 2  - The pen draws with a texture that is specified by a TextureBrush object
; 3  - The pen draws with a color gradient that is specified by a PathGradientBrush object
; 4  - The pen draws with a color gradient that is specified by a LinearGradientBrush object
; -1 - The pen type is unknown
; -2 - Error

   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetPenFillType", Ptr, pPen, "int*", result)
   If E
      return -2
   Return result
}

Gdip_GetPenStartCap(pPen) {
   result := 0
   Ptr := "UPtr"
   E := DllCall("gdiplus\GdipGetPenStartCap", Ptr, pPen, "int*", result)
   If E
      return -1
   Return result
}

Gdip_GetPenEndCap(pPen) {
   result := 0
   Ptr := "UPtr"
   E := DllCall("gdiplus\GdipGetPenEndCap", Ptr, pPen, "int*", result)
   If E
      return -1
   Return result
}

Gdip_GetPenDashCaps(pPen) {
   result := 0
   Ptr := "UPtr"
   E := DllCall("gdiplus\GdipGetPenDashCap197819", Ptr, pPen, "int*", result)
   If E
      return -1
   Return result
}

Gdip_GetPenAlignment(pPen) {
   result := 0
   Ptr := "UPtr"
   E := DllCall("gdiplus\GdipGetPenMode", Ptr, pPen, "int*", result)
   If E
      return -1
   Return result
}

;#####################################################################################
; Function    - Gdip_SetPenLineCaps
; Description - Sets the cap styles for the start, end, and dashes in a line drawn with the pPen object
; Parameters
; pPen        - Pointer to a Pen object. Start and end caps do not apply to closed lines.
;             - StartCap - Line cap style for the start cap:
;                  0x00 - Line ends at the last point. The end is squared off
;                  0x01 - Square cap. The center of the square is the last point in the line. The height and width of the square are the line width.
;                  0x02 - Circular cap. The center of the circle is the last point in the line. The diameter of the circle is the line width.
;                  0x03 - Triangular cap. The base of the triangle is the last point in the line. The base of the triangle is the line width.
;                  0x10 - Line ends are not anchored.
;                  0x11 - Line ends are anchored with a square. The center of the square is the last point in the line. The height and width of the square are the line width.
;                  0x12 - Line ends are anchored with a circle. The center of the circle is at the last point in the line. The circle is wider than the line.
;                  0x13 - Line ends are anchored with a diamond (a square turned at 45 degrees). The center of the diamond is at the last point in the line. The diamond is wider than the line.
;                  0x14 - Line ends are anchored with arrowheads. The arrowhead point is located at the last point in the line. The arrowhead is wider than the line.
;                  0xff - Line ends are made from a CustomLineCap object.
;               EndCap   - Line cap style for the end cap (same values as StartCap)
;               DashCap  - Start and end caps for a dashed line:
;                  0 - A square cap that squares off both ends of each dash
;                  2 - A circular cap that rounds off both ends of each dash
;                  3 - A triangular cap that points both ends of each dash
; Return value: status enumeration

Gdip_SetPenLineCaps(pPen, StartCap, EndCap, DashCap) {
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipSetPenLineCap197819", Ptr, pPen, "int", StartCap, "int", EndCap, "int", DashCap)
}

Gdip_SetPenStartCap(pPen, LineCap) {
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipSetPenStartCap", Ptr, pPen, "int", LineCap)
}

Gdip_SetPenEndCap(pPen, LineCap) {
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipSetPenEndCap", Ptr, pPen, "int", LineCap)
}

Gdip_SetPenDashCaps(pPen, LineCap) {
; If you set the alignment of a Pen object to
; Pen Alignment Inset, you cannot use that pen
; to draw triangular dash caps.

   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipSetPenDashCap197819", Ptr, pPen, "int", LineCap)
}

Gdip_SetPenAlignment(pPen, Alignment) {
; Specifies the alignment setting of the pen relative to the line that is drawn. The default value is Center.
; If you set the alignment of a Pen object to Inset, you cannot use that pen to draw compound lines or triangular dash caps.
; Alignment options:
; 0 [Center] - Specifies that the pen is aligned on the center of the line that is drawn.
; 1 [Inset]  - Specifies, when drawing a polygon, that the pen is aligned on the inside of the edge of the polygon.

   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipSetPenMode", Ptr, pPen, "int", Alignment)
}

Gdip_GetPenCompoundCount(pPen) {
    result := 0
    E := DllCall("gdiplus\GdipGetPenCompoundCount", Ptr, pPen, "int*", result)
    If E
       Return -1
    Return result
}

Gdip_SetPenCompoundArray(pPen, inCompounds) {
; Parameters     - pPen        - Pointer to a pPen object
;                  inCompounds - A string of compound values:
;                  "value1|value2|value3" [and so on]
;                  ExampleCompounds := "0.0|0.2|0.7|1.0"
; Remarks        - The elements in the string array must be in increasing order, between 0 and not greater than 1.
;                  Suppose you want a pen to draw two parallel lines where the width of the first line is 20 percent of the pen's
;                  width, the width of the space that separates the two lines is 50 percent of the pen's width, and the width
;                  of the second line is 30 percent of the pen's width. Start by creating a pPen object and an array of compound
;                  values. For this, you can then set the compound array by passing the array with the values "0.0|0.2|0.7|1.0".
; Return status enumeration

   arrCompounds := StrSplit(inCompounds, "|")
   totalCompounds := arrCompounds.Length
   VarSetCapacity(pCompounds, 8 * totalCompounds, 0)
   Loop (totalCompounds)
      NumPut(arrCompounds[A_Index], &pCompounds, 4*(A_Index - 1), "float")

   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipSetPenCompoundArray", Ptr, pPen, Ptr, &pCompounds, "int", totalCompounds)
}

Gdip_SetPenDashStyle(pPen, DashStyle) {
; DashStyle options:
; Solid = 0
; Dash = 1
; Dot = 2
; DashDot = 3
; DashDotDot = 4
; Custom = 5
; https://technet.microsoft.com/pt-br/ms534104(v=vs.71).aspx
; function by IPhilip
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipSetPenDashStyle", Ptr, pPen, "Int", DashStyle)
}

Gdip_SetPenDashArray(pPen, Dashes) {
; Description     Sets custom dashes and spaces for the pPen object.
;
; Parameters      pPen   - Pointer to a Pen object
;                 Dashes - The string that specifies the length of the custom dashes and spaces:
;                 Format: "dL1,sL1,dL2,sL2,dL3,sL3" [... and so on]
;                   dLn - Dash N length
;                   sLn - Space N length
;                 ExampleDashesArgument := "3,6,8,4,2,1"
;
; Remarks         This function sets the dash style for the pPen object to DashStyleCustom (6).
; Return status enumeration.

   Ptr := "UPtr"
   Points := StrSplit(Dashes, ",")
   PointsCount := Points.Length
   VarSetCapacity(PointsF, 8 * PointsCount, 0)
   Loop (PointsCount)
       NumPut(Points[A_Index], &PointsF, 4*(A_Index - 1), "float")

   Return DllCall("gdiplus\GdipSetPenDashArray", Ptr, pPen, Ptr, &PointsF, "int", PointsCount)
}

Gdip_SetPenDashOffset(pPen, Offset) {
; Sets the distance from the start of the line to the start of the first space in a dashed line
; Offset - Real number that specifies the number of times to shift the spaces in a dashed line. Each shift is
; equal to the length of a space in the dashed line

    Ptr := "UPtr"
    Return DllCall("gdiplus\GdipSetPenDashOffset", Ptr, pPen, "float", Offset)
}

Gdip_GetPenDashArray(pPen) {
   iCount := Gdip_GetPenDashCount(pPen)
   If (iCount=-1)
      Return 0

   VarSetCapacity(PointsF, 8 * iCount, 0)
   Ptr := "UPtr"
   DllCall("gdiplus\GdipGetPenDashArray", Ptr, pPen, "uPtr", &PointsF, "int", iCount)

   Loop (iCount)
   {
       A := NumGet(&PointsF, 4*(A_Index-1), "float")
       printList .= A ","
   }

   Return Trim(printList, ",")
}

Gdip_GetPenCompoundArray(pPen) {
   iCount := Gdip_GetPenCompoundCount(pPen)
   VarSetCapacity(PointsF, 4 * iCount, 0)
   Ptr := "UPtr"
   DllCall("gdiplus\GdipGetPenCompoundArray", Ptr, pPen, "uPtr", &PointsF, "int", iCount)

   Loop (iCount)
   {
       A := NumGet(&PointsF, 4*(A_Index-1), "float")
       printList .= A "|"
   }

   Return Trim(printList, "|")
}

Gdip_SetPenLineJoin(pPen, LineJoin) {
; LineJoin - Line join style:
; MITER = 0 - it produces a sharp corner or a clipped corner, depending on whether the length of the miter exceeds the miter limit.
; BEVEL = 1 - it produces a diagonal corner.
; ROUND = 2 - it produces a smooth, circular arc between the lines.
; MITERCLIPPED = 3 - it produces a sharp corner or a beveled corner, depending on whether the length of the miter exceeds the miter limit.

    Ptr := "UPtr"
    Return DllCall("gdiplus\GdipSetPenLineJoin", Ptr, pPen, "int", LineJoin)
}

Gdip_SetPenMiterLimit(pPen, MiterLimit) {
; MiterLimit - Real number that specifies the miter limit of the Pen object. A real number value that is less
; than 1.0 will be replaced with 1.0,
;
; Remarks
; The miter length is the distance from the intersection of the line walls on the inside of the join to the
; intersection of the line walls outside of the join. The miter length can be large when the angle between two
; lines is small. The miter limit is the maximum allowed ratio of miter length to stroke width. The default
; value is 10.0.
; If the miter length of the join of the intersection exceeds the limit of the join, then the join will be
; beveled to keep it within the limit of the join of the intersection

    Ptr := "UPtr"
    Return DllCall("gdiplus\GdipSetPenMiterLimit", Ptr, pPen, "float", MiterLimit)
}

Gdip_SetPenUnit(pPen, Unit) {
; Sets the unit of measurement for a pPen object.
; Unit - New unit of measurement for the pen:
; 0 - World coordinates, a non-physical unit
; 1 - Display units
; 2 - A unit is 1 pixel
; 3 - A unit is 1 point or 1/72 inch
; 4 - A unit is 1 inch
; 5 - A unit is 1/300 inch
; 6 - A unit is 1 millimeter

    Ptr := "UPtr"
    Return DllCall("gdiplus\GdipSetPenUnit", Ptr, pPen, "int", Unit)
}

Gdip_GetPenDashCount(pPen) {
    result := 0
    E := DllCall("gdiplus\GdipGetPenDashCount", Ptr, pPen, "int*", result)
    If E
       Return -1
    Return result
}

Gdip_GetPenDashOffset(pPen) {
    result := 0
    E := DllCall("gdiplus\GdipGetPenDashOffset", Ptr, pPen, "float*", result)
    If E
       Return -1
    Return result
}

Gdip_GetPenLineJoin(pPen) {
    result := 0
    E := DllCall("gdiplus\GdipGetPenLineJoin", Ptr, pPen, "int*", result)
    If E
       Return -1
    Return result
}

Gdip_GetPenMiterLimit(pPen) {
    result := 0
    E := DllCall("gdiplus\GdipGetPenMiterLimit", Ptr, pPen, "float*", result)
    If E
       Return -1
    Return result
}

Gdip_GetPenUnit(pPen) {
    result := 0
    E := DllCall("gdiplus\GdipGetPenUnit", Ptr, pPen, "int*", result)
    If E
       Return -1
    Return result
}

Gdip_ClonePen(pPen) {
   newPen := 0
   E := DllCall("gdiplus\GdipClonePen", "UPtr", pPen, "UPtr*", newPen)
   Return newPen
}

;#####################################################################################
; pBrush functions [types: SolidFill, Texture, Hatch patterns, PathGradient and LinearGradient]
; pBrush objects can be used by pPen objects via Gdip_SetPenBrushFill()
;#####################################################################################

Gdip_BrushCreateSolid(ARGB:=0xff000000) {
   pBrush := 0
   E := DllCall("gdiplus\GdipCreateSolidFill", "UInt", ARGB, "UPtr*", pBrush)
   return pBrush
}

Gdip_SetSolidFillColor(pBrush, ARGB) {
   return DllCall("gdiplus\GdipSetSolidFillColor", "UPtr", pBrush, "UInt", ARGB)
}

Gdip_GetSolidFillColor(pBrush) {
   ARGB := 0
   E := DllCall("gdiplus\GdipGetSolidFillColor", "UPtr", pBrush, "UInt*", ARGB)
   If E
      return -1
   return Format("{1:#x}", ARGB)
}

Gdip_BrushCreateHatch(ARGBfront, ARGBback, HatchStyle:=0) {
; HatchStyle options:
; Horizontal = 0
; Vertical = 1
; ForwardDiagonal = 2
; BackwardDiagonal = 3
; Cross = 4
; DiagonalCross = 5
; 05Percent = 6
; 10Percent = 7
; 20Percent = 8
; 25Percent = 9
; 30Percent = 10
; 40Percent = 11
; 50Percent = 12
; 60Percent = 13
; 70Percent = 14
; 75Percent = 15
; 80Percent = 16
; 90Percent = 17
; LightDownwardDiagonal = 18
; LightUpwardDiagonal = 19
; DarkDownwardDiagonal = 20
; DarkUpwardDiagonal = 21
; WideDownwardDiagonal = 22
; WideUpwardDiagonal = 23
; LightVertical = 24
; LightHorizontal = 25
; NarrowVertical = 26
; NarrowHorizontal = 27
; DarkVertical = 28
; DarkHorizontal = 29
; DashedDownwardDiagonal = 30
; DashedUpwardDiagonal = 31
; DashedHorizontal = 32
; DashedVertical = 33
; SmallConfetti = 34
; LargeConfetti = 35
; ZigZag = 36
; Wave = 37
; DiagonalBrick = 38
; HorizontalBrick = 39
; Weave = 40
; Plaid = 41
; Divot = 42
; DottedGrid = 43
; DottedDiamond = 44
; Shingle = 45
; Trellis = 46
; Sphere = 47
; SmallGrid = 48
; SmallCheckerBoard = 49
; LargeCheckerBoard = 50
; OutlinedDiamond = 51
; SolidDiamond = 52
; Total = 53
   pBrush := 0
   E := DllCall("gdiplus\GdipCreateHatchBrush", "int", HatchStyle, "UInt", ARGBfront, "UInt", ARGBback, "UPtr*", pBrush)
   return pBrush
}

Gdip_GetHatchBackgroundColor(pHatchBrush) {
   ARGB := 0
   Ptr := "UPtr"
   E := DllCall("gdiplus\GdipGetHatchBackgroundColor", Ptr, pHatchBrush, "uint*", ARGB)
   If E 
      Return -1
   return Format("{1:#x}", ARGB)
}

Gdip_GetHatchForegroundColor(pHatchBrush) {
   ARGB := 0
   Ptr := "UPtr"
   E := DllCall("gdiplus\GdipGetHatchForegroundColor", Ptr, pHatchBrush, "uint*", ARGB)
   If E 
      Return -1
   return Format("{1:#x}", ARGB)
}

Gdip_GetHatchStyle(pHatchBrush) {
   result := 0
   Ptr := "UPtr"
   E := DllCall("gdiplus\GdipGetHatchStyle", Ptr, pHatchBrush, "int*", result)
   If E 
      Return -1
   Return result
}

;#####################################################################################

; Function:             Gdip_CreateTextureBrush
; Description:          Creates a TextureBrush object based on an image, a wrap mode and a defining rectangle.
;
; pBitmap               Pointer to an Image object
; WrapMode              Wrap mode that specifies how repeated copies of an image are used to tile an area when it is
;                       painted with the texture brush:
;                       0 - Tile - Tiling without flipping
;                       1 - TileFlipX - Tiles are flipped horizontally as you move from one tile to the next in a row
;                       2 - TileFlipY - Tiles are flipped vertically as you move from one tile to the next in a column
;                       3 - TileFlipXY - Tiles are flipped horizontally as you move along a row and flipped vertically as you move along a column
;                       4 - Clamp - No tiling takes place
; x, y                  x, y coordinates of the image portion to be used by this brush
; w, h                  Width and height of the image portion
; matrix                A color matrix to alter the colors of the given pBitmap
; ScaleX, ScaleY        x, y scaling factor for the texture
; Angle                 Rotates the texture at given angle
;
; return                If the function succeeds, the return value is nonzero
; notes                 If w and h are omitted, the entire pBitmap is used
;                       Matrix can be omitted to just draw with no alteration to the ARGB channels
;                       Matrix may be passed as a digit from 0.0 - 1.0 to change just transparency
;                       Matrix can be passed as a matrix with "|" as delimiter. 
; Function modified by Marius Șucan, to allow use of color matrix and ImageAttributes object.

Gdip_CreateTextureBrush(pBitmap, WrapMode:=1, x:=0, y:=0, w:="", h:="", matrix:="", ScaleX:="", ScaleY:="", Angle:=0, ImageAttr:=0) {
   Ptr := "UPtr"
   PtrA := "UPtr*"
   pBrush := 0

   if !(w && h)
   {
      DllCall("gdiplus\GdipCreateTexture", Ptr, pBitmap, "int", WrapMode, PtrA, pBrush)
   } else
   {
      If !ImageAttr
      {
         if !IsNumber(Matrix)
            ImageAttr := Gdip_SetImageAttributesColorMatrix(Matrix)
         else if (Matrix != 1)
            ImageAttr := Gdip_SetImageAttributesColorMatrix("1|0|0|0|0|0|1|0|0|0|0|0|1|0|0|0|0|0|" Matrix "|0|0|0|0|0|1")
      } Else usrImageAttr := 1

      If ImageAttr
      {
         DllCall("gdiplus\GdipCreateTextureIA", Ptr, pBitmap, Ptr, ImageAttr, "float", x, "float", y, "float", w, "float", h, PtrA, pBrush)
         If pBrush
            Gdip_SetTextureWrapMode(pBrush, WrapMode)
      } Else
         DllCall("gdiplus\GdipCreateTexture2", Ptr, pBitmap, "int", WrapMode, "float", x, "float", y, "float", w, "float", h, PtrA, pBrush)
   }

   if (ImageAttr && usrImageAttr!=1)
      Gdip_DisposeImageAttributes(ImageAttr)

   If (ScaleX && ScaleX && pBrush)
      Gdip_ScaleTextureTransform(pBrush, ScaleX, ScaleY)

   If (Angle && pBrush)
      Gdip_RotateTextureTransform(pBrush, Angle)

   return pBrush
}

Gdip_RotateTextureTransform(pTexBrush, Angle, MatrixOrder:=0) {
; MatrixOrder options:
; Prepend = 0; The new operation is applied before the old operation.
; Append = 1; The new operation is applied after the old operation.
; Order of matrices multiplication:.

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipRotateTextureTransform", Ptr, pTexBrush, "float", Angle, "int", MatrixOrder)
}

Gdip_ScaleTextureTransform(pTexBrush, ScaleX, ScaleY, MatrixOrder:=0) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipScaleTextureTransform", Ptr, pTexBrush, "float", ScaleX, "float", ScaleY, "int", MatrixOrder)
}

Gdip_TranslateTextureTransform(pTexBrush, X, Y, MatrixOrder:=0) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipTranslateTextureTransform", Ptr, pTexBrush, "float", X, "float", Y, "int", MatrixOrder)
}

Gdip_MultiplyTextureTransform(pTexBrush, hMatrix, matrixOrder:=0) {
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipMultiplyTextureTransform", Ptr, pTexBrush, Ptr, hMatrix, "int", matrixOrder)
}

Gdip_SetTextureTransform(pTexBrush, hMatrix) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetTextureTransform", Ptr, pTexBrush, Ptr, hMatrix)
}

Gdip_GetTextureTransform(pTexBrush) {
   hMatrix := 0
   Ptr := "UPtr"
   DllCall("gdiplus\GdipGetTextureTransform", Ptr, pTexBrush, "UPtr*", hMatrix)
   Return hMatrix
}

Gdip_ResetTextureTransform(pTexBrush) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipResetTextureTransform", Ptr, pTexBrush)
}

Gdip_SetTextureWrapMode(pTexBrush, WrapMode) {
; WrapMode options:
; 0 - Tile - Tiling without flipping
; 1 - TileFlipX - Tiles are flipped horizontally as you move from one tile to the next in a row
; 2 - TileFlipY - Tiles are flipped vertically as you move from one tile to the next in a column
; 3 - TileFlipXY - Tiles are flipped horizontally as you move along a row and flipped vertically as you move along a column
; 4 - Clamp - No tiling takes place

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetTextureWrapMode", Ptr, pTexBrush, "int", WrapMode)
}

Gdip_GetTextureWrapMode(pTexBrush) {
   result := 0
   Ptr := "UPtr"
   E := DllCall("gdiplus\GdipGetTextureWrapMode", Ptr, pTexBrush, "int*", result)
   If E
      return -1
   Return result
}

Gdip_GetTextureImage(pTexBrush) {
   Ptr := "UPtr"
   pBitmapDest := 0
   E := DllCall("gdiplus\GdipGetTextureImage", Ptr, pTexBrush
               , "UPtr*", pBitmapDest)
   Return pBitmapDest
}

;#####################################################################################
; LinearGradientBrush functions
;#####################################################################################

Gdip_CreateLineBrush(x1, y1, x2, y2, ARGB1, ARGB2, WrapMode:=1) {
   return Gdip_CreateLinearGrBrush(x1, y1, x2, y2, ARGB1, ARGB2, WrapMode)
}

Gdip_CreateLinearGrBrush(x1, y1, x2, y2, ARGB1, ARGB2, WrapMode:=1) {
; Linear gradient brush.
; WrapMode specifies how the pattern is repeated once it exceeds the defined space
; Tile [no flipping] = 0
; TileFlipX = 1
; TileFlipY = 2
; TileFlipXY = 3
; Clamp [no tiling] = 4
   Ptr := "UPtr"
   CreatePointF(PointF1, x1, y1)
   CreatePointF(PointF2, x2, y2)
   pLinearGradientBrush := 0
   DllCall("gdiplus\GdipCreateLineBrush", Ptr, &PointF1, Ptr, &PointF2, "Uint", ARGB1, "Uint", ARGB2, "int", WrapMode, "UPtr*", pLinearGradientBrush)
   return pLinearGradientBrush
}

Gdip_SetLinearGrBrushColors(pLinearGradientBrush, ARGB1, ARGB2) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetLineColors", Ptr, pLinearGradientBrush, "UInt", ARGB1, "UInt", ARGB2)
}

Gdip_GetLinearGrBrushColors(pLinearGradientBrush, ByRef ARGB1, ByRef ARGB2) {
   Ptr := "UPtr"
   VarSetCapacity(colors, 8, 0)
   E := DllCall("gdiplus\GdipGetLineColors", Ptr, pLinearGradientBrush, "Ptr", &colors)
   ARGB1 := NumGet(colors, 0, "UInt")
   ARGB2 := NumGet(colors, 4, "UInt")
   ARGB1 := Format("{1:#x}", ARGB1)
   ARGB2 := Format("{1:#x}", ARGB2)
   return E
}

Gdip_CreateLineBrushFromRect(x, y, w, h, ARGB1, ARGB2, LinearGradientMode:=1, WrapMode:=1) {
   return Gdip_CreateLinearGrBrushFromRect(x, y, w, h, ARGB1, ARGB2, LinearGradientMode, WrapMode)
}

Gdip_CreateLinearGrBrushFromRect(x, y, w, h, ARGB1, ARGB2, LinearGradientMode:=1, WrapMode:=1) {
; WrapMode options [LinearGradientMode]:
; Horizontal = 0
; Vertical = 1
; ForwardDiagonal = 2
; BackwardDiagonal = 3
   CreateRectF(RectF, x, y, w, h)
   pLinearGradientBrush := 0
   E := DllCall("gdiplus\GdipCreateLineBrushFromRect", "UPtr", &RectF, "int", ARGB1, "int", ARGB2, "int", LinearGradientMode, "int", WrapMode, "UPtr*", pLinearGradientBrush)
   return pLinearGradientBrush
}

Gdip_GetLinearGrBrushGammaCorrection(pLinearGradientBrush) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetLineGammaCorrection", Ptr, pLinearGradientBrush, "int*", result)
   If E 
      Return -1
   Return result
}

Gdip_SetLinearGrBrushGammaCorrection(pLinearGradientBrush, UseGammaCorrection) {
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipSetLineGammaCorrection", Ptr, pLinearGradientBrush, "int", UseGammaCorrection)
}

Gdip_GetLinearGrBrushRect(pLinearGradientBrush) {
  Ptr := "UPtr"
  rData := {}
  VarSetCapacity(RectF, 16, 0)
  status := DllCall("gdiplus\GdipGetLineRect", Ptr, pLinearGradientBrush, Ptr, &RectF)

  If (!status) {
        rData.x := NumGet(&RectF, 0, "float")
      , rData.y := NumGet(&RectF, 4, "float")
      , rData.w := NumGet(&RectF, 8, "float")
      , rData.h := NumGet(&RectF, 12, "float")
  } Else {
    Return status
  }

  return rData
}

Gdip_ResetLinearGrBrushTransform(pLinearGradientBrush) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipResetLineTransform", Ptr, pLinearGradientBrush)
}

Gdip_ScaleLinearGrBrushTransform(pLinearGradientBrush, ScaleX, ScaleY, matrixOrder:=0) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipScaleLineTransform", Ptr, pLinearGradientBrush, "float", ScaleX, "float", ScaleY, "int", matrixOrder)
}

Gdip_MultiplyLinearGrBrushTransform(pLinearGradientBrush, hMatrix, matrixOrder:=0) {
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipMultiplyLineTransform", Ptr, pLinearGradientBrush, Ptr, hMatrix, "int", matrixOrder)
}

Gdip_TranslateLinearGrBrushTransform(pLinearGradientBrush, X, Y, matrixOrder:=0) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipTranslateLineTransform", Ptr, pLinearGradientBrush, "float", X, "float", Y, "int", matrixOrder)
}

Gdip_RotateLinearGrBrushTransform(pLinearGradientBrush, Angle, matrixOrder:=0) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipRotateLineTransform", Ptr, pLinearGradientBrush, "float", Angle, "int", matrixOrder)
}

Gdip_SetLinearGrBrushTransform(pLinearGradientBrush, pMatrix) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetLineTransform", Ptr, pLinearGradientBrush, Ptr, pMatrix)
}

Gdip_GetLinearGrBrushTransform(pLineGradientBrush) {
   Ptr := "UPtr"
   pMatrix := 0
   DllCall("gdiplus\GdipGetLineTransform", Ptr, pLineGradientBrush, "UPtr*", pMatrix)
   Return pMatrix
}

Gdip_RotateLinearGrBrushAtCenter(pLinearGradientBrush, Angle, MatrixOrder:=1) {
; function by Marius Șucan
; based on Gdip_RotatePathAtCenter() by RazorHalo

  Rect := Gdip_GetLinearGrBrushRect(pLinearGradientBrush) ; boundaries
  cX := Rect.x + (Rect.w / 2)
  cY := Rect.y + (Rect.h / 2)
  pMatrix := Gdip_CreateMatrix()
  Gdip_TranslateMatrix(pMatrix, -cX , -cY)
  Gdip_RotateMatrix(pMatrix, Angle, MatrixOrder)
  Gdip_TranslateMatrix(pMatrix, cX, cY, MatrixOrder)
  E := Gdip_SetLinearGrBrushTransform(pLinearGradientBrush, pMatrix)
  Gdip_DeleteMatrix(pMatrix)
  Return E
}

Gdip_GetLinearGrBrushWrapMode(pLinearGradientBrush) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetLineWrapMode", Ptr, pLinearGradientBrush, "int*", result)
   If E
      return -1
   Return result
}

Gdip_SetLinearGrBrushLinearBlend(pLinearGradientBrush, nFocus, nScale) {
; https://purebasic.developpez.com/tutoriels/gdi/documentation/GdiPlus/LinearGradientBrush/html/GdipSetLineLinearBlend.html

   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipSetLineLinearBlend", Ptr, pLinearGradientBrush, "float", nFocus, "float", nScale)
}

Gdip_SetLinearGrBrushSigmaBlend(pLinearGradientBrush, nFocus, nScale) {
; https://purebasic.developpez.com/tutoriels/gdi/documentation/GdiPlus/LinearGradientBrush/html/GdipSetLineSigmaBlend.html
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipSetLineSigmaBlend", Ptr, pLinearGradientBrush, "float", nFocus, "float", nScale)
}

Gdip_SetLinearGrBrushWrapMode(pLinearGradientBrush, WrapMode) {
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipSetLineWrapMode", Ptr, pLinearGradientBrush, "int", WrapMode)
}

Gdip_GetLinearGrBrushBlendCount(pLinearGradientBrush) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetLineBlendCount", Ptr, pLinearGradientBrush, "int*", result)
   If E
      return -1
   Return result
}

Gdip_SetLinearGrBrushPresetBlend(pBrush, pA, pB, pC, pD, clr1, clr2, clr3, clr4) {
   Ptr := "UPtr"
   CreateRectF(POSITIONS, pA, pB, pC, pD)
   CreateRect(COLORS, clr1, clr2, clr3, clr4)
   E:= DllCall("gdiplus\GdipSetLinePresetBlend", Ptr, pBrush, "Ptr", &COLORS, "Ptr", &POSITIONS, "Int", 4)
   Return E
}

Gdip_CloneBrush(pBrush) {
   pBrushClone := 0
   E := DllCall("gdiplus\GdipCloneBrush", "UPtr", pBrush, "UPtr*", pBrushClone)
   return pBrushClone
}

Gdip_GetBrushType(pBrush) {
; Possible brush types [return values]:
; 0 - Solid color
; 1 - Hatch pattern fill
; 2 - Texture fill
; 3 - Path gradient
; 4 - Linear gradient
; -1 - error

   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetBrushType", Ptr, pBrush, "int*", result)
   If E
      return -1
   Return result
}

;#####################################################################################
; Delete resources
;#####################################################################################

Gdip_DeleteRegion(Region) {
   return DllCall("gdiplus\GdipDeleteRegion", "UPtr", Region)
}

Gdip_DeletePen(pPen) {
   return DllCall("gdiplus\GdipDeletePen", "UPtr", pPen)
}

Gdip_DeleteBrush(pBrush) {
   return DllCall("gdiplus\GdipDeleteBrush", "UPtr", pBrush)
}

Gdip_DisposeImage(pBitmap, noErr:=0) {
; modified by Marius Șucan to help avoid crashes 
; by disposing a non-existent pBitmap

   If (StrLen(pBitmap)<=2 && noErr=1)
      Return 0

   r := DllCall("gdiplus\GdipDisposeImage", "UPtr", pBitmap)
   If (r=2 || r=1) && (noErr=1)
      r := 0
   Return r
}

Gdip_DeleteGraphics(pGraphics) {
   return DllCall("gdiplus\GdipDeleteGraphics", "UPtr", pGraphics)
}

Gdip_DisposeImageAttributes(ImageAttr) {
   return DllCall("gdiplus\GdipDisposeImageAttributes", "UPtr", ImageAttr)
}

Gdip_DeleteFont(hFont) {
   return DllCall("gdiplus\GdipDeleteFont", "UPtr", hFont)
}

Gdip_DeleteStringFormat(hStringFormat) {
   return DllCall("gdiplus\GdipDeleteStringFormat", "UPtr", hStringFormat)
}

Gdip_DeleteFontFamily(hFontFamily) {
   return DllCall("gdiplus\GdipDeleteFontFamily", "UPtr", hFontFamily)
}

Gdip_DeletePrivateFontCollection(hFontCollection) {
   Return DllCall("gdiplus\GdipDeletePrivateFontCollection", "ptr*", hFontCollection)
}

Gdip_DeleteMatrix(hMatrix) {
   return DllCall("gdiplus\GdipDeleteMatrix", "UPtr", hMatrix)
}

;#####################################################################################
; Text functions
; Easy to use functions:
; Gdip_DrawOrientedString() - allows to draw strings or string contours/outlines, 
; or both, rotated at any angle. On success, its boundaries are returned.
; Gdip_DrawStringAlongPolygon() - allows you to draw a string along a pPath
; or multiple given coordinates.
; Gdip_TextToGraphics() - allows you to draw strings or measure their boundaries.
;#####################################################################################

Gdip_DrawOrientedString(pGraphics, String, FontName, Size, Style, X, Y, Width, Height, Angle:=0, pBrush:=0, pPen:=0, Align:=0, ScaleX:=1) {
; FontName can be a name of an already installed font or it can point to a font file
; to be loaded and used to draw the string.

; Size   - in em, in world units [font size]
; Remarks: a high value might be required; over 60, 90... to see the text.
; X, Y   - coordinates for the rectangle where the text will be drawn
; W, H   - width and heigh for the rectangle where the text will be drawn
; Angle  - the angle at which the text should be rotated

; pBrush - a pointer to a pBrush object to fill the text with
; pPen   - a pointer to a pPen object to draw the outline [contour] of the text
; Remarks: both are optional, but one at least must be given, otherwise
; the function fails, returns -3.
; For example, if you want only the contour of the text, pass only a pPen object.

; Align options:
; Near/left = 0
; Center = 1
; Far/right = 2

; Style options:
; Regular = 0
; Bold = 1
; Italic = 2
; BoldItalic = 3
; Underline = 4
; Strikeout = 8

; ScaleX - if you want to distort the text [make it wider or narrower]

; On success, the function returns an array:
; PathBounds.x , PathBounds.y , PathBounds.w , PathBounds.h

   If (!pBrush && !pPen)
      Return -3

   If RegExMatch(FontName, "^(.\:\\.)")
   {
      hFontCollection := Gdip_NewPrivateFontCollection()
      hFontFamily := Gdip_CreateFontFamilyFromFile(FontName, hFontCollection)
   } Else hFontFamily := Gdip_FontFamilyCreate(FontName)

   If !hFontFamily
      hFontFamily := Gdip_FontFamilyCreateGeneric(1)
 
   If !hFontFamily
   {
      If hFontCollection
         Gdip_DeletePrivateFontCollection(hFontCollection)
      Return -1
   }

   FormatStyle := 0x4000
   hStringFormat := Gdip_StringFormatCreate(FormatStyle)
   If !hStringFormat
      hStringFormat := Gdip_StringFormatGetGeneric(1)

   If !hStringFormat
   {
      Gdip_DeleteFontFamily(hFontFamily)
      If hFontCollection
         Gdip_DeletePrivateFontCollection(hFontCollection)
      Return -2
   }

   Gdip_SetStringFormatTrimming(hStringFormat, 3)
   Gdip_SetStringFormatAlign(hStringFormat, Align)
   pPath := Gdip_CreatePath()

   E := Gdip_AddPathString(pPath, String, hFontFamily, Style, Size, hStringFormat, X, Y, Width, Height)
   If (ScaleX>0 && ScaleX!=1)
   {
      hMatrix := Gdip_CreateMatrix()
      Gdip_ScaleMatrix(hMatrix, ScaleX, 1)
      Gdip_TransformPath(pPath, hMatrix)
      Gdip_DeleteMatrix(hMatrix)
   }
   Gdip_RotatePathAtCenter(pPath, Angle)

   If (!E && pBrush)
      E := Gdip_FillPath(pGraphics, pBrush, pPath)
   If (!E && pPen)
      E := Gdip_DrawPath(pGraphics, pPen, pPath)
   PathBounds := Gdip_GetPathWorldBounds(pPath)
   Gdip_DeleteStringFormat(hStringFormat)
   Gdip_DeleteFontFamily(hFontFamily)
   Gdip_DeletePath(pPath)
   If hFontCollection
      Gdip_DeletePrivateFontCollection(hFontCollection)
   Return E ? E : PathBounds
}

Gdip_TextToGraphics(pGraphics, Text, Options, Font:="Arial", Width:="", Height:="", Measure:=0, userBrush:=0, Unit:=0) {
; Font parameter can be a name of an already installed font or it can point to a font file
; to be loaded and used to draw the string.
;
; Set Unit to 3 [Pts] to have the texts rendered at the same size
; with the texts rendered in GUIs with -DPIscale
;
; userBrush - if a pBrush object is passed, this will be used to draw the text
; Remarks: by changing the alignment, the text will be rendered at a different X
; coordinate position; the position of the text is set relative to
; the given X position coordinate and the text width..
; See also Gdip_SetStringFormatAlign().
;
; On success, the function returns a string in the following format:
; "x|y|width|height|chars|lines"
; The first four elements represent the boundaries of the text.
; The string is returned by Gdip_MeasureString()

   Static Styles := "Regular|Bold|Italic|BoldItalic|Underline|Strikeout"
        , Alignments := "Near|Left|Centre|Center|Far|Right"

   IWidth := Width, IHeight:= Height
   pattern_opts := (A_AhkVersion < "2") ? "iO)" : "i)"
   RegExMatch(Options, pattern_opts "X([\-\d\.]+)(p*)", xpos)
   RegExMatch(Options, pattern_opts "Y([\-\d\.]+)(p*)", ypos)
   RegExMatch(Options, pattern_opts "W([\-\d\.]+)(p*)", Width)
   RegExMatch(Options, pattern_opts "H([\-\d\.]+)(p*)", Height)
   RegExMatch(Options, pattern_opts "C(?!(entre|enter))([a-f\d]+)", Colour)
   RegExMatch(Options, pattern_opts "Top|Up|Bottom|Down|vCentre|vCenter", vPos)
   RegExMatch(Options, pattern_opts "NoWrap", NoWrap)
   RegExMatch(Options, pattern_opts "R(\d)", Rendering)
   RegExMatch(Options, pattern_opts "S(\d+)(p*)", Size)

   If (width && height && !NoWrap) || (Iwidth && Iheight && !NoWrap)
      mustTrimText := Measure=1 ? 0 : 1

   if Colour && IsInteger(Colour[2]) && !Gdip_DeleteBrush(Gdip_CloneBrush(Colour[2]))
   {
      PassBrush := 1
      pBrush := Colour[2]
   }

   if !(IWidth && IHeight) && ((xpos && xpos[2]) || (ypos && ypos[2]) || (Width && Width[2]) || (Height && Height[2]) || (Size && Size[2]))
      return -1

   Style := 0
   For eachStyle, valStyle in StrSplit(Styles, "|")
   {
      if RegExMatch(Options, "\b" valStyle)
         Style |= (valStyle != "StrikeOut") ? (A_Index-1) : 8
   }

   Align := 0
   For eachAlignment, valAlignment in StrSplit(Alignments, "|")
   {
      if RegExMatch(Options, "\b" valAlignment)
         Align |= A_Index//2.1   ; 0|0|1|1|2|2
   }

   xpos := (xpos && (xpos[1] != "")) ? xpos[2] ? IWidth*(xpos[1]/100) : xpos[1] : 0
   ypos := (ypos && (ypos[1] != "")) ? ypos[2] ? IHeight*(ypos[1]/100) : ypos[1] : 0
   Width := (Width && Width[1]) ? Width[2] ? IWidth*(Width[1]/100) : Width[1] : IWidth
   Height := (Height && Height[1]) ? Height[2] ? IHeight*(Height[1]/100) : Height[1] : IHeight
   If !PassBrush
      Colour := "0x" (Colour && Colour[2] ? Colour[2] : "ff000000")
   Rendering := (Rendering && (Rendering[1] >= 0) && (Rendering[1] <= 5)) ? Rendering[1] : 4
   Size := (Size && (Size[1] > 0)) ? Size[2] ? IHeight*(Size[1]/100) : Size[1] : 12
   If RegExMatch(Font, "^(.\:\\.)")
   {
      hFontCollection := Gdip_NewPrivateFontCollection()
      hFontFamily := Gdip_CreateFontFamilyFromFile(Font, hFontCollection)
   } Else hFontFamily := Gdip_FontFamilyCreate(Font)

   If !hFontFamily
      hFontFamily := Gdip_FontFamilyCreateGeneric(1)

   hFont := Gdip_FontCreate(hFontFamily, Size, Style, Unit)
   FormatStyle := NoWrap ? 0x4000 | 0x1000 : 0x4000
   hStringFormat := Gdip_StringFormatCreate(FormatStyle)
   If !hStringFormat
      hStringFormat := Gdip_StringFormatGetGeneric(1)

   pBrush := PassBrush ? pBrush : Gdip_BrushCreateSolid(Colour)
   if !(hFontFamily && hFont && hStringFormat && pBrush && pGraphics)
   {
      E := !pGraphics ? -2 : !hFontFamily ? -3 : !hFont ? -4 : !hStringFormat ? -5 : !pBrush ? -6 : 0
      If pBrush
         Gdip_DeleteBrush(pBrush)
      If hStringFormat
         Gdip_DeleteStringFormat(hStringFormat)
      If hFont
         Gdip_DeleteFont(hFont)
      If hFontFamily
         Gdip_DeleteFontFamily(hFontFamily)
      If hFontCollection
         Gdip_DeletePrivateFontCollection(hFontCollection)
      return E
   }

   CreateRectF(RC, xpos, ypos, Width, Height)
   Gdip_SetStringFormatAlign(hStringFormat, Align)
   If (mustTrimText=1)
      Gdip_SetStringFormatTrimming(hStringFormat, 3)
   Gdip_SetTextRenderingHint(pGraphics, Rendering)
   ReturnRC := Gdip_MeasureString(pGraphics, Text, hFont, hStringFormat, RC)
   ReturnRCtest := StrSplit(ReturnRC, "|")
   testX := Floor(ReturnRCtest[1]) - 2
   If (testX>xpos) ; error correction for different text alignments
   {
      nxpos := Floor(xpos - (testX - xpos))
      CreateRectF(RC, nxpos, ypos, Width, Height)
      ReturnRC := Gdip_MeasureString(pGraphics, Text, hFont, hStringFormat, RC)
      ; MsgBox, % nxpos "--" xpos "--" ypos "`n" width "--" height "`n" ReturnRC
   }

   If vPos
   {
      ReturnRC := StrSplit(ReturnRC, "|")
      if (vPos[0] = "vCentre") || (vPos[0] = "vCenter")
         ypos += (Height-ReturnRC[4])//2
      else if (vPos[0] = "Top") || (vPos[0] = "Up")
         ypos := 0
      else if (vPos[0] = "Bottom") || (vPos[0] = "Down")
         ypos := Height-ReturnRC[4]

      CreateRectF(RC, xpos, ypos, Width, ReturnRC[4])
      ReturnRC := Gdip_MeasureString(pGraphics, Text, hFont, hStringFormat, RC)
   }

   thisBrush := userBrush ? userBrush : pBrush
   if !Measure
      _E := Gdip_DrawString(pGraphics, Text, hFont, hStringFormat, thisBrush, RC)

   if !PassBrush
      Gdip_DeleteBrush(pBrush)
   Gdip_DeleteStringFormat(hStringFormat)
   Gdip_DeleteFont(hFont)
   Gdip_DeleteFontFamily(hFontFamily)
   If hFontCollection
      Gdip_DeletePrivateFontCollection(hFontCollection)
   return _E ? _E : ReturnRC
}

Gdip_DrawString(pGraphics, sString, hFont, hStringFormat, pBrush, ByRef RectF) {
   Ptr := "UPtr"
   if (!A_IsUnicode)
   {
      nSize := DllCall("MultiByteToWideChar", "uint", 0, "uint", 0, Ptr, &sString, "int", -1, Ptr, 0, "int", 0)
      VarSetCapacity(wString, nSize*2)
      DllCall("MultiByteToWideChar", "uint", 0, "uint", 0, Ptr, &sString, "int", -1, Ptr, &wString, "int", nSize)
   }

   return DllCall("gdiplus\GdipDrawString"
               , Ptr, pGraphics
               , Ptr, A_IsUnicode ? &sString : &wString
               , "int", -1
               , Ptr, hFont
               , Ptr, &RectF
               , Ptr, hStringFormat
               , Ptr, pBrush)
}

Gdip_MeasureString(pGraphics, sString, hFont, hStringFormat, ByRef RectF) {
; The function returns a string in the following format:
; "x|y|width|height|chars|lines"
; The first four elements represent the boundaries of the text

   Ptr := "UPtr"
   VarSetCapacity(RC, 16)
   if !A_IsUnicode
   {
      nSize := DllCall("MultiByteToWideChar", "uint", 0, "uint", 0, Ptr, &sString, "int", -1, "uint", 0, "int", 0)
      VarSetCapacity(wString, nSize*2)
      DllCall("MultiByteToWideChar", "uint", 0, "uint", 0, Ptr, &sString, "int", -1, Ptr, &wString, "int", nSize)
   }

   Chars := 0
   Lines := 0
   DllCall("gdiplus\GdipMeasureString"
               , Ptr, pGraphics
               , Ptr, A_IsUnicode ? &sString : &wString
               , "int", -1
               , Ptr, hFont
               , Ptr, &RectF
               , Ptr, hStringFormat
               , Ptr, &RC
               , "uint*", Chars
               , "uint*", Lines)

   return &RC ? NumGet(RC, 0, "float") "|" NumGet(RC, 4, "float") "|" NumGet(RC, 8, "float") "|" NumGet(RC, 12, "float") "|" Chars "|" Lines : 0
}

Gdip_DrawStringAlongPolygon(pGraphics, String, FontName, FontSize, Style, pBrush, DriverPoints:=0, pPath:=0, minDist:=0, flatness:=4, hMatrix:=0, Unit:=0) {
; The function allows you to draw a text string along a polygonal line.
; Each point on the line corresponds to a letter.
; If they are too close, the letters will overlap. If they are fewer than
; the string length, the text is going to be truncated.
; If given, a pPath object will be segmented according to the precision defined by «flatness».
;
; pGraphics    - a pointer to a pGraphics object where to draw the text
; FontName       can be the name of an already installed font or it can point to a font file
;                to be loaded and used to draw the string.
; FontSize     - in em, in world units
;                a high value might be required; over 60, 90... to see the text.
; pBrush       - a pointer to a pBrush object to fill the text with
; DriverPoints - a string with X, Y coordinates where the letters
;                of the string will be drawn. Each X/Y pair corresponds to a letter.
;                "x1,y1|x2,y2|x3,y3" [...and so on]
; pPath        - A pointer to a pPath object.
;                It will be used only if DriverPoints parameter is omitted.
; If both DriverPoints and pPath are omitted, the function will return -4.
; Intermmediate points will be generated if there are more glyphs / letters than defined points.
;
; flatness - from 0.1 to 5; the precision for arcs, beziers and curves segmentation;
;            the lower the number is, the higher density of points is;
;            it applies only for given pPath objects
;
; minDist  - the minimum distance between letters; by default it is FontSize/4
;            does not apply for pPath objects; use the flatness parameter to control points density
;
; Style options:
; Regular = 0
; Bold = 1
; Italic = 2
; BoldItalic = 3
; Underline = 4
; Strikeout = 8
;
; Set Unit to 3 [Pts] to have the texts rendered at the same size
; with the texts rendered in GUIs with -DPIscale

   If (!minDist || minDist<1)
      minDist := FontSize//4 + 1

   If (pPath && !DriverPoints)
   {
      newPath := Gdip_ClonePath(pPath)
      Gdip_PathOutline(newPath, flatness)
      DriverPoints := Gdip_GetPathPoints(newPath)
      Gdip_DeletePath(newPath)
      If !DriverPoints
         Return -5
   }

   If (!pPath && !DriverPoints)
      Return -4

   If RegExMatch(FontName, "^(.\:\\.)")
   {
      hFontCollection := Gdip_NewPrivateFontCollection()
      hFontFamily := Gdip_CreateFontFamilyFromFile(FontName, hFontCollection)
   } Else hFontFamily := Gdip_FontFamilyCreate(FontName)

   If !hFontFamily
      hFontFamily := Gdip_FontFamilyCreateGeneric(1)

   If !hFontFamily
   {
      If hFontCollection
         Gdip_DeletePrivateFontCollection(hFontCollection)
      Return -1
   }

   hFont := Gdip_FontCreate(hFontFamily, FontSize, Style, Unit)
   If !hFont
   {
      If hFontCollection
         Gdip_DeletePrivateFontCollection(hFontCollection)
      Gdip_DeleteFontFamily(hFontFamily)
      Return -2
   }

   Points := StrSplit(DriverPoints, "|")
   PointsCount := Points.Length
   If (PointsCount<2)
   {
      If hFontCollection
         Gdip_DeletePrivateFontCollection(hFontCollection)
      Gdip_DeleteFont(hFont)
      Gdip_DeleteFontFamily(hFontFamily)
      Return -3
   }

   txtLen := StrLen(String)
   If (PointsCount<txtLen)
   {
      loopsMax := txtLen * 3
      newDriverPoints := DriverPoints
      Loop (loopsMax)
      { 
         newDriverPoints := GenerateIntermediatePoints(newDriverPoints, minDist, totalResult)
         If (totalResult>=txtLen)
            Break
      }
      String := SubStr(String, 1, totalResult)
   } Else newDriverPoints := DriverPoints

   E := Gdip_DrawDrivenString(pGraphics, String, hFont, pBrush, newDriverPoints, 1, hMatrix)
   Gdip_DeleteFont(hFont)
   Gdip_DeleteFontFamily(hFontFamily)
   If hFontCollection
      Gdip_DeletePrivateFontCollection(hFontCollection)
   return E   
}

GenerateIntermediatePoints(PointsList, minDist, ByRef resultPointsCount) {
; function used by Gdip_DrawFreeFormString()
   AllPoints := StrSplit(PointsList, "|")
   PointsCount := AllPoints.Length
   thizIndex := 0.5
   resultPointsCount := 0
   loopsMax := PointsCount*2
   Loop (loopsMax)
   {
        thizIndex += 0.5
        thisIndex := InStr(thizIndex, ".5") ? thizIndex : Trim(Round(thizIndex))
        thisPoint := AllPoints[thisIndex]
        theseCoords := StrSplit(thisPoint, ",")
        If (theseCoords[1]!="" && theseCoords[2]!="")
        {
           resultPointsCount++
           newPointsList .= theseCoords[1] "," theseCoords[2] "|"
        } Else
        {
           aIndex := Trim(Round(thizIndex - 0.5))
           bIndex := Trim(Round(thizIndex + 0.5))
           theseAcoords := StrSplit(AllPoints[aIndex], ",")
           theseBcoords := StrSplit(AllPoints[bIndex], ",")
           If (theseAcoords[1]!="" && theseAcoords[2]!="")
           && (theseBcoords[1]!="" && theseBcoords[2]!="")
           {
               newPosX := (theseAcoords[1] + theseBcoords[1])//2
               newPosY := (theseAcoords[2] + theseBcoords[2])//2
               distPosX := newPosX - theseAcoords[1]
               distPosY := newPosY - theseAcoords[2]
               If (distPosX>minDist || distPosY>minDist)
               {
                  newPointsList .= newPosX "," newPosY "|"
                  resultPointsCount++
               }
           }
        }
   }
   If !newPointsList
      Return PointsList
   Return Trim(newPointsList, "|")
}

Gdip_DrawDrivenString(pGraphics, String, hFont, pBrush, DriverPoints, Flags:=1, hMatrix:=0) {
; Parameters:
; pBrush       - pointer to a pBrush object used to draw the text into the given pGraphics
; hFont        - pointer for a Font object used to draw the given text that determines font, size and style 
; hMatrix      - pointer to a transformation matrix object that specifies the transformation matrix to apply to each value in the DriverPoints
; DriverPoints - a list of points coordinates that determines where the glyphs [letters] will be drawn
;                "x1,y1|x2,y2|x3,y3" [... and so on]
; Flags options:
; 1 - The string array contains Unicode character values. If this flag is not set, each value in $vText is
;     interpreted as an index to a font glyph that defines a character to be displayed
; 2 - The string is displayed vertically
; 4 - The glyph positions are calculated from the position of the first glyph. If this flag is not set, the
;     glyph positions are obtained from an array of coordinates ($aPoints)
; 8 - Less memory should be used for cache of antialiased glyphs. This also produces lower quality. If this
;     flag is not set, more memory is used, but the quality is higher

   txtLen := -1 ; StrLen(String)
   Ptr := "UPtr"
   iCount := CreatePointsF(PointsF, DriverPoints)
   return DllCall("gdiplus\GdipDrawDriverString", Ptr, pGraphics, "UPtr", &String, "int", txtLen, Ptr, hFont, Ptr, pBrush, Ptr, &PointsF, "int", Flags, Ptr, hMatrix)
}

Gdip_StringFormatCreate(FormatFlags:=0, LangID:=0) {
; Format options [StringFormatFlags]
; DirectionRightToLeft    = 0x00000001
; - Activates is right to left reading order. For horizontal text, characters are read from right to left. For vertical text, columns are read from right to left.
; DirectionVertical       = 0x00000002
; - Individual lines of text are drawn vertically on the display device.
; NoFitBlackBox           = 0x00000004
; - Parts of characters are allowed to overhang the string's layout rectangle.
; DisplayFormatControl    = 0x00000020
; - Unicode layout control characters are displayed with a representative character.
; NoFontFallback          = 0x00000400
; - Prevent using an alternate font  for characters that are not supported in the requested font.
; MeasureTrailingSpaces   = 0x00000800
; - The spaces at the end of each line are included in a string measurement.
; NoWrap                  = 0x00001000
; - Disable text wrapping
; LineLimit               = 0x00002000
; - Only entire lines are laid out in the layout rectangle.
; NoClip                  = 0x00004000
; - Characters overhanging the layout rectangle and text extending outside the layout rectangle are allowed to show.

   hStringFormat := 0
   E := DllCall("gdiplus\GdipCreateStringFormat", "int", FormatFlags, "int", LangID, "UPtr*", hStringFormat)
   return hStringFormat
}

Gdip_CloneStringFormat(hStringFormat) {
   Ptr := "UPtr"
   newHStringFormat := 0
   DllCall("gdiplus\GdipCloneStringFormat", Ptr, hStringFormat, "uint*", newHStringFormat)
   Return newHStringFormat
}

Gdip_StringFormatGetGeneric(whichFormat:=0) {
; Default = 0
; Typographic := 1
   hStringFormat := 0
   If (whichFormat=1)
      DllCall("gdiplus\GdipStringFormatGetGenericTypographic", "UPtr*", hStringFormat)
   Else
      DllCall("gdiplus\GdipStringFormatGetGenericDefault", "UPtr*", hStringFormat)
   Return hStringFormat
}

Gdip_SetStringFormatAlign(hStringFormat, Align) {
; Text alignments:
; 0 - [Near / Left] Alignment is towards the origin of the bounding rectangle
; 1 - [Center] Alignment is centered between origin and extent (width) of the formatting rectangle
; 2 - [Far / Right] Alignment is to the far extent (right side) of the formatting rectangle

   return DllCall("gdiplus\GdipSetStringFormatAlign", "UPtr", hStringFormat, "int", Align)
}

Gdip_GetStringFormatAlign(hStringFormat) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetStringFormatAlign", Ptr, hStringFormat, "int*", result)
   If E
      Return -1
   Return result
}

Gdip_GetStringFormatLineAlign(hStringFormat) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetStringFormatLineAlign", Ptr, hStringFormat, "int*", result)
   If E
      Return -1
   Return result
}

Gdip_GetStringFormatDigitSubstitution(hStringFormat) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetStringFormatDigitSubstitution", Ptr, hStringFormat, "ushort*", 0, "int*", result)
   If E
      Return -1
   Return result
}

Gdip_GetStringFormatHotkeyPrefix(hStringFormat) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetStringFormatHotkeyPrefix", Ptr, hStringFormat, "int*", result)
   If E
      Return -1
   Return result
}

Gdip_GetStringFormatTrimming(hStringFormat) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetStringFormatTrimming", Ptr, hStringFormat, "int*", result)
   If E
      Return -1
   Return result
}

Gdip_SetStringFormatLineAlign(hStringFormat, StringAlign) {
; The line alignment setting specifies how to align the string vertically in the layout rectangle.
; The layout rectangle is used to position the displayed string
; StringAlign  - Type of line alignment to use:
; 0 - [Left] Alignment is towards the origin of the bounding rectangle
; 1 - [Center] Alignment is centered between origin and the height of the formatting rectangle
; 2 - [Right] Alignment is to the far extent (right side) of the formatting rectangle

   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipSetStringFormatLineAlign", Ptr, hStringFormat, "int", StringAlign)
}

Gdip_SetStringFormatDigitSubstitution(hStringFormat, DigitSubstitute, LangID:=0) {
; Sets the language ID and the digit substitution method that is used by a StringFormat object
; DigitSubstitute - Digit substitution method that will be used by the StringFormat object:
; 0 - A user-defined substitution scheme
; 1 - Digit substitution is disabled
; 2 - Substitution digits that correspond with the official national language of the user's locale
; 3 - Substitution digits that correspond with the user's native script or language

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetStringFormatDigitSubstitution", Ptr, hStringFormat, "ushort", LangID, "int", DigitSubstitute)
}

Gdip_SetStringFormatFlags(hStringFormat, Flags) {
; see Gdip_StringFormatCreate() for possible StringFormatFlags
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetStringFormatFlags", Ptr, hStringFormat, "int", Flags)
}

Gdip_SetStringFormatHotkeyPrefix(hStringFormat, PrefixProcessMode) {
; Sets the type of processing that is performed on a string when a hot key prefix (&) is encountered
; PrefixProcessMode - Type of hot key prefix processing to use:
; 0 - No hot key processing occurs.
; 1 - Unicode text is scanned for ampersands (&). All pairs of ampersands are replaced by a single ampersand.
;     All single ampersands are removed, the first character that follows a single ampersand is displayed underlined.
; 2 - Same as 1 but a character following a single ampersand is not displayed underlined.

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetStringFormatHotkeyPrefix", Ptr, hStringFormat, "int", PrefixProcessMode)
}

Gdip_SetStringFormatTrimming(hStringFormat, TrimMode) {
; TrimMode - The trimming style  to use:
; 0 - No trimming is done
; 1 - String is broken at the boundary of the last character that is inside the layout rectangle
; 2 - String is broken at the boundary of the last word that is inside the layout rectangle
; 3 - String is broken at the boundary of the last character that is inside the layout rectangle and an ellipsis (...) is inserted after the character
; 4 - String is broken at the boundary of the last word that is inside the layout rectangle and an ellipsis (...) is inserted after the word
; 5 - The center is removed from the string and replaced by an ellipsis. The algorithm keeps as much of the last portion of the string as possible

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetStringFormatTrimming", Ptr, hStringFormat, "int", TrimMode)
}

Gdip_FontCreate(hFontFamily, Size, Style:=0, Unit:=0) {
; Font style options:
; Regular = 0
; Bold = 1
; Italic = 2
; BoldItalic = 3
; Underline = 4
; Strikeout = 8
; Unit options: see Gdip_SetPageUnit()
   hFont := 0
   DllCall("gdiplus\GdipCreateFont", "UPtr", hFontFamily, "float", Size, "int", Style, "int", Unit, "UPtr*", hFont)
   return hFont
}

Gdip_FontFamilyCreate(FontName) {
   Ptr := "UPtr"
   if (!A_IsUnicode)
   {
      nSize := DllCall("MultiByteToWideChar", "uint", 0, "uint", 0, Ptr, &FontName, "int", -1, "uint", 0, "int", 0)
      VarSetCapacity(wFontName, nSize*2)
      DllCall("MultiByteToWideChar", "uint", 0, "uint", 0, Ptr, &FontName, "int", -1, Ptr, &wFontName, "int", nSize)
   }
   hFontFamily := 0
   _E := DllCall("gdiplus\GdipCreateFontFamilyFromName"
               , Ptr, A_IsUnicode ? &FontName : &wFontName
               , "uint", 0
               , "UPtr*", hFontFamily)

   return hFontFamily
}

Gdip_NewPrivateFontCollection() {
   hFontCollection := 0
   DllCall("gdiplus\GdipNewPrivateFontCollection", "ptr*", hFontCollection)
   Return hFontCollection
}

Gdip_CreateFontFamilyFromFile(FontFile, hFontCollection, FontName:="") {
; hFontCollection - the collection to add the font to
; Pass the result of Gdip_NewPrivateFontCollection() to this parameter
; to create a private collection of fonts.
; After no longer needing the private fonts, use Gdip_DeletePrivateFontCollection()
; to free up resources.
;
; GDI+ does not support PostScript fonts or OpenType fonts which do not have TrueType outlines.
;
; function by tmplinshi
; source: https://www.autohotkey.com/boards/viewtopic.php?f=6&t=813&p=298435#p297794
; modified by Marius Șucan
   If !hFontCollection
      Return

   hFontFamily := 0
   E := DllCall("gdiplus\GdipPrivateAddFontFile", "ptr", hFontCollection, "str", FontFile)
   if (FontName="" && !E)
   {
      VarSetCapacity(pFontFamily, 10, 0)
      DllCall("gdiplus\GdipGetFontCollectionFamilyList", "ptr", hFontCollection, "int", 1, "ptr", &pFontFamily, "int*", found)

      VarSetCapacity(FontName, 100)
      DllCall("gdiplus\GdipGetFamilyName", "ptr", NumGet(pFontFamily, 0, "ptr"), "str", FontName, "ushort", 1033)
   }

   If !E
      DllCall("gdiplus\GdipCreateFontFamilyFromName", "str", FontName, "ptr", hFontCollection, "uint*", hFontFamily)
   Return hFontFamily
}

Gdip_FontFamilyCreateGeneric(whichStyle) {
; This function returns a hFontFamily font object that uses a generic font.
;
; whichStyle options:
; 0 - monospace generic font 
; 1 - sans-serif generic font 
; 2 - serif generic font 

   hFontFamily := 0
   If (whichStyle=0)
      DllCall("gdiplus\GdipGetGenericFontFamilyMonospace", "UPtr*", hFontFamily)
   Else If (whichStyle=1)
      DllCall("gdiplus\GdipGetGenericFontFamilySansSerif", "UPtr*", hFontFamily)
   Else If (whichStyle=2)
      DllCall("gdiplus\GdipGetGenericFontFamilySerif", "UPtr*", hFontFamily)
   Return hFontFamily
}

Gdip_CreateFontFromDC(hDC) {
; a font must be selected in the hDC for this function to work
; function extracted from a class based wrapper around the GDI+ API made by nnnik

   pFont := 0
   r := DllCall("gdiplus\GdipCreateFontFromDC", "UPtr", hDC, "UPtr*", pFont)
   Return pFont
}

Gdip_CreateFontFromLogfontW(hDC, LogFont) {
; extracted from: https://github.com/flipeador/Library-AutoHotkey/tree/master/graphics
; by flipeador
;
; Creates a Font object directly from a GDI logical font.
; The GDI logical font is a LOGFONTW structure, which is the wide character version of a logical font.
; Parameters:
;     hDC:
;         A handle to a Windows device context that has a font selected.
;     LogFont:
;         A LOGFONTW structure that contains attributes of the font.
;         The LOGFONTW structure is the wide character version of the logical font.
;
; https://docs.microsoft.com/en-us/windows/win32/api/gdiplusheaders/nf-gdiplusheaders-font-font(inhdc_inconstlogfontw)

     pFont := 0
     DllCall("Gdiplus\GdipCreateFontFromLogfontW", "Ptr", hDC, "Ptr", LogFont, "UPtrP", pFont)
     return pFont
}

Gdip_GetFontHeight(hFont, pGraphics:=0) {
; Gets the line spacing of a font in the current unit of a specified pGraphics object.
; The line spacing is the vertical distance between the base lines of two consecutive lines of text.
; Therefore, the line spacing includes the blank space between lines along with the height of 
; the character itself.

   Ptr := "UPtr"
   result := 0
   DllCall("gdiplus\GdipGetFontHeight", Ptr, hFont, Ptr, pGraphics, "float*", result)
   Return result
}

Gdip_GetFontHeightGivenDPI(hFont, DPI:=72) {
; Remarks: it seems to always yield the same value 
; regardless of the given DPI.

   Ptr := "UPtr"
   result := 0
   DllCall("gdiplus\GdipGetFontHeightGivenDPI", Ptr, hFont, "float", DPI, "float*", result)
   Return result
}

Gdip_GetFontSize(hFont) {
   Ptr := "UPtr"
   result := 0
   DllCall("gdiplus\GdipGetFontSize", Ptr, hFont, "float*", result)
   Return result
}

Gdip_GetFontStyle(hFont) {
; see also Gdip_FontCreate()
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetFontStyle", Ptr, hFont, "int*", result)
   If E
      Return -1
   Return result
}

Gdip_GetFontUnit(hFont) {
; Gets the unit of measure of a Font object.
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetFontUnit", Ptr, hFont, "int*", result)
   If E
      Return -1
   Return result
}

Gdip_CloneFont(hfont) {
   Ptr := "UPtr"
   newHFont := 0
   DllCall("gdiplus\GdipCloneFont", Ptr, hFont, "UPtr*", newHFont)
   Return newHFont
}

Gdip_GetFontFamily(hFont) {
; On success returns a handle to a hFontFamily object
   Ptr := "UPtr"
   hFontFamily := 0
   DllCall("gdiplus\GdipGetFamily", Ptr, hFont, "UPtr*", hFontFamily)
   Return hFontFamily
}


Gdip_CloneFontFamily(hFontFamily) {
   Ptr := "UPtr"
   newHFontFamily := 0
   DllCall("gdiplus\GdipCloneFontFamily", Ptr, hFontFamily, "UPtr*", newHFontFamily)
   Return newHFontFamily
}

Gdip_IsFontStyleAvailable(hFontFamily, Style) {
; Remarks: given a proper hFontFamily object, it seems to be always 
; returning 1 [true] regardless of Style...

   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipIsStyleAvailable", Ptr, hFontFamily, "int", Style, "Int*", result)
   If E
      Return -1
   Return result
}

Gdip_GetFontFamilyCellScents(hFontFamily, ByRef Ascent, ByRef Descent, Style:=0) {
; Ascent and Descent values are given in «design units»

   Ptr := "UPtr"
   Ascent := 0
   Descent := 0
   E := DllCall("gdiplus\GdipGetCellAscent", Ptr, hFontFamily, "int", Style, "ushort*", Ascent)
   E := DllCall("gdiplus\GdipGetCellDescent", Ptr, hFontFamily, "int", Style, "ushort*", Descent)
   Return E
}

Gdip_GetFontFamilyEmHeight(hFontFamily, Style:=0) {
; EmHeight returned in «design units»
   Ptr := "UPtr"
   result := 0
   DllCall("gdiplus\GdipGetEmHeight", Ptr, hFontFamily, "int", Style, "ushort*", result)
   Return result
}

Gdip_GetFontFamilyLineSpacing(hFontFamily, Style:=0) {
; Line spacing returned in «design units»
   Ptr := "UPtr"
   result := 0
   DllCall("gdiplus\GdipGetLineSpacing", Ptr, hFontFamily, "int", Style, "ushort*", result)
   Return result
}

Gdip_GetFontFamilyName(hFontFamily) {
   Ptr := "UPtr"
   VarSetCapacity(FontName, 90)
   DllCall("gdiplus\GdipGetFamilyName", Ptr, hFontFamily, "Ptr", &FontName, "ushort", 0)
   Return FontName
}


;#####################################################################################
; Matrix functions
;#####################################################################################

Gdip_CreateAffineMatrix(m11, m12, m21, m22, x, y) {
   hMatrix := 0
   DllCall("gdiplus\GdipCreateMatrix2", "float", m11, "float", m12, "float", m21, "float", m22, "float", x, "float", y, "UPtr*", hMatrix)
   return hMatrix
}

Gdip_CreateMatrix() {
   hMatrix := 0
   DllCall("gdiplus\GdipCreateMatrix", "UPtr*", hMatrix)
   return hMatrix
}

Gdip_InvertMatrix(hMatrix) {
; Replaces the elements of a matrix with the elements of its inverse
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipInvertMatrix", Ptr, hMatrix)
}

Gdip_IsMatrixEqual(hMatrixA, hMatrixB) {
; compares two matrices; if identical, the function returns 1
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipIsMatrixEqual", Ptr, hMatrixA, Ptr, hMatrixB, "int*", result)
   If E
      Return -1
   Return result
}

Gdip_IsMatrixIdentity(hMatrix) {
; The identity matrix represents a transformation with no scaling, translation, rotation and conversion, and
; represents a transformation that does nothing.
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipIsMatrixIdentity", Ptr, hMatrix, "int*", result)
   If E
      Return -1
   Return result
}

Gdip_IsMatrixInvertible(hMatrix) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipIsMatrixInvertible", Ptr, hMatrix, "int*", result)
   If E
      Return -1
   Return result
}

Gdip_MultiplyMatrix(hMatrixA, hMatrixB, matrixOrder) {
; Updates hMatrixA with the product of itself and hMatrixB
; matrixOrder - Order of matrices multiplication:
; 0 - The second matrix is on the left
; 1 - The second matrix is on the right

   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipMultiplyMatrix", Ptr, hMatrixA, Ptr, hMatrixB, "int", matrixOrder)
}

Gdip_CloneMatrix(hMatrix) {
   Ptr := "UPtr"
   newHMatrix := 0
   DllCall("gdiplus\GdipCloneMatrix", Ptr, hMatrix, "UPtr*", newHMatrix)
   return newHMatrix
}

;#####################################################################################
; GraphicsPath functions
; pPath objects are rendered/drawn by pGraphics object using:
; - a) Gdip_FillPath() with an associated pBrush object created with any of the following functions:
;       - Gdip_BrushCreateSolid()     - SolidFill
;       - Gdip_CreateTextureBrush()   - Texture brush derived from a pBitmap
;       - Gdip_CreateLinearGrBrush()  - LinearGradient
;       - Gdip_BrushCreateHatch()     - Hatch pattern
;       - Gdip_PathGradientCreateFromPath()
; - b) Gdip_DrawPath() with an associated pPen created with Gdip_CreatePen()
;
; A pPath object can be converted using:
; - a) Gdip_PathGradientCreateFromPath() to a PathGradient brush object
; - b) Gdip_CreateRegionPath() to a region object
;#####################################################################################

Gdip_CreatePath(BrushMode:=0) {
; Alternate = 0
; Winding = 1
   pPath := 0
   DllCall("gdiplus\GdipCreatePath", "int", BrushMode, "UPtr*", pPath)
   return pPath
}

Gdip_AddPathEllipse(pPath, x, y, w, h) {
   return DllCall("gdiplus\GdipAddPathEllipse", "UPtr", pPath, "float", x, "float", y, "float", w, "float", h)
}

Gdip_AddPathRectangle(pPath, x, y, w, h) {
   return DllCall("gdiplus\GdipAddPathRectangle", "UPtr", pPath, "float", x, "float", y, "float", w, "float", h)
}

Gdip_AddPathRoundedRectangle(pPath, x, y, w, h, r) {
; extracted from: https://github.com/tariqporter/Gdip2/blob/master/lib/Object.ahk
; and adapted by Marius Șucan
   E := 0
   r := (w <= h) ? (r < w / 2) ? r : w / 2 : (r < h / 2) ? r : h / 2
   If (E := Gdip_AddPathRectangle(pPath, x+r, y, w-(2*r), r))
      Return E
   If (E := Gdip_AddPathRectangle(pPath, x+r, y+h-r, w-(2*r), r))
      Return E
   If (E := Gdip_AddPathRectangle(pPath, x, y+r, r, h-(2*r)))
      Return E
   If (E := Gdip_AddPathRectangle(pPath, x+w-r, y+r, r, h-(2*r)))
      Return E
   If (E := Gdip_AddPathRectangle(pPath, x+r, y+r, w-(2*r), h-(2*r)))
      Return E
   If (E := Gdip_AddPathPie(pPath, x, y, 2*r, 2*r, 180, 90))
      Return E
   If (E := Gdip_AddPathPie(pPath, x+w-(2*r), y, 2*r, 2*r, 270, 90))
      Return E
   If (E := Gdip_AddPathPie(pPath, x, y+h-(2*r), 2*r, 2*r, 90, 90))
      Return E
   If (E := Gdip_AddPathPie(pPath, x+w-(2*r), y+h-(2*r), 2*r, 2*r, 0, 90))
      Return E
   Return E
}


Gdip_AddPathPolygon(pPath, Points) {
; Points: the coordinates of all the points passed as x1,y1|x2,y2|x3,y3..... [minimum three points must be given]

   Ptr := "UPtr"
   iCount := CreatePointsF(PointsF, Points)
   return DllCall("gdiplus\GdipAddPathPolygon", Ptr, pPath, Ptr, &PointsF, "int", iCount)
}

Gdip_AddPathClosedCurve(pPath, Points, Tension:="") {
; Adds a closed cardinal spline to a path.
; A cardinal spline is a curve that passes through each point in the array.
;
; Parameters:
; pPath: Pointer to the GraphicsPath
; Points: the coordinates of all the points passed as x1,y1|x2,y2|x3,y3..... [minimum three points must be given]
; Tension: Non-negative real number that controls the length of the curve and how the curve bends. A value of
; zero specifies that the spline is a sequence of straight lines. As the value increases, the curve becomes fuller.

  Ptr := "UPtr"
  iCount := CreatePointsF(PointsF, Points)
  If Tension
     return DllCall("gdiplus\GdipAddPathClosedCurve2", Ptr, pPath, Ptr, &PointsF, "int", iCount, "float", Tension)
  Else
     return DllCall("gdiplus\GdipAddPathClosedCurve", Ptr, pPath, Ptr, &PointsF, "int", iCount)
}

Gdip_AddPathCurve(pPath, Points, Tension:="") {
; Adds a cardinal spline to the current figure of a path
; A cardinal spline is a curve that passes through each point in the array.
;
; Parameters:
; pPath: Pointer to the GraphicsPath
; Points: the coordinates of all the points passed as x1,y1|x2,y2|x3,y3..... [minimum three points must be given]
; Tension: Non-negative real number that controls the length of the curve and how the curve bends. A value of
; zero specifies that the spline is a sequence of straight lines. As the value increases, the curve becomes fuller.

  Ptr := "UPtr"
  iCount := CreatePointsF(PointsF, Points)
  If Tension
     return DllCall("gdiplus\GdipAddPathCurve2", Ptr, pPath, Ptr, &PointsF, "int", iCount, "float", Tension)
  Else
     return DllCall("gdiplus\GdipAddPathCurve", Ptr, pPath, Ptr, &PointsF, "int", iCount)
}

Gdip_AddPathToPath(pPathA, pPathB, fConnect) {
; Adds a path into another path.
;
; Parameters:
; pPathA and pPathB - Pointers to GraphicsPath objects
; fConnect - Specifies whether the first figure in the added path is part of the last figure in this path:
; 1 - The first figure in the added pPathB is part of the last figure in the pPathB path.
; 0 - The first figure in the added pPathB is separated from the last figure in the pPathA path.
;
; Remarks: Even if the value of the fConnect parameter is 1, this function might not be able to make the first figure
; of the added pPathB path part of the last figure of the pPathA path. If either of those figures is closed,
; then they must remain separated figures.

  Ptr := "UPtr"
  return DllCall("gdiplus\GdipAddPathCurve2", Ptr, pPathA, Ptr, pPathB, "int", fConnect)
}

Gdip_AddPathStringSimplified(pPath, String, FontName, Size, Style, X, Y, Width, Height, Align:=0, NoWrap:=0) {
; Adds the outline of a given string with the given font name, size and style 
; to a Path object.

; Size - in em, in world units [font size]
; Remarks: a high value might be required; over 60, 90... to see the text.

; X, Y   - coordinates for the rectangle where the text will be placed
; W, H   - width and heigh for the rectangle where the text will be placed

; Align options:
; Near/left = 0
; Center = 1
; Far/right = 2

; Style options:
; Regular = 0
; Bold = 1
; Italic = 2
; BoldItalic = 3
; Underline = 4
; Strikeout = 8

   FormatStyle := NoWrap ? 0x4000 | 0x1000 : 0x4000
   If RegExMatch(FontName, "^(.\:\\.)")
   {
      hFontCollection := Gdip_NewPrivateFontCollection()
      hFontFamily := Gdip_CreateFontFamilyFromFile(FontName, hFontCollection)
   } Else hFontFamily := Gdip_FontFamilyCreate(FontName)

   If !hFontFamily
      hFontFamily := Gdip_FontFamilyCreateGeneric(1)
 
   If !hFontFamily
   {
      If hFontCollection
         Gdip_DeletePrivateFontCollection(hFontCollection)
      Return -1
   }

   hStringFormat := Gdip_StringFormatCreate(FormatStyle)
   If !hStringFormat
      hStringFormat := Gdip_StringFormatGetGeneric(1)

   If !hStringFormat
   {
      Gdip_DeleteFontFamily(hFontFamily)
      If hFontCollection
         Gdip_DeletePrivateFontCollection(hFontCollection)
      Return -2
   }

   Gdip_SetStringFormatTrimming(hStringFormat, 3)
   Gdip_SetStringFormatAlign(hStringFormat, Align)
   E := Gdip_AddPathString(pPath, String, hFontFamily, Style, Size, hStringFormat, X, Y, Width, Height)
   Gdip_DeleteStringFormat(hStringFormat)
   Gdip_DeleteFontFamily(hFontFamily)
   If hFontCollection
      Gdip_DeletePrivateFontCollection(hFontCollection)
   Return E
}

Gdip_AddPathString(pPath, String, hFontFamily, Style, Size, hStringFormat, X, Y, W, H) {
   Ptr := "UPtr"
   CreateRectF(RectF, X, Y, W, H)
   E := DllCall("gdiplus\GdipAddPathString", Ptr, pPath, "WStr", String, "int", -1, Ptr, hFontFamily, "int", Style, "float", Size, Ptr, &RectF, Ptr, hStringFormat)
   Return E
}


Gdip_SetPathFillMode(pPath, FillMode) {
; Parameters
; pPath      - Pointer to a GraphicsPath object
; FillMode   - Path fill mode:
;              0 -  [Alternate] The areas are filled according to the even-odd parity rule
;              1 -  [Winding] The areas are filled according to the non-zero winding rule

   return DllCall("gdiplus\GdipSetPathFillMode", "UPtr", pPath, "int", FillMode)
}

Gdip_GetPathFillMode(pPath) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetPathFillMode", Ptr, pPath, "int*", result)
   If E 
      Return -1
   Return result
}

Gdip_GetPathLastPoint(pPath, ByRef X, ByRef Y) {
   Ptr := "UPtr"
   VarSetCapacity(PointF, 8, 0)
   E := DllCall("gdiplus\GdipGetPathLastPoint", Ptr, pPath, "UPtr", &PointF)
   If !E
   {
      x := NumGet(PointF, 0, "float")
      y := NumGet(PointF, 4, "float")
   }

   Return E
}

Gdip_GetPathPointsCount(pPath) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetPointCount", Ptr, pPath, "int*", result)
   If E 
      Return -1
   Return result
}

Gdip_GetPathPoints(pPath) {
   PointsCount := Gdip_GetPathPointsCount(pPath)
   If (PointsCount=-1)
      Return 0

   Ptr := "UPtr"
   VarSetCapacity(PointsF, 8 * PointsCount, 0)
   DllCall("gdiplus\GdipGetPathPoints", Ptr, pPath, Ptr, &PointsF, "intP", PointsCount)
   Loop (PointsCount)
   {
       A := NumGet(&PointsF, 8*(A_Index-1), "float")
       B := NumGet(&PointsF, (8*(A_Index-1))+4, "float")
       printList .= A "," B "|"
   }
   Return Trim(printList, "|")
}

Gdip_FlattenPath(pPath, flatness, hMatrix:=0) {
; flatness - a precision value that specifies the maximum error between the path and
; its flattened [segmented] approximation. Reducing the flatness increases the number
; of line segments in the approximation. 
;
; hMatrix - a pointer to a transformation matrix to apply.
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipFlattenPath", Ptr, pPath, Ptr, hMatrix, "float", flatness)
}

Gdip_WidenPath(pPath, pPen, hMatrix:=0, Flatness:=1) {
; Replaces this path with curves that enclose the area that is filled when this path is drawn by a specified pen.
; This method also flattens the path.

  Ptr := "UPtr"
  return DllCall("gdiplus\GdipWidenPath", Ptr, pPath, "uint", pPen, Ptr, hMatrix, "float", Flatness)
}

Gdip_PathOutline(pPath, flatness:=1, hMatrix:=0) {
; Transforms and flattens [segmentates] a pPath object, and then converts the path's data points
; so that they represent only the outline of the given path.
;
; flatness - a precision value that specifies the maximum error between the path and
; its flattened [segmented] approximation. Reducing the flatness increases the number
; of line segments in the resulted approximation. 
;
; hMatrix - a pointer to a transformation matrix to apply.

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipWindingModeOutline", Ptr, pPath, Ptr, hMatrix, "float", flatness)
}

Gdip_ResetPath(pPath) {
; Empties a path and sets the fill mode to alternate (0)

   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipResetPath", Ptr, pPath)
}

Gdip_ReversePath(pPath) {
; Reverses the order of the points that define a path's lines and curves

   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipReversePath", Ptr, pPath)
}

Gdip_IsOutlineVisiblePathPoint(pGraphics, pPath, pPen, X, Y) {
   result := 0
   E := DllCall("gdiplus\GdipIsOutlineVisiblePathPoint", Ptr, pPath, "float", X, "float", Y, Ptr, pPen, Ptr, pGraphics, "int*", result)
   If E 
      Return -1
   Return result
}

Gdip_IsVisiblePathPoint(pPath, x, y, pGraphics) {
; Function by RazorHalo, modified by Marius Șucan
  result := 0
  Ptr := "UPtr"
  E := DllCall("gdiplus\GdipIsVisiblePathPoint", Ptr, pPath, "float", x, "float", y, Ptr, pGraphics, "UPtr*", result)
  If E
     return -1
  return result
}

Gdip_DeletePath(pPath) {
   return DllCall("gdiplus\GdipDeletePath", "UPtr", pPath)
}

;#####################################################################################
; pGraphics rendering options functions
;#####################################################################################

Gdip_SetTextRenderingHint(pGraphics, RenderingHint) {
; RenderingHint options:
; SystemDefault = 0
; SingleBitPerPixelGridFit = 1
; SingleBitPerPixel = 2
; AntiAliasGridFit = 3
; AntiAlias = 4
   return DllCall("gdiplus\GdipSetTextRenderingHint", "UPtr", pGraphics, "int", RenderingHint)
}

Gdip_SetInterpolationMode(pGraphics, InterpolationMode) {
; InterpolationMode options:
; Default = 0
; LowQuality = 1
; HighQuality = 2
; Bilinear = 3
; Bicubic = 4
; NearestNeighbor = 5
; HighQualityBilinear = 6
; HighQualityBicubic = 7
   return DllCall("gdiplus\GdipSetInterpolationMode", "UPtr", pGraphics, "int", InterpolationMode)
}

Gdip_SetSmoothingMode(pGraphics, SmoothingMode) {
; SmoothingMode options:
; Default = 0
; HighSpeed = 1
; HighQuality = 2
; None = 3
; AntiAlias = 4
; AntiAlias8x4 = 5
; AntiAlias8x8 = 6
   return DllCall("gdiplus\GdipSetSmoothingMode", "UPtr", pGraphics, "int", SmoothingMode)
}

Gdip_SetCompositingMode(pGraphics, CompositingMode) {
; CompositingMode_SourceOver = 0 (blended / default)
; CompositingMode_SourceCopy = 1 (overwrite)

   return DllCall("gdiplus\GdipSetCompositingMode", "UPtr", pGraphics, "int", CompositingMode)
}

Gdip_SetCompositingQuality(pGraphics, CompositionQuality) {
; CompositionQuality options:
; 0 - Gamma correction is not applied.
; 1 - Gamma correction is not applied. High speed, low quality.
; 2 - Gamma correction is applied. Composition of high quality and speed.
; 3 - Gamma correction is applied.
; 4 - Gamma correction is not applied. Linear values are used.

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetCompositingQuality", Ptr, pGraphics, "int", CompositionQuality)
} 

Gdip_SetPageScale(pGraphics, Scale) {
; Sets the scaling factor for the page transformation of a pGraphics object.
; The page transformation converts page coordinates to device coordinates.

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetPageScale", Ptr, pGraphics, "float", Scale)
}

Gdip_SetPageUnit(pGraphics, Unit) {
; Sets the unit of measurement for a pGraphics object.
; Unit of measuremnet options:
; 0 - World coordinates, a non-physical unit
; 1 - Display units
; 2 - A unit is 1 pixel
; 3 - A unit is 1 point or 1/72 inch
; 4 - A unit is 1 inch
; 5 - A unit is 1/300 inch
; 6 - A unit is 1 millimeter

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetPageUnit", Ptr, pGraphics, "int", Unit)
}

Gdip_SetPixelOffsetMode(pGraphics, PixelOffsetMode) {
; Sets the pixel offset mode of a pGraphics object.
; PixelOffsetMode options:
; HighSpeed = QualityModeLow - Default
;             0, 1, 3 - Pixel centers have integer coordinates
; ModeHalf - ModeHighQuality
;             2, 4    - Pixel centers have coordinates that are half way between integer values (i.e. 0.5, 20, 105.5, etc...)

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetPixelOffsetMode", Ptr, pGraphics, "int", PixelOffsetMode)
}

Gdip_SetRenderingOrigin(pGraphics, X, Y) {
; The rendering origin is used to set the dither origin for 8-bits-per-pixel and 16-bits-per-pixel dithering
; and is also used to set the origin for hatch brushes
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetRenderingOrigin", Ptr, pGraphics, "int", X, "int", Y)
}

Gdip_SetTextContrast(pGraphics, Contrast) {
; Contrast - A number between 0 and 12, which defines the value of contrast used for antialiasing text

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetTextContrast", Ptr, pGraphics, "uint", Contrast)
}

Gdip_RestoreGraphics(pGraphics, State) {
  ; Sets the state of this Graphics object to the state stored by a previous call to the Save method of this Graphics object.
  ; Parameters:
  ;     State:
  ;         A value returned by a previous call to the Save method that identifies a block of saved state.
  ; Return value:
  ;     Returns TRUE if successful, or FALSE otherwise. To get extended error information, check Â«Gdiplus.LastStatusÂ».
  ; https://docs.microsoft.com/en-us/windows/win32/api/gdiplusgraphics/nf-gdiplusgraphics-graphics-restore
    return DllCall("Gdiplus\GdipRestoreGraphics", "UPtr", pGraphics, "UInt", State)
}

Gdip_SaveGraphics(pGraphics) {
  ; Saves the current state (transformations, clipping region, and quality settings) of this Graphics object.
  ; You can restore the state later by calling the Restore method.
  ; Return value:
  ;     Returns a value that identifies the saved state.
  ;     Pass this value to the Restore method when you want to restore the state.
  ; Remarks:
  ;     The identifier returned by a given call to the Save method can be passed only once to the Restore method.
 ; https://docs.microsoft.com/en-us/windows/win32/api/gdiplusgraphics/nf-gdiplusgraphics-graphics-save
     State := 0
     DllCall("Gdiplus\GdipSaveGraphics", "Ptr", pGraphics, "UIntP", State)
     return State
}

Gdip_GetTextContrast(pGraphics) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetTextContrast", Ptr, pGraphics, "uint*", result)
   If E
      return -1
   Return result
}

Gdip_GetCompositingMode(pGraphics) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetCompositingMode", Ptr, pGraphics, "int*", result)
   If E
      return -1
   Return result
}

Gdip_GetCompositingQuality(pGraphics) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetCompositingQuality", Ptr, pGraphics, "int*", result)
   If E
      return -1
   Return result
}

Gdip_GetInterpolationMode(pGraphics) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetInterpolationMode", Ptr, pGraphics, "int*", result)
   If E
      return -1
   Return result
}

Gdip_GetSmoothingMode(pGraphics) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetSmoothingMode", Ptr, pGraphics, "int*", result)
   If E
      return -1
   Return result
}

Gdip_GetPageScale(pGraphics) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetPageScale", Ptr, pGraphics, "float*", result)
   If E
      return -1
   Return result
}

Gdip_GetPageUnit(pGraphics) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetPageUnit", Ptr, pGraphics, "int*", result)
   If E
      return -1
   Return result
}

Gdip_GetPixelOffsetMode(pGraphics) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetPixelOffsetMode", Ptr, pGraphics, "int*", result)
   If E
      return -1
   Return result
}

Gdip_GetRenderingOrigin(pGraphics, ByRef X, ByRef Y) {
   Ptr := "UPtr"
   x := 0
   y := 0
   return DllCall("gdiplus\GdipGetRenderingOrigin", Ptr, pGraphics, "uint*", X, "uint*", Y)
}

Gdip_GetTextRenderingHint(pGraphics) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetTextRenderingHint", Ptr, pGraphics, "int*", result)
   If E
      return -1
   Return result
}

;#####################################################################################
; More pGraphics functions
;#####################################################################################

Gdip_RotateWorldTransform(pGraphics, Angle, MatrixOrder:=0) {
; MatrixOrder options:
; Prepend = 0; The new operation is applied before the old operation.
; Append = 1; The new operation is applied after the old operation.
; Order of matrices multiplication:.

   return DllCall("gdiplus\GdipRotateWorldTransform", "UPtr", pGraphics, "float", Angle, "int", MatrixOrder)
}

Gdip_ScaleWorldTransform(pGraphics, ScaleX, ScaleY, MatrixOrder:=0) {
   return DllCall("gdiplus\GdipScaleWorldTransform", "UPtr", pGraphics, "float", ScaleX, "float", ScaleY, "int", MatrixOrder)
}

Gdip_TranslateWorldTransform(pGraphics, x, y, MatrixOrder:=0) {
   return DllCall("gdiplus\GdipTranslateWorldTransform", "UPtr", pGraphics, "float", x, "float", y, "int", MatrixOrder)
}

Gdip_MultiplyWorldTransform(pGraphics, hMatrix, matrixOrder:=0) {
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipMultiplyWorldTransform", Ptr, pGraphics, Ptr, hMatrix, "int", matrixOrder)
}

Gdip_ResetWorldTransform(pGraphics) {
   return DllCall("gdiplus\GdipResetWorldTransform", "UPtr", pGraphics)
}

Gdip_ResetPageTransform(pGraphics) {
   return DllCall("gdiplus\GdipResetPageTransform", "UPtr", pGraphics)
}

Gdip_SetWorldTransform(pGraphics, hMatrix) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetWorldTransform", Ptr, pGraphics, Ptr, hMatrix)
}

Gdip_GetRotatedTranslation(Width, Height, Angle, ByRef xTranslation, ByRef yTranslation) {
   pi := 3.14159, TAngle := Angle*(pi/180)

   Bound := (Angle >= 0) ? Mod(Angle, 360) : 360-Mod(-Angle, -360)
   if ((Bound >= 0) && (Bound <= 90))
      xTranslation := Height*Sin(TAngle), yTranslation := 0
   else if ((Bound > 90) && (Bound <= 180))
      xTranslation := (Height*Sin(TAngle))-(Width*Cos(TAngle)), yTranslation := -Height*Cos(TAngle)
   else if ((Bound > 180) && (Bound <= 270))
      xTranslation := -(Width*Cos(TAngle)), yTranslation := -(Height*Cos(TAngle))-(Width*Sin(TAngle))
   else if ((Bound > 270) && (Bound <= 360))
      xTranslation := 0, yTranslation := -Width*Sin(TAngle)
}

Gdip_GetRotatedDimensions(Width, Height, Angle, ByRef RWidth, ByRef RHeight) {
; modified by Marius Șucan; removed Ceil()
   Static pi := 3.14159
   if !(Width && Height)
      return -1

   TAngle := Angle*(pi/180)
   RWidth := Abs(Width*Cos(TAngle))+Abs(Height*Sin(TAngle))
   RHeight := Abs(Width*Sin(TAngle))+Abs(Height*Cos(Tangle))
}

Gdip_GetRotatedEllipseDimensions(Width, Height, Angle, ByRef RWidth, ByRef RHeight) {
   if !(Width && Height)
      return -1

   pPath := Gdip_CreatePath()
   Gdip_AddPathEllipse(pPath, 0, 0, Width, Height)
   ; testAngle := Mod(Angle, 30)
   pMatrix := Gdip_CreateMatrix()
   Gdip_RotateMatrix(pMatrix, Angle, MatrixOrder)
   E := Gdip_TransformPath(pPath, pMatrix)
   Gdip_DeleteMatrix(pMatrix)
   pathBounds := Gdip_GetPathWorldBounds(pPath)
   Gdip_DeletePath(pPath)
   RWidth := pathBounds.w
   RHeight := pathBounds.h
   Return E
}

Gdip_GetWorldTransform(pGraphics) {
; Returns the world transformation matrix of a pGraphics object.
; On error, it returns -1
   Ptr := "UPtr"
   hMatrix := 0
   E := DllCall("gdiplus\GdipGetWorldTransform", Ptr, pGraphics, "UPtr*", hMatrix)
   Return hMatrix
}

Gdip_IsVisibleGraphPoint(pGraphics, X, Y) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipIsVisiblePoint", Ptr, pGraphics, "float", X, "float", Y, "int*", result)
   If E
      Return -1
   Return result
}

Gdip_IsVisibleGraphRect(pGraphics, X, Y, Width, Height) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipIsVisibleRect", Ptr, pGraphics, "float", X, "float", Y, "float", Width, "float", Height, "int*", result)
   If E
      Return -1
   Return result
}

Gdip_IsVisibleGraphRectEntirely(pGraphics, X, Y, Width, Height) {
    a := Gdip_IsVisibleGraphPoint(pGraphics, X, Y)
    b := Gdip_IsVisibleGraphPoint(pGraphics, X + Width, Y)
    c := Gdip_IsVisibleGraphPoint(pGraphics, X + Width, Y + Height)
    d := Gdip_IsVisibleGraphPoint(pGraphics, X, Y + Height)
    If (a=1 && b=1 && c=1 && d=1)
       Return 1
    Else If (a=-1 || b=-1 || c=-1 || d=-1)
       Return -1
    Else
       Return 0
}

;#####################################################################################
; Region and clip functions [pGraphics related]
;
; One of the properties of the pGraphics class is the clip region.
; All drawing done in a given pGraphics object can be restricted
; to the clip region of that pGraphics object. 

; The GDI+ Region class allows you to define a custom shape.
; The shape[s] can be made up of lines, polygons, and curves.
;
; Two common uses for regions are hit testing and clipping. 
; Hit testing is determining whether the mouse was clicked
; in a certain region of the screen.
;
; Clipping is restricting drawing to a certain region in
; a given pGraphics object.
;
;#####################################################################################

Gdip_IsClipEmpty(pGraphics) {
; Determines whether the clipping region of a pGraphics object is empty

   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipIsClipEmpty", Ptr, pGraphics, "int*", result)
   If E
      Return -1
   Return result
}

Gdip_IsVisibleClipEmpty(pGraphics) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipIsVisibleClipEmpty", Ptr, pGraphics, "uint*", result)
   If E
      Return -1
   Return result
}

;#####################################################################################

; Name............. Gdip_SetClipFromGraphics
;
; Parameters:
; pGraphicsA        Pointer to a pGraphics object
; pGrahpicsB        Pointer to a pGraphics object that contains the clipping region to be combined with
;                   the clipping region of the pGraphicsA object
; CombineMode       Regions combination mode:
;                   0 - The existing region is replaced by the new region
;                   1 - The existing region is replaced by the intersection of itself and the new region
;                   2 - The existing region is replaced by the union of itself and the new region
;                   3 - The existing region is replaced by the result of performing an XOR on the two regions
;                   4 - The existing region is replaced by the portion of itself that is outside of the new region
;                   5 - The existing region is replaced by the portion of the new region that is outside of the existing region
; return            Status enumeration value

Gdip_SetClipFromGraphics(pGraphics, pGraphicsSrc, CombineMode:=0) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetClipGraphics", Ptr, pGraphics, Ptr, pGraphicsSrc, "int", CombineMode)
}

Gdip_GetClipBounds(pGraphics) {
  Ptr := "UPtr"
  rData := {}

  VarSetCapacity(RectF, 16, 0)
  status := DllCall("gdiplus\GdipGetClipBounds", Ptr, pGraphics, Ptr, &RectF)

  If (!status) {
        rData.x := NumGet(&RectF, 0, "float")
      , rData.y := NumGet(&RectF, 4, "float")
      , rData.w := NumGet(&RectF, 8, "float")
      , rData.h := NumGet(&RectF, 12, "float")
  } Else {
    Return status
  }

  return rData
}

Gdip_GetVisibleClipBounds(pGraphics) {
  Ptr := "UPtr"
  rData := {}

  VarSetCapacity(RectF, 16, 0)
  status := DllCall("gdiplus\GdipGetVisibleClipBounds", Ptr, pGraphics, Ptr, &RectF)

  If (!status) {
        rData.x := NumGet(&RectF, 0, "float")
      , rData.y := NumGet(&RectF, 4, "float")
      , rData.w := NumGet(&RectF, 8, "float")
      , rData.h := NumGet(&RectF, 12, "float")
  } Else {
    Return status
  }

  return rData
}

Gdip_TranslateClip(pGraphics, dX, dY) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipTranslateClip", Ptr, pGraphics, "float", dX, "float", dY)
}

Gdip_ResetClip(pGraphics) {
   return DllCall("gdiplus\GdipResetClip", "UPtr", pGraphics)
}

Gdip_GetClipRegion(pGraphics) {
   Region := Gdip_CreateRegion()
   E := DllCall("gdiplus\GdipGetClip", "UPtr", pGraphics, "Uint", Region)
   If E
      return -1
   return Region
}

Gdip_SetClipRegion(pGraphics, Region, CombineMode:=0) {
   ; see CombineMode options from Gdip_SetClipRect()

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetClipRegion", Ptr, pGraphics, Ptr, Region, "int", CombineMode)
}

Gdip_SetClipRect(pGraphics, x, y, w, h, CombineMode:=0) {
; CombineMode options:
; Replace = 0
; Intersect = 1
; Union = 2
; Xor = 3
; Exclude = 4
; Complement = 5

   return DllCall("gdiplus\GdipSetClipRect", "UPtr", pGraphics, "float", x, "float", y, "float", w, "float", h, "int", CombineMode)
}

Gdip_SetClipPath(pGraphics, pPath, CombineMode:=0) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetClipPath", Ptr, pGraphics, Ptr, pPath, "int", CombineMode)
}

Gdip_CreateRegion() {
   Region := 0
   DllCall("gdiplus\GdipCreateRegion", "UInt*", Region)
   return Region
}

Gdip_CombineRegionRegion(Region, Region2, CombineMode) {
; Updates this region to the portion of itself that intersects another region. Added by Learning one
; see CombineMode options from Gdip_SetClipRect()

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipCombineRegionRegion", Ptr, Region, Ptr, Region2, "int", CombineMode)
}

Gdip_CombineRegionRect(Region, x, y, w, h, CombineMode) {
; Updates this region to the portion of itself that intersects with the given rectangle.
; see CombineMode options from Gdip_SetClipRect()

   Ptr := "UPtr"
   CreateRectF(RectF, x, y, w, h)
   return DllCall("gdiplus\GdipCombineRegionRect", Ptr, Region, Ptr, &RectF, "int", CombineMode)
}

Gdip_CombineRegionPath(Region, pPath, CombineMode) {
; see CombineMode options from Gdip_SetClipRect()
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipCombineRegionPath", Ptr, Region, Ptr, pPath, "int", CombineMode)
}

Gdip_CreateRegionPath(pPath) {
; Creates a region that is defined by a GraphicsPath [pPath object]. Written by Learning one.

   Ptr := "UPtr"
   Region := 0
   E := DllCall("gdiplus\GdipCreateRegionPath", Ptr, pPath, "UInt*", Region)
   If E
      return -1
   return Region
}

Gdip_CreateRegionRect(x, y, w, h) {
   CreateRectF(RectF, x, y, w, h)
   E := DllCall("gdiplus\GdipCreateRegionRect", "UPtr", &RectF, "UInt*", Region)
   If E
      return -1
   return Region
}

Gdip_IsEmptyRegion(pGraphics, Region) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipIsEmptyRegion", Ptr, Region, Ptr, pGraphics, "uInt*", result)
   If E
      return -1
   Return result
}

Gdip_IsEqualRegion(pGraphics, Region1, Region2) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipIsEqualRegion", Ptr, Region1, Ptr, Region2, Ptr, pGraphics, "uInt*", result)
   If E
      return -1
   Return result
}

Gdip_IsInfiniteRegion(pGraphics, Region) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipIsInfiniteRegion", Ptr, Region, Ptr, pGraphics, "uInt*", result)
   If E
      return -1
   Return result
}

Gdip_IsVisibleRegionPoint(pGraphics, Region, x, y) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipIsVisibleRegionPoint", Ptr, Region, "float", X, "float", Y, Ptr, pGraphics, "uInt*", result)
   If E
      return -1
   Return result
}

Gdip_IsVisibleRegionRect(pGraphics, Region, x, y, width, height) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipIsVisibleRegionRect", Ptr, Region, "float", X, "float", Y, "float", Width, "float", Height, Ptr, pGraphics, "uInt*", result)
   If E
      return -1
   Return result
}

Gdip_IsVisibleRegionRectEntirely(pGraphics, Region, x, y, width, height) {
    a := Gdip_IsVisibleRegionPoint(pGraphics, Region, X, Y)
    b := Gdip_IsVisibleRegionPoint(pGraphics, Region, X + Width, Y)
    c := Gdip_IsVisibleRegionPoint(pGraphics, Region, X + Width, Y + Height)
    d := Gdip_IsVisibleRegionPoint(pGraphics, Region, X, Y + Height)
    If (a=1 && b=1 && c=1 && d=1)
       Return 1
    Else If (a=-1 || b=-1 || c=-1 || d=-1)
       Return -1
    Else
       Return 0
}

Gdip_SetEmptyRegion(Region) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetEmpty", Ptr, Region)
}

Gdip_SetInfiniteRegion(Region) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetInfinite", Ptr, Region)
}

Gdip_GetRegionBounds(pGraphics, Region) {
  Ptr := "UPtr"
  rData := {}

  VarSetCapacity(RectF, 16, 0)
  status := DllCall("gdiplus\GdipGetRegionBounds", Ptr, Region, Ptr, pGraphics, Ptr, &RectF)

  If (!status) {
        rData.x := NumGet(&RectF, 0, "float")
      , rData.y := NumGet(&RectF, 4, "float")
      , rData.w := NumGet(&RectF, 8, "float")
      , rData.h := NumGet(&RectF, 12, "float")
  } Else {
    Return status
  }

  return rData
}

Gdip_TranslateRegion(Region, X, Y) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipTranslateRegion", Ptr, Region, "float", X, "float", Y)
}

Gdip_RotateRegionAtCenter(pGraphics, Region, Angle, MatrixOrder:=1) {
; function by Marius Șucan
; based on Gdip_RotatePathAtCenter() by RazorHalo

  Rect := Gdip_GetRegionBounds(pGraphics, Region)
  cX := Rect.x + (Rect.w / 2)
  cY := Rect.y + (Rect.h / 2)
  pMatrix := Gdip_CreateMatrix()
  Gdip_TranslateMatrix(pMatrix, -cX , -cY)
  Gdip_RotateMatrix(pMatrix, Angle, MatrixOrder)
  Gdip_TranslateMatrix(pMatrix, cX, cY, MatrixOrder)
  E := Gdip_TransformRegion(Region, pMatrix)
  Gdip_DeleteMatrix(pMatrix)
  Return E
}

Gdip_TransformRegion(Region, pMatrix) {
  Ptr := "UPtr"
  return DllCall("gdiplus\GdipTransformRegion", Ptr, Region, Ptr, pMatrix)
}

Gdip_CloneRegion(Region) {
   Ptr := "UPtr"
   newRegion := 0
   DllCall("gdiplus\GdipCloneRegion", Ptr, Region, "UInt*", newRegion)
   return newRegion
}

;#####################################################################################
; BitmapLockBits
;#####################################################################################

Gdip_LockBits(pBitmap, x, y, w, h, ByRef Stride, ByRef Scan0, ByRef BitmapData, LockMode := 3, PixelFormat := 0x26200a) {
   Ptr := "UPtr"

   CreateRect(_Rect, x, y, w, h)
   VarSetCapacity(BitmapData, 16+2*A_PtrSize, 0)
   _E := DllCall("Gdiplus\GdipBitmapLockBits", Ptr, pBitmap, Ptr, &_Rect, "uint", LockMode, "int", PixelFormat, Ptr, &BitmapData)
   Stride := NumGet(BitmapData, 8, "Int")
   Scan0 := NumGet(BitmapData, 16, Ptr)
   return _E
}

Gdip_UnlockBits(pBitmap, ByRef BitmapData) {
   Ptr := "UPtr"
   return DllCall("Gdiplus\GdipBitmapUnlockBits", Ptr, pBitmap, Ptr, &BitmapData)
}

Gdip_SetLockBitPixel(ARGB, Scan0, x, y, Stride) {
   NumPut(ARGB, Scan0+0, (x*4)+(y*Stride), "UInt")
}

Gdip_GetLockBitPixel(Scan0, x, y, Stride) {
   return NumGet(Scan0+0, (x*4)+(y*Stride), "UInt")
}

;#####################################################################################

Gdip_PixelateBitmap(pBitmap, ByRef pBitmapOut, BlockSize) {
; it does not work on x64, AHK_L Unicode, Windows 10 

   static PixelateBitmap
   Ptr := "UPtr"
   if (!PixelateBitmap)
   {
      if (A_PtrSize!=8) ; x86 machine code
      MCode_PixelateBitmap := "
      (LTrim Join
      558BEC83EC3C8B4514538B5D1C99F7FB56578BC88955EC894DD885C90F8E830200008B451099F7FB8365DC008365E000894DC88955F08945E833FF897DD4
      397DE80F8E160100008BCB0FAFCB894DCC33C08945F88945FC89451C8945143BD87E608B45088D50028BC82BCA8BF02BF2418945F48B45E02955F4894DC4
      8D0CB80FAFCB03CA895DD08BD1895DE40FB64416030145140FB60201451C8B45C40FB604100145FC8B45F40FB604020145F883C204FF4DE475D6034D18FF
      4DD075C98B4DCC8B451499F7F98945148B451C99F7F989451C8B45FC99F7F98945FC8B45F899F7F98945F885DB7E648B450C8D50028BC82BCA83C103894D
      C48BC82BCA41894DF48B4DD48945E48B45E02955E48D0C880FAFCB03CA895DD08BD18BF38A45148B7DC48804178A451C8B7DF488028A45FC8804178A45F8
      8B7DE488043A83C2044E75DA034D18FF4DD075CE8B4DCC8B7DD447897DD43B7DE80F8CF2FEFFFF837DF0000F842C01000033C08945F88945FC89451C8945
      148945E43BD87E65837DF0007E578B4DDC034DE48B75E80FAF4D180FAFF38B45088D500203CA8D0CB18BF08BF88945F48B45F02BF22BFA2955F48945CC0F
      B6440E030145140FB60101451C0FB6440F010145FC8B45F40FB604010145F883C104FF4DCC75D8FF45E4395DE47C9B8B4DF00FAFCB85C9740B8B451499F7
      F9894514EB048365140033F63BCE740B8B451C99F7F989451CEB0389751C3BCE740B8B45FC99F7F98945FCEB038975FC3BCE740B8B45F899F7F98945F8EB
      038975F88975E43BDE7E5A837DF0007E4C8B4DDC034DE48B75E80FAF4D180FAFF38B450C8D500203CA8D0CB18BF08BF82BF22BFA2BC28B55F08955CC8A55
      1488540E038A551C88118A55FC88540F018A55F888140183C104FF4DCC75DFFF45E4395DE47CA68B45180145E0015DDCFF4DC80F8594FDFFFF8B451099F7
      FB8955F08945E885C00F8E450100008B45EC0FAFC38365DC008945D48B45E88945CC33C08945F88945FC89451C8945148945103945EC7E6085DB7E518B4D
      D88B45080FAFCB034D108D50020FAF4D18034DDC8BF08BF88945F403CA2BF22BFA2955F4895DC80FB6440E030145140FB60101451C0FB6440F010145FC8B
      45F40FB604080145F883C104FF4DC875D8FF45108B45103B45EC7CA08B4DD485C9740B8B451499F7F9894514EB048365140033F63BCE740B8B451C99F7F9
      89451CEB0389751C3BCE740B8B45FC99F7F98945FCEB038975FC3BCE740B8B45F899F7F98945F8EB038975F88975103975EC7E5585DB7E468B4DD88B450C
      0FAFCB034D108D50020FAF4D18034DDC8BF08BF803CA2BF22BFA2BC2895DC88A551488540E038A551C88118A55FC88540F018A55F888140183C104FF4DC8
      75DFFF45108B45103B45EC7CAB8BC3C1E0020145DCFF4DCC0F85CEFEFFFF8B4DEC33C08945F88945FC89451C8945148945103BC87E6C3945F07E5C8B4DD8
      8B75E80FAFCB034D100FAFF30FAF4D188B45088D500203CA8D0CB18BF08BF88945F48B45F02BF22BFA2955F48945C80FB6440E030145140FB60101451C0F
      B6440F010145FC8B45F40FB604010145F883C104FF4DC875D833C0FF45108B4DEC394D107C940FAF4DF03BC874068B451499F7F933F68945143BCE740B8B
      451C99F7F989451CEB0389751C3BCE740B8B45FC99F7F98945FCEB038975FC3BCE740B8B45F899F7F98945F8EB038975F88975083975EC7E63EB0233F639
      75F07E4F8B4DD88B75E80FAFCB034D080FAFF30FAF4D188B450C8D500203CA8D0CB18BF08BF82BF22BFA2BC28B55F08955108A551488540E038A551C8811
      8A55FC88540F018A55F888140883C104FF4D1075DFFF45088B45083B45EC7C9F5F5E33C05BC9C21800
      )"
      else ; x64 machine code
      MCode_PixelateBitmap := "
      (LTrim Join
      4489442418488954241048894C24085355565741544155415641574883EC28418BC1448B8C24980000004C8BDA99488BD941F7F9448BD0448BFA8954240C
      448994248800000085C00F8E9D020000418BC04533E4458BF299448924244C8954241041F7F933C9898C24980000008BEA89542404448BE889442408EB05
      4C8B5C24784585ED0F8E1A010000458BF1418BFD48897C2418450FAFF14533D233F633ED4533E44533ED4585C97E5B4C63BC2490000000418D040A410FAF
      C148984C8D441802498BD9498BD04D8BD90FB642010FB64AFF4403E80FB60203E90FB64AFE4883C2044403E003F149FFCB75DE4D03C748FFCB75D0488B7C
      24188B8C24980000004C8B5C2478418BC59941F7FE448BE8418BC49941F7FE448BE08BC59941F7FE8BE88BC69941F7FE8BF04585C97E4048639C24900000
      004103CA4D8BC1410FAFC94863C94A8D541902488BCA498BC144886901448821408869FF408871FE4883C10448FFC875E84803D349FFC875DA8B8C249800
      0000488B5C24704C8B5C24784183C20448FFCF48897C24180F850AFFFFFF8B6C2404448B2424448B6C24084C8B74241085ED0F840A01000033FF33DB4533
      DB4533D24533C04585C97E53488B74247085ED7E42438D0C04418BC50FAF8C2490000000410FAFC18D04814863C8488D5431028BCD0FB642014403D00FB6
      024883C2044403D80FB642FB03D80FB642FA03F848FFC975DE41FFC0453BC17CB28BCD410FAFC985C9740A418BC299F7F98BF0EB0233F685C9740B418BC3
      99F7F9448BD8EB034533DB85C9740A8BC399F7F9448BD0EB034533D285C9740A8BC799F7F9448BC0EB034533C033D24585C97E4D4C8B74247885ED7E3841
      8D0C14418BC50FAF8C2490000000410FAFC18D04814863C84A8D4431028BCD40887001448818448850FF448840FE4883C00448FFC975E8FFC2413BD17CBD
      4C8B7424108B8C2498000000038C2490000000488B5C24704503E149FFCE44892424898C24980000004C897424100F859EFDFFFF448B7C240C448B842480
      000000418BC09941F7F98BE8448BEA89942498000000896C240C85C00F8E3B010000448BAC2488000000418BCF448BF5410FAFC9898C248000000033FF33
      ED33F64533DB4533D24533C04585FF7E524585C97E40418BC5410FAFC14103C00FAF84249000000003C74898488D541802498BD90FB642014403D00FB602
      4883C2044403D80FB642FB03F00FB642FA03E848FFCB75DE488B5C247041FFC0453BC77CAE85C9740B418BC299F7F9448BE0EB034533E485C9740A418BC3
      99F7F98BD8EB0233DB85C9740A8BC699F7F9448BD8EB034533DB85C9740A8BC599F7F9448BD0EB034533D24533C04585FF7E4E488B4C24784585C97E3541
      8BC5410FAFC14103C00FAF84249000000003C74898488D540802498BC144886201881A44885AFF448852FE4883C20448FFC875E941FFC0453BC77CBE8B8C
      2480000000488B5C2470418BC1C1E00203F849FFCE0F85ECFEFFFF448BAC24980000008B6C240C448BA4248800000033FF33DB4533DB4533D24533C04585
      FF7E5A488B7424704585ED7E48418BCC8BC5410FAFC94103C80FAF8C2490000000410FAFC18D04814863C8488D543102418BCD0FB642014403D00FB60248
      83C2044403D80FB642FB03D80FB642FA03F848FFC975DE41FFC0453BC77CAB418BCF410FAFCD85C9740A418BC299F7F98BF0EB0233F685C9740B418BC399
      F7F9448BD8EB034533DB85C9740A8BC399F7F9448BD0EB034533D285C9740A8BC799F7F9448BC0EB034533C033D24585FF7E4E4585ED7E42418BCC8BC541
      0FAFC903CA0FAF8C2490000000410FAFC18D04814863C8488B442478488D440102418BCD40887001448818448850FF448840FE4883C00448FFC975E8FFC2
      413BD77CB233C04883C428415F415E415D415C5F5E5D5BC3
      )"

      VarSetCapacity(PixelateBitmap, StrLen(MCode_PixelateBitmap)//2)
      nCount := StrLen(MCode_PixelateBitmap)//2
      Loop (nCount)
         NumPut("0x" SubStr(MCode_PixelateBitmap, (2*A_Index)-1, 2), PixelateBitmap, A_Index-1, "UChar")

      DllCall("VirtualProtect", Ptr, &PixelateBitmap, Ptr, VarSetCapacity(PixelateBitmap), "uint", 0x40, "UPtr*", 0)
   }

   Gdip_GetImageDimensions(pBitmap, Width, Height)
   if (Width != Gdip_GetImageWidth(pBitmapOut) || Height != Gdip_GetImageHeight(pBitmapOut))
      return -1

   if (BlockSize > Width || BlockSize > Height)
      return -2

   E1 := Gdip_LockBits(pBitmap, 0, 0, Width, Height, Stride1, Scan01, BitmapData1)
   E2 := Gdip_LockBits(pBitmapOut, 0, 0, Width, Height, Stride2, Scan02, BitmapData2)
   if (E1 || E2)
      return -3

   ; E := - unused exit code
   DllCall(&PixelateBitmap, Ptr, Scan01, Ptr, Scan02, "int", Width, "int", Height, "int", Stride1, "int", BlockSize)

   Gdip_UnlockBits(pBitmap, BitmapData1), Gdip_UnlockBits(pBitmapOut, BitmapData2)
   return 0
}

;#####################################################################################

Gdip_ToARGB(A, R, G, B) {
   return (A << 24) | (R << 16) | (G << 8) | B
}

Gdip_FromARGB(ARGB, ByRef A, ByRef R, ByRef G, ByRef B) {
   A := (0xff000000 & ARGB) >> 24
   R := (0x00ff0000 & ARGB) >> 16
   G := (0x0000ff00 & ARGB) >> 8
   B := 0x000000ff & ARGB
}

Gdip_AFromARGB(ARGB) {
   return (0xff000000 & ARGB) >> 24
}

Gdip_RFromARGB(ARGB) {
   return (0x00ff0000 & ARGB) >> 16
}

Gdip_GFromARGB(ARGB) {
   return (0x0000ff00 & ARGB) >> 8
}

Gdip_BFromARGB(ARGB) {
   return 0x000000ff & ARGB
}

;#####################################################################################

StrGetB(Address, Length:=-1, Encoding:=0) {
   ; Flexible parameter handling:
   if !IsInteger(Length)
      Encoding := Length,  Length := -1

   ; Check for obvious errors.
   if (Address+0 < 1024)
      return

   ; Ensure 'Encoding' contains a numeric identifier.
   if (Encoding = "UTF-16")
      Encoding := 1200
   else if (Encoding = "UTF-8")
      Encoding := 65001
   else if SubStr(Encoding,1,2)="CP"
      Encoding := SubStr(Encoding,3)

   if !Encoding ; "" or 0
   {
      ; No conversion necessary, but we might not want the whole string.
      if (Length == -1)
         Length := DllCall("lstrlen", "uint", Address)
      VarSetCapacity(String, Length)
      DllCall("lstrcpyn", "str", String, "uint", Address, "int", Length + 1)
   }
   else if (Encoding = 1200) ; UTF-16
   {
      char_count := DllCall("WideCharToMultiByte", "uint", 0, "uint", 0x400, "uint", Address, "int", Length, "uint", 0, "uint", 0, "uint", 0, "uint", 0)
      VarSetCapacity(String, char_count)
      DllCall("WideCharToMultiByte", "uint", 0, "uint", 0x400, "uint", Address, "int", Length, "str", String, "int", char_count, "uint", 0, "uint", 0)
   }
   else if IsInteger(Encoding)
   {
      ; Convert from target encoding to UTF-16 then to the active code page.
      char_count := DllCall("MultiByteToWideChar", "uint", Encoding, "uint", 0, "uint", Address, "int", Length, "uint", 0, "int", 0)
      VarSetCapacity(String, char_count * 2)
      char_count := DllCall("MultiByteToWideChar", "uint", Encoding, "uint", 0, "uint", Address, "int", Length, "uint", &String, "int", char_count * 2)
      String := StrGetB(&String, char_count, 1200)
   }

   return String
}

Gdip_Startup(multipleInstances:=0) {
   Ptr := "UPtr"
   pToken := 0

   If (multipleInstances=0)
   {
      if !DllCall("GetModuleHandle", "str", "gdiplus", Ptr)
         DllCall("LoadLibrary", "str", "gdiplus")
   } Else DllCall("LoadLibrary", "str", "gdiplus")

   VarSetCapacity(si, A_PtrSize = 8 ? 24 : 16, 0), si := Chr(1)
   DllCall("gdiplus\GdiplusStartup", "UPtr*", pToken, Ptr, &si, Ptr, 0)
   return pToken
}

Gdip_Shutdown(pToken) {
   Ptr := "UPtr"

   DllCall("gdiplus\GdiplusShutdown", Ptr, pToken)
   hModule := DllCall("GetModuleHandle", "str", "gdiplus", Ptr)
   if hModule
      DllCall("FreeLibrary", Ptr, hModule)
   return 0
}

;#####################################################################################
; in AHK v1: uses normal 'if var is' command
; in AHK v2: all if's are expression-if, so the Integer variable is dereferenced to the string
;#####################################################################################
IsInteger(Var) {
   Static Integer := "Integer"
   If Var Is Integer
      Return True
   Return False
}

IsNumber(Var) {
   Static number := "number"
   If Var Is number
      Return True
   Return False
}

; ======================================================================================================================
; Multiple Display Monitors Functions -> msdn.microsoft.com/en-us/library/dd145072(v=vs.85).aspx
; by 'just me'
; https://autohotkey.com/boards/viewtopic.php?f=6&t=4606
; ======================================================================================================================

GetMonitorCount() {
   Monitors := MDMF_Enum()
   for k,v in Monitors
      count := A_Index
   return count
}

GetMonitorInfo(MonitorNum) {
   Monitors := MDMF_Enum()
   for k,v in Monitors
      if (v.Num = MonitorNum)
         return v
}

GetPrimaryMonitor() {
   Monitors := MDMF_Enum()
   for k,v in Monitors
      If (v.Primary)
         return v.Num
}

; ----------------------------------------------------------------------------------------------------------------------
; Name ..........: MDMF - Multiple Display Monitor Functions
; Description ...: Various functions for multiple display monitor environments
; Tested with ...: AHK 1.1.32.00 (A32/U32/U64) and 2.0-a108-a2fa0498 (U32/U64)
; Original Author: just me (https://www.autohotkey.com/boards/viewtopic.php?f=6&t=4606)
; Mod Authors ...: iPhilip, guest3456
; Changes .......: Modified to work with v2.0-a108 and changed 'Count' key to 'TotalCount' to avoid conflicts
; ................ Modified MDMF_Enum() so that it works under both AHK v1 and v2.
; ................ Modified MDMF_EnumProc() to provide Count and Primary keys to the Monitors array.
; ................ Modified MDMF_FromHWND() to allow flag values that determine the function's return value if the
; ................    window does not intersect any display monitor.
; ................ Modified MDMF_FromPoint() to allow the cursor position to be returned ByRef if not specified and
; ................    allow flag values that determine the function's return value if the point is not contained within
; ................    any display monitor.
; ................ Modified MDMF_FromRect() to allow flag values that determine the function's return value if the
; ................    rectangle does not intersect any display monitor.
;................. Modified MDMF_GetInfo() with minor changes.
; ----------------------------------------------------------------------------------------------------------------------
;
; ======================================================================================================================
; Multiple Display Monitors Functions -> msdn.microsoft.com/en-us/library/dd145072(v=vs.85).aspx =======================
; ======================================================================================================================
; Enumerates display monitors and returns an object containing the properties of all monitors or the specified monitor.
; ======================================================================================================================

MDMF_Enum(HMON := "") {
   Static CallbackFunc := Func(A_AhkVersion < "2" ? "RegisterCallback" : "CallbackCreate")
   Static EnumProc := CallbackFunc.Call("MDMF_EnumProc")
   Static Obj := (A_AhkVersion < "2") ? "Object" : "Map"
   Static Monitors := {}
   If (HMON = "") ; new enumeration
   {
      Monitors := %Obj%("TotalCount", 0)
      If !DllCall("User32.dll\EnumDisplayMonitors", "Ptr", 0, "Ptr", 0, "Ptr", EnumProc, "Ptr", &Monitors, "Int")
         Return False
   }
   Return (HMON = "") ? Monitors : Monitors.HasKey(HMON) ? Monitors[HMON] : False
}
; ======================================================================================================================
;  Callback function that is called by the MDMF_Enum function.
; ======================================================================================================================
MDMF_EnumProc(HMON, HDC, PRECT, ObjectAddr) {
   Monitors := Object(ObjectAddr)
   Monitors[HMON] := MDMF_GetInfo(HMON)
   Monitors["TotalCount"]++
   If (Monitors[HMON].Primary)
      Monitors["Primary"] := HMON
   Return True
}
; ======================================================================================================================
; Retrieves the display monitor that has the largest area of intersection with a specified window.
; The following flag values determine the function's return value if the window does not intersect any display monitor:
;    MONITOR_DEFAULTTONULL    = 0 - Returns NULL.
;    MONITOR_DEFAULTTOPRIMARY = 1 - Returns a handle to the primary display monitor. 
;    MONITOR_DEFAULTTONEAREST = 2 - Returns a handle to the display monitor that is nearest to the window.
; ======================================================================================================================
MDMF_FromHWND(HWND, Flag := 0) {
   Return DllCall("User32.dll\MonitorFromWindow", "Ptr", HWND, "UInt", Flag, "Ptr")
}
; ======================================================================================================================
; Retrieves the display monitor that contains a specified point.
; If either X or Y is empty, the function will use the current cursor position for this value and return it ByRef.
; The following flag values determine the function's return value if the point is not contained within any
; display monitor:
;    MONITOR_DEFAULTTONULL    = 0 - Returns NULL.
;    MONITOR_DEFAULTTOPRIMARY = 1 - Returns a handle to the primary display monitor. 
;    MONITOR_DEFAULTTONEAREST = 2 - Returns a handle to the display monitor that is nearest to the point.
; ======================================================================================================================
MDMF_FromPoint(ByRef X := "", ByRef Y := "", Flag := 0) {
   If (X = "") || (Y = "") {
      VarSetCapacity(PT, 8, 0)
      DllCall("User32.dll\GetCursorPos", "Ptr", &PT, "Int")
      If (X = "")
         X := NumGet(PT, 0, "Int")
      If (Y = "")
         Y := NumGet(PT, 4, "Int")
   }
   Return DllCall("User32.dll\MonitorFromPoint", "Int64", (X & 0xFFFFFFFF) | (Y << 32), "UInt", Flag, "Ptr")
}
; ======================================================================================================================
; Retrieves the display monitor that has the largest area of intersection with a specified rectangle.
; Parameters are consistent with the common AHK definition of a rectangle, which is X, Y, W, H instead of
; Left, Top, Right, Bottom.
; The following flag values determine the function's return value if the rectangle does not intersect any
; display monitor:
;    MONITOR_DEFAULTTONULL    = 0 - Returns NULL.
;    MONITOR_DEFAULTTOPRIMARY = 1 - Returns a handle to the primary display monitor. 
;    MONITOR_DEFAULTTONEAREST = 2 - Returns a handle to the display monitor that is nearest to the rectangle.
; ======================================================================================================================
MDMF_FromRect(X, Y, W, H, Flag := 0) {
   VarSetCapacity(RC, 16, 0)
   NumPut(X, RC, 0, "Int"), NumPut(Y, RC, 4, "Int"), NumPut(X + W, RC, 8, "Int"), NumPut(Y + H, RC, 12, "Int")
   Return DllCall("User32.dll\MonitorFromRect", "Ptr", &RC, "UInt", Flag, "Ptr")
}
; ======================================================================================================================
; Retrieves information about a display monitor.
; ======================================================================================================================
MDMF_GetInfo(HMON) {
   NumPut(VarSetCapacity(MIEX, 40 + (32 << !!A_IsUnicode)), MIEX, 0, "UInt")
   If DllCall("User32.dll\GetMonitorInfo", "Ptr", HMON, "Ptr", &MIEX, "Int")
      Return {Name:      (Name := StrGet(&MIEX + 40, 32))  ; CCHDEVICENAME = 32
            , Num:       RegExReplace(Name, ".*(\d+)$", "$1")
            , Left:      NumGet(MIEX, 4, "Int")    ; display rectangle
            , Top:       NumGet(MIEX, 8, "Int")    ; "
            , Right:     NumGet(MIEX, 12, "Int")   ; "
            , Bottom:    NumGet(MIEX, 16, "Int")   ; "
            , WALeft:    NumGet(MIEX, 20, "Int")   ; work area
            , WATop:     NumGet(MIEX, 24, "Int")   ; "
            , WARight:   NumGet(MIEX, 28, "Int")   ; "
            , WABottom:  NumGet(MIEX, 32, "Int")   ; "
            , Primary:   NumGet(MIEX, 36, "UInt")} ; contains a non-zero value for the primary monitor.
   Return False
}

;######################################################################################################################################
; The following functions are written by Just Me
; Taken from https://autohotkey.com/board/topic/85238-get-image-metadata-using-gdi-ahk-l/
; October 2013; minimal modifications by Marius Șucan in July 2019

Gdip_LoadImageFromFile(sFile, useICM:=0) {
; An Image object encapsulates a bitmap or a metafile and stores attributes that you can retrieve.
   pImage := 0
   function2call := (useICM=1) ? "GdipLoadImageFromFileICM" : "GdipLoadImageFromFile"
   R := DllCall("gdiplus\" function2call, "WStr", sFile, "UPtrP", pImage)
   ErrorLevel := R
   Return pImage
}

;######################################################################################################################################
; Gdip_GetPropertyCount() - Gets the number of properties (pieces of metadata) stored in this Image object.
; Parameters:
;     pImage      -  Pointer to the Image object.
; Return values:
;     On success  -  Number of properties.
;     On failure  -  0, ErrorLevel contains the GDIP status
;######################################################################################################################################

Gdip_GetPropertyCount(pImage) {
   PropCount := 0
   Ptr := "UPtr"
   R := DllCall("gdiplus\GdipGetPropertyCount", Ptr, pImage, "UIntP", PropCount)
   ErrorLevel := R
   Return PropCount
}

;######################################################################################################################################
; Gdip_GetPropertyIdList() - Gets an aray of the property identifiers used in the metadata of this Image object.
; Parameters:
;     pImage      -  Pointer to the Image object.
; Return values:
;     On success  -  Array containing the property identifiers as integer keys and the name retrieved from
;                    Gdip_GetPropertyTagName(PropID) as values.
;                    The total number of properties is stored in Array.Count.
;     On failure  -  False, ErrorLevel contains the GDIP status
;######################################################################################################################################

Gdip_GetPropertyIdList(pImage) {
   PropNum := Gdip_GetPropertyCount(pImage)
   Ptr := "UPtr"
   If (ErrorLevel) || (PropNum = 0)
      Return False
   VarSetCapacity(PropIDList, 4 * PropNum, 0)
   R := DllCall("gdiplus\GdipGetPropertyIdList", Ptr, pImage, "UInt", PropNum, "Ptr", &PropIDList)
   If (R) {
      ErrorLevel := R
      Return False
   }

   PropArray := {Count: PropNum}
   Loop (PropNum)
   {
      PropID := NumGet(PropIDList, (A_Index - 1) << 2, "UInt")
      PropArray[PropID] := Gdip_GetPropertyTagName(PropID)
   }
   Return PropArray
}

;######################################################################################################################################
; Gdip_GetPropertyItem() - Gets a specified property item (piece of metadata) from this Image object.
; Parameters:
;     pImage      -  Pointer to the Image object.
;     PropID      -  Integer that identifies the property item to be retrieved (see Gdip_GetPropertyTagName()).
; Return values:
;     On success  -  Property item object containing three keys:
;                    Length   -  Length of the value in bytes.
;                    Type     -  Type of the value (see Gdip_GetPropertyTagType()).
;                    Value    -  The value itself.
;     On failure  -  False, ErrorLevel contains the GDIP status
;######################################################################################################################################

Gdip_GetPropertyItem(pImage, PropID) {
   PropItem := {Length: 0, Type: 0, Value: ""}
   ItemSize := 0
   R := DllCall("gdiplus\GdipGetPropertyItemSize", "Ptr", pImage, "UInt", PropID, "UIntP", ItemSize)
   If (R) {
      ErrorLevel := R
      Return False
   }

   Ptr := "UPtr"
   VarSetCapacity(Item, ItemSize, 0)
   R := DllCall("gdiplus\GdipGetPropertyItem", Ptr, pImage, "UInt", PropID, "UInt", ItemSize, "Ptr", &Item)
   If (R) {
      ErrorLevel := R
      Return False
   }
   PropLen := NumGet(Item, 4, "UInt")
   PropType := NumGet(Item, 8, "Short")
   PropAddr := NumGet(Item, 8 + A_PtrSize, "UPtr")
   PropItem.Length := PropLen
   PropItem.Type := PropType
   If (PropLen > 0)
   {
      PropVal := ""
      Gdip_GetPropertyItemValue(PropVal, PropLen, PropType, PropAddr)
      If (PropType = 1) || (PropType = 7) {
         PropItem.SetCapacity("Value", PropLen)
         ValAddr := PropItem.GetAddress("Value")
         DllCall("Kernel32.dll\RtlMoveMemory", "Ptr", ValAddr, "Ptr", &PropVal, "Ptr", PropLen)
      } Else {
         PropItem.Value := PropVal
      }
   }
   ErrorLevel := 0
   Return PropItem
}

;######################################################################################################################################
; Gdip_GetAllPropertyItems() - Gets all the property items (metadata) stored in this Image object.
; Parameters:
;     pImage      -  Pointer to the Image object.
; Return values:
;     On success  -  Properties object containing one integer key for each property ID. Each value is an object
;                    containing three keys:
;                    Length   -  Length of the value in bytes.
;                    Type     -  Type of the value (see Gdip_GetPropertyTagType()).
;                    Value    -  The value itself.
;                    The total number of properties is stored in Properties.Count.
;     On failure  -  False, ErrorLevel contains the GDIP status
;######################################################################################################################################

Gdip_GetAllPropertyItems(pImage) {
   BufSize := PropNum := ErrorLevel := 0
   R := DllCall("gdiplus\GdipGetPropertySize", "Ptr", pImage, "UIntP", BufSize, "UIntP", PropNum)
   If (R) || (PropNum = 0) {
      ErrorLevel := R ? R : 19 ; 19 = PropertyNotFound
      Return False
   }
   VarSetCapacity(Buffer, BufSize, 0)
   Ptr := "UPtr"
   R := DllCall("gdiplus\GdipGetAllPropertyItems", Ptr, pImage, "UInt", BufSize, "UInt", PropNum, "Ptr", &Buffer)
   If (R) {
      ErrorLevel := R
      Return False
   }
   PropsObj := {Count: PropNum}
   PropSize := 8 + (2 * A_PtrSize)

   Loop (PropNum)
   {
      OffSet := PropSize * (A_Index - 1)
      PropID := NumGet(Buffer, OffSet, "UInt")
      PropLen := NumGet(Buffer, OffSet + 4, "UInt")
      PropType := NumGet(Buffer, OffSet + 8, "Short")
      PropAddr := NumGet(Buffer, OffSet + 8 + A_PtrSize, "UPtr")
      PropVal := ""
      PropsObj[PropID] := {}
      PropsObj[PropID, "Length"] := PropLen
      PropsObj[PropID, "Type"] := PropType
      PropsObj[PropID, "Value"] := PropVal
      If (PropLen > 0)
      {
         Gdip_GetPropertyItemValue(PropVal, PropLen, PropType, PropAddr)
         If (PropType = 1) || (PropType = 7)
         {
            PropsObj[PropID].SetCapacity("Value", PropLen)
            ValAddr := PropsObj[PropID].GetAddress("Value")
            DllCall("Kernel32.dll\RtlMoveMemory", "Ptr", ValAddr, "Ptr", PropAddr, "Ptr", PropLen)
         } Else {
            PropsObj[PropID].Value := PropVal
         }
      }
   }
   ErrorLevel := 0
   Return PropsObj
}

;######################################################################################################################################
; Gdip_GetPropertyTagName() - Gets the name for the integer identifier of this property as defined in "Gdiplusimaging.h".
; Parameters:
;     PropID      -  Integer that identifies the property item to be retrieved.
; Return values:
;     On success  -  Corresponding name.
;     On failure  -  "Unknown"
;######################################################################################################################################

Gdip_GetPropertyTagName(PropID) {
; All tags are taken from "Gdiplusimaging.h", probably there will be more.
; For most of them you'll find a description on http://msdn.microsoft.com/en-us/library/ms534418(VS.85).aspx
;
; modified by Marius Șucan in July/August 2019:
; I transformed the function to not yield errors on AHK v2

   Static PropTagsA := {0x0001:"GPS LatitudeRef",0x0002:"GPS Latitude",0x0003:"GPS LongitudeRef",0x0004:"GPS Longitude",0x0005:"GPS AltitudeRef",0x0006:"GPS Altitude",0x0007:"GPS Time",0x0008:"GPS Satellites",0x0009:"GPS Status",0x000A:"GPS MeasureMode",0x001D:"GPS Date",0x001E:"GPS Differential",0x00FE:"NewSubfileType",0x00FF:"SubfileType",0x0102:"Bits Per Sample",0x0103:"Compression",0x0106:"Photometric Interpolation",0x0107:"ThreshHolding",0x010A:"Fill Order",0x010D:"Document Name",0x010E:"Image Description",0x010F:"Equipment Make",0x0110:"Equipment Model",0x0112:"Orientation",0x0115:"Samples Per Pixel",0x0118:"Min Sample Value",0x0119:"Max Sample Value",0x011D:"Page Name",0x0122:"GrayResponseUnit",0x0123:"GrayResponseCurve",0x0128:"Resolution Unit",0x012D:"Transfer Function",0x0131:"Software Used",0x0132:"Internal Date Time",0x013B:"Artist"
   ,0x013C:"Host Computer",0x013D:"Predictor",0x013E:"White Point",0x013F:"Primary Chromaticities",0x0140:"Color Map",0x014C:"Ink Set",0x014D:"Ink Names",0x014E:"Number Of Inks",0x0150:"Dot Range",0x0151:"Target Printer",0x0152:"Extra Samples",0x0153:"Sample Format",0x0156:"Transfer Range",0x0200:"JPEGProc",0x0205:"JPEGLosslessPredictors",0x0301:"Gamma",0x0302:"ICC Profile Descriptor",0x0303:"SRGB Rendering Intent",0x0320:"Image Title",0x5010:"JPEG Quality",0x5011:"Grid Size",0x501A:"Color Transfer Function",0x5100:"Frame Delay",0x5101:"Loop Count",0x5110:"Pixel Unit",0x5111:"Pixel Per Unit X",0x5112:"Pixel Per Unit Y",0x8298:"Copyright",0x829A:"EXIF Exposure Time",0x829D:"EXIF F Number",0x8773:"ICC Profile",0x8822:"EXIF ExposureProg",0x8824:"EXIF SpectralSense",0x8827:"EXIF ISO Speed",0x9003:"EXIF Date Original",0x9004:"EXIF Date Digitized"
   ,0x9102:"EXIF CompBPP",0x9201:"EXIF Shutter Speed",0x9202:"EXIF Aperture",0x9203:"EXIF Brightness",0x9204:"EXIF Exposure Bias",0x9205:"EXIF Max. Aperture",0x9206:"EXIF Subject Dist",0x9207:"EXIF Metering Mode",0x9208:"EXIF Light Source",0x9209:"EXIF Flash",0x920A:"EXIF Focal Length",0x9214:"EXIF Subject Area",0x927C:"EXIF Maker Note",0x9286:"EXIF Comments",0xA001:"EXIF Color Space",0xA002:"EXIF PixXDim",0xA003:"EXIF PixYDim",0xA004:"EXIF Related WAV",0xA005:"EXIF Interop",0xA20B:"EXIF Flash Energy",0xA20E:"EXIF Focal X Res",0xA20F:"EXIF Focal Y Res",0xA210:"EXIF FocalResUnit",0xA214:"EXIF Subject Loc",0xA215:"EXIF Exposure Index",0xA217:"EXIF Sensing Method",0xA300:"EXIF File Source",0xA301:"EXIF Scene Type",0xA401:"EXIF Custom Rendered",0xA402:"EXIF Exposure Mode",0xA403:"EXIF White Balance",0xA404:"EXIF Digital Zoom Ratio"
   ,0xA405:"EXIF Focal Length In 35mm Film",0xA406:"EXIF Scene Capture Type",0xA407:"EXIF Gain Control",0xA408:"EXIF Contrast",0xA409:"EXIF Saturation",0xA40A:"EXIF Sharpness",0xA40B:"EXIF Device Setting Description",0xA40C:"EXIF Subject Distance Range",0xA420:"EXIF Unique Image ID"}

   Static PropTagsB := {0x0000:"GpsVer",0x000B:"GpsGpsDop",0x000C:"GpsSpeedRef",0x000D:"GpsSpeed",0x000E:"GpsTrackRef",0x000F:"GpsTrack",0x0010:"GpsImgDirRef",0x0011:"GpsImgDir",0x0012:"GpsMapDatum",0x0013:"GpsDestLatRef",0x0014:"GpsDestLat",0x0015:"GpsDestLongRef",0x0016:"GpsDestLong",0x0017:"GpsDestBearRef",0x0018:"GpsDestBear",0x0019:"GpsDestDistRef",0x001A:"GpsDestDist",0x001B:"GpsProcessingMethod",0x001C:"GpsAreaInformation",0x0100:"Original Image Width",0x0101:"Original Image Height",0x0108:"CellWidth",0x0109:"CellHeight",0x0111:"Strip Offsets",0x0116:"RowsPerStrip",0x0117:"StripBytesCount",0x011A:"XResolution",0x011B:"YResolution",0x011C:"Planar Config",0x011E:"XPosition",0x011F:"YPosition",0x0120:"FreeOffset",0x0121:"FreeByteCounts",0x0124:"T4Option",0x0125:"T6Option",0x0129:"PageNumber",0x0141:"Halftone Hints",0x0142:"TileWidth",0x0143:"TileLength",0x0144:"TileOffset"
   ,0x0145:"TileByteCounts",0x0154:"SMin Sample Value",0x0155:"SMax Sample Value",0x0201:"JPEGInterFormat",0x0202:"JPEGInterLength",0x0203:"JPEGRestartInterval",0x0206:"JPEGPointTransforms",0x0207:"JPEGQTables",0x0208:"JPEGDCTables",0x0209:"JPEGACTables",0x0211:"YCbCrCoefficients",0x0212:"YCbCrSubsampling",0x0213:"YCbCrPositioning",0x0214:"REFBlackWhite",0x5001:"ResolutionXUnit",0x5002:"ResolutionYUnit",0x5003:"ResolutionXLengthUnit",0x5004:"ResolutionYLengthUnit",0x5005:"PrintFlags",0x5006:"PrintFlagsVersion",0x5007:"PrintFlagsCrop",0x5008:"PrintFlagsBleedWidth",0x5009:"PrintFlagsBleedWidthScale",0x500A:"HalftoneLPI",0x500B:"HalftoneLPIUnit",0x500C:"HalftoneDegree",0x500D:"HalftoneShape",0x500E:"HalftoneMisc",0x500F:"HalftoneScreen",0x5012:"ThumbnailFormat",0x5013:"ThumbnailWidth",0x5014:"ThumbnailHeight",0x5015:"ThumbnailColorDepth"
   ,0x5016:"ThumbnailPlanes",0x5017:"ThumbnailRawBytes",0x5018:"ThumbnailSize",0x5019:"ThumbnailCompressedSize",0x501B:"ThumbnailData",0x5020:"ThumbnailImageWidth",0x5021:"ThumbnailImageHeight",0x5022:"ThumbnailBitsPerSample",0x5023:"ThumbnailCompression",0x5024:"ThumbnailPhotometricInterp",0x5025:"ThumbnailImageDescription",0x5026:"ThumbnailEquipMake",0x5027:"ThumbnailEquipModel",0x5028:"ThumbnailStripOffsets",0x5029:"ThumbnailOrientation",0x502A:"ThumbnailSamplesPerPixel",0x502B:"ThumbnailRowsPerStrip",0x502C:"ThumbnailStripBytesCount",0x502D:"ThumbnailResolutionX",0x502E:"ThumbnailResolutionY",0x502F:"ThumbnailPlanarConfig",0x5030:"ThumbnailResolutionUnit",0x5031:"ThumbnailTransferFunction",0x5032:"ThumbnailSoftwareUsed",0x5033:"ThumbnailDateTime",0x5034:"ThumbnailArtist",0x5035:"ThumbnailWhitePoint"
   ,0x5036:"ThumbnailPrimaryChromaticities",0x5037:"ThumbnailYCbCrCoefficients",0x5038:"ThumbnailYCbCrSubsampling",0x5039:"ThumbnailYCbCrPositioning",0x503A:"ThumbnailRefBlackWhite",0x503B:"ThumbnailCopyRight",0x5090:"LuminanceTable",0x5091:"ChrominanceTable",0x5102:"Global Palette",0x5103:"Index Background",0x5104:"Index Transparent",0x5113:"Palette Histogram",0x8769:"ExifIFD",0x8825:"GpsIFD",0x8828:"ExifOECF",0x9000:"ExifVer",0x9101:"EXIF CompConfig",0x9290:"EXIF DTSubsec",0x9291:"EXIF DTOrigSS",0x9292:"EXIF DTDigSS",0xA000:"EXIF FPXVer",0xA20C:"EXIF Spatial FR",0xA302:"EXIF CfaPattern"}

   r := PropTagsA.HasKey(PropID) ? PropTagsA[PropID] : "Unknown"
   If (r="Unknown")
      r := PropTagsB.HasKey(PropID) ? PropTagsB[PropID] : "Unknown"
   Return r
}

;######################################################################################################################################
; Gdip_GetPropertyTagType() - Gets the name for he type of this property's value as defined in "Gdiplusimaging.h".
; Parameters:
;     PropType    -  Integer that identifies the type of the property item to be retrieved.
; Return values:
;     On success  -  Corresponding type.
;     On failure  -  "Unknown"
;######################################################################################################################################

Gdip_GetPropertyTagType(PropType) {
   Static PropTypes := {1: "Byte", 2: "ASCII", 3: "Short", 4: "Long", 5: "Rational", 7: "Undefined", 9: "SLong", 10: "SRational"}
   Return PropTypes.HasKey(PropType) ? PropTypes[PropType] : "Unknown"
}

Gdip_GetPropertyItemValue(ByRef PropVal, PropLen, PropType, PropAddr) {
; Gdip_GetPropertyItemValue() - Reserved for internal use
   PropVal := ""
   If (PropType = 2)
   {
      PropVal := StrGet(PropAddr, PropLen, "CP0")
      Return True
   }

   If (PropType = 3)
   {
      PropyLen := PropLen // 2
      Loop (PropyLen)
         PropVal .= (A_Index > 1 ? " " : "") . NumGet(PropAddr + 0, (A_Index - 1) << 1, "Short")
      Return True
   }

   If (PropType = 4) || (PropType = 9)
   {
      NumType := PropType = 4 ? "UInt" : "Int"
      PropyLen := PropLen // 4
      Loop (PropyLen)
         PropVal .= (A_Index > 1 ? " " : "") . NumGet(PropAddr + 0, (A_Index - 1) << 2, NumType)
      Return True
   }

   If (PropType = 5) || (PropType = 10)
   {
      NumType := PropType = 5 ? "UInt" : "Int"
      PropyLen := PropLen // 8
      Loop (PropyLen)
         PropVal .= (A_Index > 1 ? " " : "") . NumGet(PropAddr + 0, (A_Index - 1) << 2, NumType)
                 .  "/" . NumGet(PropAddr + 4, (A_Index - 1) << 2, NumType)
      Return True
   }

   If (PropType = 1) || (PropType = 7)
   {
      VarSetCapacity(PropVal, PropLen, 0)
      DllCall("Kernel32.dll\RtlMoveMemory", "Ptr", &PropVal, "Ptr", PropAddr, "Ptr", PropLen)
      Return True
   }
   Return False
}

;#####################################################################################
; RotateAtCenter() and related Functions by RazorHalo
; from https://www.autohotkey.com/boards/viewtopic.php?f=6&t=6517&start=260
; in April 2019.
;#####################################################################################
; The Matrix order has to be "Append" for the transformations to be applied 
; in the correct order - instead of the default "Prepend"

Gdip_RotatePathAtCenter(pPath, Angle, MatrixOrder:=1, withinBounds:=0, withinBkeepRatio:=1) {
; modified by Marius Șucan - added withinBounds option

  ; Gets the bounding rectangle of the GraphicsPath
  ; returns array x, y, w, h
  Rect := Gdip_GetPathWorldBounds(pPath)

  ; Calculate center of bounding rectangle which will be the center of the graphics path
  cX := Rect.x + (Rect.w / 2)
  cY := Rect.y + (Rect.h / 2)
  
  ; Create a Matrix for the transformations
  pMatrix := Gdip_CreateMatrix()
  
  ; Move the GraphicsPath center to the origin (0, 0) of the graphics object
  Gdip_TranslateMatrix(pMatrix, -cX , -cY)

  ; Rotate matrix on graphics object origin
  Gdip_RotateMatrix(pMatrix, Angle, MatrixOrder)
  
  ; Move the GraphicsPath origin point back to its original position
  Gdip_TranslateMatrix(pMatrix, cX, cY, MatrixOrder)

  ; Apply the transformations
  E := Gdip_TransformPath(pPath, pMatrix)

  ; Delete Matrix
  Gdip_DeleteMatrix(pMatrix)

  If (withinBounds=1 && !E && Angle!=0)
  {
     nRect := Gdip_GetPathWorldBounds(pPath)
     ncX := nRect.x + (nRect.w / 2)
     ncY := nRect.y + (nRect.h / 2)
     pMatrix := Gdip_CreateMatrix()
     Gdip_TranslateMatrix(pMatrix, -ncX , -ncY)
     sX := Rect.w / nRect.w
     sY := Rect.h / nRect.h
     If (withinBkeepRatio=1)
     {
        sX := min(sX, sY)
        sY := min(sX, sY)
     }
     Gdip_ScaleMatrix(pMatrix, sX, sY, MatrixOrder)
     Gdip_TranslateMatrix(pMatrix, ncX, ncY, MatrixOrder)
     If (sX!=0 && sY!=0)
        E := Gdip_TransformPath(pPath, pMatrix)
     Gdip_DeleteMatrix(pMatrix)
  }
  Return E
}

;#####################################################################################
; Matrix transformations functions by RazorHalo
;
; NOTE: Be aware of the order that transformations are applied.  You may need
; to pass MatrixOrder as 1 for "Append"
; the (default is 0 for "Prepend") to get the correct results.

Gdip_ResetMatrix(hMatrix) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipResetMatrix", Ptr, hMatrix)
}

Gdip_RotateMatrix(hMatrix, Angle, MatrixOrder:=0) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipRotateMatrix", Ptr, hMatrix, "float", Angle, "Int", MatrixOrder)
}


Gdip_GetPathWorldBounds(pPath, hMatrix:=0, pPen:=0) {
; hMatrix to use for calculating the boundaries
; pPen to use for calculating the boundaries
; Both will not affect the actual GraphicsPath.

  Ptr := "UPtr"
  rData := {}

  VarSetCapacity(RectF, 16, 0)
  status := DllCall("gdiplus\GdipGetPathWorldBounds", Ptr, pPath, Ptr, &RectF, Ptr, hMatrix, Ptr, pPen)

  If (!status) {
        rData.x := NumGet(&RectF, 0, "float")
      , rData.y := NumGet(&RectF, 4, "float")
      , rData.w := NumGet(&RectF, 8, "float")
      , rData.h := NumGet(&RectF, 12, "float")
  } Else {
    Return status
  }
  
  return rData
}

Gdip_ScaleMatrix(hMatrix, ScaleX, ScaleY, MatrixOrder:=0) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipScaleMatrix", Ptr, hMatrix, "float", ScaleX, "float", ScaleY, "Int", MatrixOrder)
}

Gdip_TranslateMatrix(hMatrix, offsetX, offsetY, MatrixOrder:=0) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipTranslateMatrix", Ptr, hMatrix, "float", offsetX, "float", offsetY, "Int", MatrixOrder)
}

Gdip_TransformPath(pPath, hMatrix) {
  Ptr := "UPtr"
  return DllCall("gdiplus\GdipTransformPath", Ptr, pPath, Ptr, hMatrix)
}

Gdip_SetMatrixElements(hMatrix, m11, m12, m21, m22, x, y) {
  Ptr := "UPtr"
  return DllCall("gdiplus\GdipSetMatrixElements", Ptr, hMatrix, "float", m11, "float", m12, "float", m21, "float", m22, "float", x, "float", y)
}

Gdip_GetMatrixLastStatus(pMatrix) {
  Ptr := "UPtr"
  return DllCall("gdiplus\GdipGetLastStatus", Ptr, pMatrix)
}

;#####################################################################################
; GraphicsPath functions written by Learning one
; found on https://autohotkey.com/board/topic/29449-gdi-standard-library-145-by-tic/page-75
; Updated on 14/08/2019 by Marius Șucan
;#####################################################################################
;
; Function:    Gdip_AddPathBeziers
; Description: Adds a sequence of connected Bézier splines to the current figure of this path.
; A Bezier spline does not pass through its control points. The control points act as magnets, pulling the curve
; in certain directions to influence the way the spline bends.
;
; pPath:  Pointer to the GraphicsPath.
; Points: The coordinates of all the points passed as x1,y1|x2,y2|x3,y3...
;
; Return: Status enumeration. 0 = success.
;
; Notes: The first spline is constructed from the first point through the fourth point in the array and uses the second and third points as control points. Each subsequent spline in the sequence needs exactly three more points: the ending point of the previous spline is used as the starting point, the next two points in the sequence are control points, and the third point is the ending point.

Gdip_AddPathBeziers(pPath, Points) {
  Ptr := "UPtr"
  iCount := CreatePointsF(PointsF, Points)
  return DllCall("gdiplus\GdipAddPathBeziers", Ptr, pPath, Ptr, &PointsF, "int", iCount)
}

Gdip_AddPathBezier(pPath, x1, y1, x2, y2, x3, y3, x4, y4) {
  ; Adds a Bézier spline to the current figure of this path
  Ptr := "UPtr"
  return DllCall("gdiplus\GdipAddPathBezier", Ptr, pPath
         , "float", x1, "float", y1, "float", x2, "float", y2
         , "float", x3, "float", y3, "float", x4, "float", y4)
}

;#####################################################################################
; Function: Gdip_AddPathLines
; Description: Adds a sequence of connected lines to the current figure of this path.
;
; pPath: Pointer to the GraphicsPath
; Points: the coordinates of all the points passed as x1,y1|x2,y2|x3,y3.....
;
; Return: status enumeration. 0 = success.

Gdip_AddPathLines(pPath, Points) {
  Ptr := "UPtr"
  iCount := CreatePointsF(PointsF, Points)
  return DllCall("gdiplus\GdipAddPathLine2", Ptr, pPath, Ptr, &PointsF, "int", iCount)
}

Gdip_AddPathLine(pPath, x1, y1, x2, y2) {
  Ptr := "UPtr"
  return DllCall("gdiplus\GdipAddPathLine", Ptr, pPath, "float", x1, "float", y1, "float", x2, "float", y2)
}

Gdip_AddPathArc(pPath, x, y, w, h, StartAngle, SweepAngle) {
  Ptr := "UPtr"
  return DllCall("gdiplus\GdipAddPathArc", Ptr, pPath, "float", x, "float", y, "float", w, "float", h, "float", StartAngle, "float", SweepAngle)
}

Gdip_AddPathPie(pPath, x, y, w, h, StartAngle, SweepAngle) {
  Ptr := "UPtr"
  return DllCall("gdiplus\GdipAddPathPie", Ptr, pPath, "float", x, "float", y, "float", w, "float", h, "float", StartAngle, "float", SweepAngle)
}

Gdip_StartPathFigure(pPath) {
; Starts a new figure without closing the current figure.
; Subsequent points added to this path are added to the new figure.
  Ptr := "UPtr"
  return DllCall("gdiplus\GdipStartPathFigure", Ptr, pPath)
}

Gdip_ClosePathFigure(pPath) {
; Closes the current figure of this path.
  Ptr := "UPtr"
  return DllCall("gdiplus\GdipClosePathFigure", Ptr, pPath)
}

Gdip_ClosePathFigures(pPath) {
; Closes the current figure of this path.
  Ptr := "UPtr"
  return DllCall("gdiplus\GdipClosePathFigures", Ptr, pPath)
}

;#####################################################################################
; Function: Gdip_DrawPath
; Description: Draws a sequence of lines and curves defined by a GraphicsPath object
;
; pGraphics: Pointer to the Graphics of a bitmap
; pPen: Pointer to a pen object
; pPath: Pointer to a Path object
;
; Return: status enumeration. 0 = success.

Gdip_DrawPath(pGraphics, pPen, pPath) {
  Ptr := "UPtr"
  return DllCall("gdiplus\GdipDrawPath", Ptr, pGraphics, Ptr, pPen, Ptr, pPath)
}

Gdip_ClonePath(pPath) {
  Ptr := "UPtr"
  PtrA:= "UPtr*"
  pPathClone := 0
  DllCall("gdiplus\GdipClonePath", Ptr, pPath, PtrA, pPathClone)
  return pPathClone
}

;######################################################################################################################################
; The following PathGradient brush functions were written by 'Just Me' in March 2012
; source: https://autohotkey.com/board/topic/29449-gdi-standard-library-145-by-tic/page-65
;######################################################################################################################################

Gdip_PathGradientCreateFromPath(pPath) {
   ; Creates and returns a path gradient brush.
   ; pPath              path object returned from Gdip_CreatePath()
   pBrush := 0
   DllCall("gdiplus\GdipCreatePathGradientFromPath", "Ptr", pPath, "PtrP", pBrush)
   Return pBrush
}

Gdip_PathGradientSetCenterPoint(pBrush, X, Y) {
   ; Sets the center point of this path gradient brush.
   ; pBrush             Brush object returned from Gdip_PathGradientCreateFromPath().
   ; X, Y               X, y coordinates in pixels
   VarSetCapacity(POINTF, 8)
   NumPut(X, POINTF, 0, "Float")
   NumPut(Y, POINTF, 4, "Float")
   Return DllCall("gdiplus\GdipSetPathGradientCenterPoint", "Ptr", pBrush, "Ptr", &POINTF)
}

Gdip_PathGradientSetCenterColor(pBrush, CenterColor) {
   ; Sets the center color of this path gradient brush.
   ; pBrush             Brush object returned from Gdip_PathGradientCreateFromPath().
   ; CenterColor        ARGB color value: A(lpha)R(ed)G(reen)B(lue).
   Return DllCall("gdiplus\GdipSetPathGradientCenterColor", "Ptr", pBrush, "UInt", CenterColor)   
}

Gdip_PathGradientSetSurroundColors(pBrush, SurroundColors) {
   ; Sets the surround colors of this path gradient brush. 
   ; pBrush             Brush object returned from Gdip_PathGradientCreateFromPath().
   ; SurroundColours    One or more ARGB color values seperated by pipe (|)).
   ; updated by Marius Șucan 

   Colors := StrSplit(SurroundColors, "|")
   tColors := Colors.Length
   VarSetCapacity(ColorArray, 4 * tColors, 0)

   Loop (tColors)
      NumPut(Colors[A_Index], ColorArray, 4 * (A_Index - 1), "UInt")

   Return DllCall("gdiplus\GdipSetPathGradientSurroundColorsWithCount", "Ptr", pBrush, "Ptr", &ColorArray
                , "IntP", tColors)
}

Gdip_PathGradientSetSigmaBlend(pBrush, Focus, Scale:=1) {
   ; Sets the blend shape of this path gradient brush to bell shape.
   ; pBrush             Brush object returned from Gdip_PathGradientCreateFromPath().
   ; Focus              Number that specifies where the center color will be at its highest intensity.
   ;                    Values: 1.0 (center) - 0.0 (border)
   ; Scale              Number that specifies the maximum intensity of center color that gets blended with 
   ;                    the boundary color.
   ;                    Values:  1.0 (100 %) - 0.0 (0 %)
   Return DllCall("gdiplus\GdipSetPathGradientSigmaBlend", "Ptr", pBrush, "Float", Focus, "Float", Scale)
}

Gdip_PathGradientSetLinearBlend(pBrush, Focus, Scale:=1) {
   ; Sets the blend shape of this path gradient brush to triangular shape.
   ; pBrush             Brush object returned from Gdip_PathGradientCreateFromPath()
   ; Focus              Number that specifies where the center color will be at its highest intensity.
   ;                    Values: 1.0 (center) - 0.0 (border)
   ; Scale              Number that specifies the maximum intensity of center color that gets blended with 
   ;                    the boundary color.
   ;                    Values:  1.0 (100 %) - 0.0 (0 %)
   Return DllCall("gdiplus\GdipSetPathGradientLinearBlend", "Ptr", pBrush, "Float", Focus, "Float", Scale)
}

Gdip_PathGradientSetFocusScales(pBrush, xScale, yScale) {
   ; Sets the focus scales of this path gradient brush.
   ; pBrush             Brush object returned from Gdip_PathGradientCreateFromPath().
   ; xScale             Number that specifies the x focus scale.
   ;                    Values: 0.0 (0 %) - 1.0 (100 %)
   ; yScale             Number that specifies the y focus scale.
   ;                    Values: 0.0 (0 %) - 1.0 (100 %)
   Return DllCall("gdiplus\GdipSetPathGradientFocusScales", "Ptr", pBrush, "Float", xScale, "Float", yScale)
}

Gdip_AddPathGradient(pGraphics, x, y, w, h, cX, cY, cClr, sClr, BlendFocus, ScaleX, ScaleY, Shape, Angle:=0) {
; Parameters:
; X, Y   - coordinates where to add the gradient path object 
; W, H   - the width and height of the path gradient object 
; cX, cY - the coordinates of the Center Point of the gradient within the wdith and height object boundaries
; cClr   - the center color in 0xARGB
; sClr   - the surrounding color in 0xARGB
; BlendFocus - 0.0 to 1.0; where the center color reaches the highest intensity
; Shape   - 1 = rectangle ; 0 = ellipse
; Angle   - Rotate the pPathGradientBrush at given angle
;
; function based on the example provided by Just Me for the path gradient functions
; adaptations/modifications by Marius Șucan

   pPath := Gdip_CreatePath()
   If (Shape=1)
      Gdip_AddPathRectangle(pPath, x, y, W, H)
   Else
      Gdip_AddPathEllipse(pPath, x, y, W, H)
   zBrush := Gdip_PathGradientCreateFromPath(pPath)
   If (Angle!=0)
      Gdip_RotatePathGradientAtCenter(zBrush, Angle)
   Gdip_PathGradientSetCenterPoint(zBrush, cX, cY)
   Gdip_PathGradientSetCenterColor(zBrush, cClr)
   Gdip_PathGradientSetSurroundColors(zBrush, sClr)
   Gdip_PathGradientSetSigmaBlend(zBrush, BlendFocus)
   Gdip_PathGradientSetLinearBlend(zBrush, BlendFocus)
   Gdip_PathGradientSetFocusScales(zBrush, ScaleX, ScaleY)
   E := Gdip_FillPath(pGraphics, zBrush, pPath)
   Gdip_DeleteBrush(zBrush)
   Gdip_DeletePath(pPath)
   Return E
}

;######################################################################################################################################
; The following PathGradient brush functions were written by Marius Șucan
;######################################################################################################################################

Gdip_CreatePathGradient(Points, WrapMode) {
; Creates a PathGradientBrush object based on an array of points and initializes the wrap mode of the brush
;
; Points array format:
; Points := "x1,y1|x2,y2|x3,y3|x4,y4" [... and so on]
;
; WrapMode options: specifies how an area is tiled when it is painted with a brush:
; 0 - Tile - Tiling without flipping
; 1 - TileFlipX - Tiles are flipped horizontally as you move from one tile to the next in a row
; 2 - TileFlipY - Tiles are flipped vertically as you move from one tile to the next in a column
; 3 - TileFlipXY - Tiles are flipped horizontally as you move along a row and flipped vertically as you move along a column
; 4 - Clamp - No tiling

    Ptr := "UPtr"
    iCount := CreatePointsF(PointsF, Points)
    pPathGradientBrush := 0
    DllCall("gdiplus\GdipCreatePathGradient", Ptr, &PointsF, "int", iCount, "int", WrapMode, "int*", pPathGradientBrush)
    Return pPathGradientBrush
}

Gdip_PathGradientGetGammaCorrection(pPathGradientBrush) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetPathGradientGammaCorrection", Ptr, pPathGradientBrush, "int*", result)
   If E
      return -1
   Return result
}

Gdip_PathGradientGetPointCount(pPathGradientBrush) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetPathGradientPointCount", Ptr, pPathGradientBrush, "int*", result)
   If E
      return -1
   Return result
}

Gdip_PathGradientGetWrapMode(pPathGradientBrush) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetPathGradientWrapMode", Ptr, pPathGradientBrush, "int*", result)
   If E
      return -1
   Return result
}

Gdip_PathGradientGetRect(pPathGradientBrush) {
  Ptr := "UPtr"
  rData := {}

  VarSetCapacity(RectF, 16, 0)
  status := DllCall("gdiplus\GdipGetPathGradientRect", Ptr, pPathGradientBrush, Ptr, &RectF)

  If (!status) {
        rData.x := NumGet(&RectF, 0, "float")
      , rData.y := NumGet(&RectF, 4, "float")
      , rData.w := NumGet(&RectF, 8, "float")
      , rData.h := NumGet(&RectF, 12, "float")
  } Else {
    Return status
  }

  return rData
}

Gdip_PathGradientResetTransform(pPathGradientBrush) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipResetPathGradientTransform", Ptr, pPathGradientBrush)
}

Gdip_PathGradientRotateTransform(pPathGradientBrush, Angle, matrixOrder:=0) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipRotatePathGradientTransform", Ptr, pPathGradientBrush, "float", Angle, "int", matrixOrder)
}

Gdip_PathGradientScaleTransform(pPathGradientBrush, ScaleX, ScaleY, matrixOrder:=0) {
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipScalePathGradientTransform", Ptr, pPathGradientBrush, "float", ScaleX, "float", ScaleY, "int", matrixOrder)
}

Gdip_PathGradientTranslateTransform(pPathGradientBrush, X, Y, matrixOrder:=0) {
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipTranslatePathGradientTransform", Ptr, pPathGradientBrush, "float", X, "float", Y, "int", matrixOrder)
}

Gdip_PathGradientMultiplyTransform(pPathGradientBrush, hMatrix, matrixOrder:=0) {
   Ptr := "UPtr"
   Return DllCall("gdiplus\GdipMultiplyPathGradientTransform", Ptr, pPathGradientBrush, Ptr, hMatrix, "int", matrixOrder)
}

Gdip_PathGradientSetTransform(pPathGradientBrush, pMatrix) {
  Ptr := "UPtr"
  return DllCall("gdiplus\GdipSetPathGradientTransform", Ptr, pPathGradientBrush, Ptr, pMatrix)
}

Gdip_PathGradientGetTransform(pPathGradientBrush) {
   Ptr := "UPtr"
   pMatrix := 0
   DllCall("gdiplus\GdipGetPathGradientTransform", Ptr, pPathGradientBrush, "UPtr*", pMatrix)
   Return pMatrix
}

Gdip_RotatePathGradientAtCenter(pPathGradientBrush, Angle, MatrixOrder:=1) {
; function by Marius Șucan
; based on Gdip_RotatePathAtCenter() by RazorHalo

  Rect := Gdip_PathGradientGetRect(pPathGradientBrush)
  cX := Rect.x + (Rect.w / 2)
  cY := Rect.y + (Rect.h / 2)
  pMatrix := Gdip_CreateMatrix()
  Gdip_TranslateMatrix(pMatrix, -cX , -cY)
  Gdip_RotateMatrix(pMatrix, Angle, MatrixOrder)
  Gdip_TranslateMatrix(pMatrix, cX, cY, MatrixOrder)
  E := Gdip_PathGradientSetTransform(pPathGradientBrush, pMatrix)
  Gdip_DeleteMatrix(pMatrix)
  Return E
}


Gdip_PathGradientSetGammaCorrection(pPathGradientBrush, UseGammaCorrection) {
; Specifies whether gamma correction is enabled for a path gradient brush
; UseGammaCorrection: 1 or 0.
   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetPathGradientGammaCorrection", Ptr, pPathGradientBrush, "int", UseGammaCorrection)
}

Gdip_PathGradientSetWrapMode(pPathGradientBrush, WrapMode) {
; WrapMode options: specifies how an area is tiled when it is painted with a brush:
; 0 - Tile - Tiling without flipping
; 1 - TileFlipX - Tiles are flipped horizontally as you move from one tile to the next in a row
; 2 - TileFlipY - Tiles are flipped vertically as you move from one tile to the next in a column
; 3 - TileFlipXY - Tiles are flipped horizontally as you move along a row and flipped vertically as you move along a column
; 4 - Clamp - No tiling

   Ptr := "UPtr"
   return DllCall("gdiplus\GdipSetPathGradientWrapMode", Ptr, pPathGradientBrush, "int", WrapMode)
}

Gdip_PathGradientGetCenterColor(pPathGradientBrush) {
   Ptr := "UPtr"
   ARGB := 0
   E := DllCall("gdiplus\GdipGetPathGradientCenterColor", Ptr, pPathGradientBrush, "uint*", ARGB)
   If E
      return -1
   Return Format("{1:#x}", ARGB)
}

Gdip_PathGradientGetCenterPoint(pPathGradientBrush, ByRef X, ByRef Y) {
   Ptr := "UPtr"
   VarSetCapacity(PointF, 8, 0)
   E := DllCall("gdiplus\GdipGetPathGradientCenterPoint", Ptr, pPathGradientBrush, "UPtr", &PointF)
   If !E
   {
      x := NumGet(PointF, 0, "float")
      y := NumGet(PointF, 4, "float")
   }
   Return E
}

Gdip_PathGradientGetFocusScales(pPathGradientBrush, ByRef X, ByRef Y) {
   Ptr := "UPtr"
   x := 0
   y := 0
   Return DllCall("gdiplus\GdipGetPathGradientFocusScales", Ptr, pPathGradientBrush, "float*", X, "float*", Y)
}

Gdip_PathGradientGetSurroundColorCount(pPathGradientBrush) {
   Ptr := "UPtr"
   result := 0
   E := DllCall("gdiplus\GdipGetPathGradientSurroundColorCount", Ptr, pPathGradientBrush, "int*", result)
   If E
      return -1
   Return result
}

Gdip_GetPathGradientSurroundColors(pPathGradientBrush) {
   iCount := Gdip_PathGradientGetSurroundColorCount(pPathGradientBrush)
   If (iCount=-1)
      Return 0


   Ptr := "UPtr"
   VarSetCapacity(sColors, 8 * iCount, 0)
   DllCall("gdiplus\GdipGetPathGradientSurroundColorsWithCount", Ptr, pPathGradientBrush, Ptr, &sColors, "intP", iCount)
   Loop (iCount)
   {
       A := NumGet(&sColors, 8*(A_Index-1), "uint")
       printList .= Format("{1:#x}", A) ","
   }

   Return Trim(printList, ",")
}

;######################################################################################################################################
; Function written by swagfag in July 2019
; source https://www.autohotkey.com/boards/viewtopic.php?f=6&t=62550
; modified by Marius Șucan
; whichFormat = 2;  histogram for each channel: R, G, B
; whichFormat = 3;  histogram of the luminance/brightness of the image
; Return: Status enumerated return type; 0 = OK/Success

Gdip_GetHistogram(pBitmap, whichFormat, ByRef newArrayA, ByRef newArrayB, ByRef newArrayC) {
   Static sizeofUInt := 4

   ; HistogramFormats := {ARGB: 0, PARGB: 1, RGB: 2, Gray: 3, B: 4, G: 5, R: 6, A: 7}
   z := DllCall("gdiplus\GdipBitmapGetHistogramSize", "UInt", whichFormat, "UInt*", numEntries)

   newArrayA := [], newArrayB := [], newArrayC := []
   VarSetCapacity(ch0, numEntries * sizeofUInt)
   VarSetCapacity(ch1, numEntries * sizeofUInt)
   VarSetCapacity(ch2, numEntries * sizeofUInt)
   If (whichFormat=2)
      r := DllCall("gdiplus\GdipBitmapGetHistogram", "Ptr", pBitmap, "UInt", whichFormat, "UInt", numEntries, "Ptr", &ch0, "Ptr", &ch1, "Ptr", &ch2, "Ptr", 0)
   Else If (whichFormat>2)
      r := DllCall("gdiplus\GdipBitmapGetHistogram", "Ptr", pBitmap, "UInt", whichFormat, "UInt", numEntries, "Ptr", &ch0, "Ptr", 0, "Ptr", 0, "Ptr", 0)

   Loop (numEntries)
   {
      i := A_Index - 1
      r := NumGet(&ch0+0, i * sizeofUInt, "UInt")
      newArrayA[i] := r

      If (whichFormat=2)
      {
         g := NumGet(&ch1+0, i * sizeofUInt, "UInt")
         b := NumGet(&ch2+0, i * sizeofUInt, "UInt")
         newArrayB[i] := g
         newArrayC[i] := b
      }
   }

   Return r
}

Gdip_DrawRoundedLine(G, x1, y1, x2, y2, LineWidth, LineColor) {
; function by DevX and Rabiator found on:
; https://autohotkey.com/board/topic/29449-gdi-standard-library-145-by-tic/page-11

  pPen := Gdip_CreatePen(LineColor, LineWidth) 
  Gdip_DrawLine(G, pPen, x1, y1, x2, y2) 
  Gdip_DeletePen(pPen) 

  pPen := Gdip_CreatePen(LineColor, LineWidth/2) 
  Gdip_DrawEllipse(G, pPen, x1-LineWidth/4, y1-LineWidth/4, LineWidth/2, LineWidth/2)
  Gdip_DrawEllipse(G, pPen, x2-LineWidth/4, y2-LineWidth/4, LineWidth/2, LineWidth/2)
  Gdip_DeletePen(pPen) 
}

Gdi_CreateBitmap(hDC:="", w:=1, h:=1, BitCount:=32, Planes:=1, pBits:=0) {
; Creates a GDI bitmap; it can be a DIB or DDB.
; https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createcompatiblebitmap
    If (hDC="")
       Return DllCall("Gdi32\CreateBitmap", "Int", w, "Int", h, "UInt", Planes, "UInt", BitCount, "Ptr", pBits, "UPtr")
    Else
       Return DllCall("gdi32\CreateCompatibleBitmap", "UPtr", hdc, "int", w, "int", h)
}

Gdi_CreateDIBitmap(hdc, bmpInfoHeader, CBM_INIT, pBits, BITMAPINFO, DIB_COLORS) {
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

   Ptr := "UPtr"
   hBitmap := DllCall("CreateDIBitmap"
            , Ptr, hdc
            , Ptr, bmpInfoHeader
            , "uint", CBM_INIT    ; =4
            , Ptr, pBits
            , Ptr, BITMAPINFO
            , "uint", DIB_COLORS, Ptr)    ; PAL=1 ; RGB=2

   Return hBitmap
}

Gdip_CreateBitmapFromGdiDib(BITMAPINFO, BitmapData) {
   Ptr := "UPtr"
   pBitmap := 0
   E := DllCall("gdiplus\GdipCreateBitmapFromGdiDib", Ptr, BITMAPINFO, Ptr, BitmapData, "UPtr*", pBitmap)
   Return pBitmap
}

Gdi_StretchDIBits(hDestDC, dX, dY, dW, dH, sX, sY, sW, sH, tBITMAPINFO, DIB_COLORS, pBits, RasterOper) {
   Ptr := "UPtr"
   Return DllCall("StretchDIBits"
      , Ptr, hDestDC, "int", dX, "int", dY
      , "int", dW, "int", dH, "int", sX, "int", sY
      , "int", sW, "int", sH, Ptr, pBits, Ptr, tBITMAPINFO
      , "int", DIB_COLORS, "uint", RasterOper)
}

Gdi_SetDIBitsToDevice(hDC, dX, dY, Width, Height, sX, sY, StartScan, ScanLines, pBits, BITMAPINFO, DIB_COLORS) {
   Ptr := "UPtr"
   Return DllCall("SetDIBitsToDevice", Ptr, hDC
         , "int", dX, "int", dY
         , "uint", Width, "uint", Height
         , "int", sX, "int", sY
         , "uint", StartScan, "uint", ScanLines
         , Ptr, pBits, Ptr, BITMAPINFO, "uint", DIB_COLORS)
}

Gdi_GetDIBits(hDC, hBitmap, start, cLines, pBits, BITMAPINFO, DIB_COLORS) {
; hDC     - A handle to the device context.
; hBitmap - A handle to the GDI bitmap. This must be a compatible bitmap (DDB).
; pbits   --A pointer to a buffer to receive the bitmap data.
;           If this parameter is NULL, the function passes the dimensions
;           and format of the bitmap to the BITMAPINFO structure pointed to 
;           by the BITMAPINFO parameter.
; A DDB is a Device-Dependent Bitmap, (as opposed to a DIB, or Device-Independent Bitmap).
; That means: a DDB does not contain color values; instead, the colors are in a
; device-dependent format. Therefore, it requires a hDC.
; 
; This function returns the data-bits as device-independent bitmap
; from a hBitmap into the pBits pointer.
;
; Return: if the function fails, the return value is zero.
; It can also return ERROR_INVALID_PARAMETER.
; Function written by Marius Șucan.

   Ptr := "UPtr"
   Return DllCall("GetDIBits"
            , Ptr, hDC
            , Ptr, hBitmap
            , "uint", start
            , "uint", cLines
            , Ptr, pBits
            , Ptr, BITMAPINFO
            , "uint", DIB_COLORS, Ptr)    ; PAL=1 ; RGB=2
}

;#####################################################################################

; Function        Gdip_DrawImageFX
; Description     This function draws a bitmap into the pGraphics that can use an Effect.
;
; pGraphics       Pointer to the Graphics of a bitmap
; pBitmap         Pointer to a bitmap to be drawn
; dX, dY          x, y coordinates of the destination upper-left corner where the image will be painted
; sX, sY          x, y coordinates of the source upper-left corner
; sW, sH          width and height of the source image
; Matrix          a color matrix used to alter image attributes when drawing
; pEffect         a pointer to an Effect object to apply when drawing the image
; hMatrix         a pointer to a transformation matrix
; Unit            Unit of measurement:
;                 0 - World coordinates, a nonphysical unit
;                 1 - Display units
;                 2 - A unit is 1 pixel
;                 3 - A unit is 1 point or 1/72 inch
;                 4 - A unit is 1 inch
;                 5 - A unit is 1/300 inch
;                 6 - A unit is 1 millimeter
;
; return          status enumeration. 0 = success
;
; notes on the color matrix:
;                 Matrix can be omitted to just draw with no alteration to ARGB
;                 Matrix may be passed as a digit from 0.0 - 1.0 to change just transparency
;                 Matrix can be passed as a matrix with "|" as delimiter. For example:
;                 MatrixBright=
;                 (
;                 1.5   |0    |0    |0    |0
;                 0     |1.5  |0    |0    |0
;                 0     |0    |1.5  |0    |0
;                 0     |0    |0    |1    |0
;                 0.05  |0.05 |0.05 |0    |1
;                 )
;
; example color matrix:
;                 MatrixBright = 1.5|0|0|0|0|0|1.5|0|0|0|0|0|1.5|0|0|0|0|0|1|0|0.05|0.05|0.05|0|1
;                 MatrixGreyScale = 0.299|0.299|0.299|0|0|0.587|0.587|0.587|0|0|0.114|0.114|0.114|0|0|0|0|0|1|0|0|0|0|0|1
;                 MatrixNegative = -1|0|0|0|0|0|-1|0|0|0|0|0|-1|0|0|0|0|0|1|0|1|1|1|0|1
;                 To generate a color matrix using user-friendly parameters,
;                 use GenerateColorMatrix()
; Function written by Marius Șucan.


Gdip_DrawImageFX(pGraphics, pBitmap, dX:="", dY:="", sX:="", sY:="", sW:="", sH:="", matrix:="", pEffect:="", ImageAttr:=0, hMatrix:=0, Unit:=2) {
    Ptr := "UPtr"
    If !ImageAttr
    {
       if !IsNumber(Matrix)
          ImageAttr := Gdip_SetImageAttributesColorMatrix(Matrix)
       else if (Matrix != 1)
          ImageAttr := Gdip_SetImageAttributesColorMatrix("1|0|0|0|0|0|1|0|0|0|0|0|1|0|0|0|0|0|" Matrix "|0|0|0|0|0|1")
    } Else usrImageAttr := 1

    if (sX="" && sY="")
       sX := sY := 0

    if (sW="" && sH="")
       Gdip_GetImageDimensions(pBitmap, sW, sH)

    if (!hMatrix && dX!="" && dY!="")
    {
       hMatrix := dhMatrix := Gdip_CreateMatrix()
       Gdip_TranslateMatrix(dhMatrix, dX, dY, 1)
    }

    CreateRectF(sourceRect, sX, sY, sW, sH)
    E := DllCall("gdiplus\GdipDrawImageFX"
      , Ptr, pGraphics
      , Ptr, pBitmap
      , Ptr, &sourceRect
      , Ptr, hMatrix ? hMatrix : 0        ; transformation matrix
      , Ptr, pEffect
      , Ptr, ImageAttr ? ImageAttr : 0
      , "Uint", Unit)            ; srcUnit
    ; r4 := GetStatus(A_LineNumber ":GdipDrawImageFX",r4)

   If dhMatrix
      Gdip_DeleteMatrix(dhMatrix)

    If (ImageAttr && usrImageAttr!=1)
       Gdip_DisposeImageAttributes(ImageAttr)
      
    Return E
}

Gdip_BitmapApplyEffect(pBitmap, pEffect, x:="", y:="", w:="", h:=0) {
; X, Y   - coordinates for the rectangle where the effect is applied
; W, H   - width and heigh for the rectangle where the effect is applied
; If X, Y, W or H are omitted , the effect is applied on the entire pBitmap 
;
; written by Marius Șucan
; many thanks to Drugwash for the help provided
  If InStr(pEffect, "err-")
     Return pEffect

  If (!x && !y && !w && !h)
  {
     Gdip_GetImageDimensions(pBitmap, Width, Height)
     CreateRectF(RectF, 0, 0, Width, Height)
  } Else CreateRectF(RectF, X, Y, W, H)

  Ptr := "UPtr"
  E := DllCall("gdiplus\GdipBitmapApplyEffect"
      , Ptr, pBitmap
      , Ptr, pEffect
      , Ptr, &RectF
      , Ptr, 0
      , Ptr, 0
      , Ptr, 0)

   Return E
}

COM_CLSIDfromString(ByRef CLSID, String) {
    VarSetCapacity(CLSID, 16, 0)
    E := DllCall("ole32\CLSIDFromString", "WStr", String, "UPtr", &CLSID)
    Return E
}

Gdip_CreateEffect(whichFX, paramA, paramB, paramC:=0) {
/*
   whichFX options:
   1 - Blur
          paramA - radius [0, 255]
          paramB - bool [0, 1]
   2 - Sharpen
          paramA - radius [0, 255]
          paramB - amount [0, 100]
   3 - ! ColorMatrix
   4 - ! ColorLUT
   5 - BrightnessContrast
          paramA - brightness [-255, 255]
          paramB - contrast [-100, 100]
   6 - HueSaturationLightness
          paramA - hue [-180, 180]
          paramB - saturation [-100, 100]
          paramC - light [-100, 100]
   7 - LevelsAdjust
          paramA - highlights [0, 100]
          paramB - midtones [-100, 100]
          paramC - shadows [0, 100]
   8 - Tint
          paramA - hue [-180, 180]
          paramB - amount [0, 100]
   9 - ColorBalance
          paramA - Cyan / Red [-100, 100]
          paramB - Magenta / Green [-100, 100]
          paramC - Yellow / Blue [-100, 100]
   10 - ! RedEyeCorrection
   11 - ColorCurve
          paramA - Type of adjustments [0, 7]
                   0 - AdjustExposure         [-255, 255]
                   1 - AdjustDensity          [-255, 255]
                   2 - AdjustContrast         [-100, 100]
                   3 - AdjustHighlight        [-100, 100]
                   4 - AdjustShadow           [-100, 100]
                   5 - AdjustMidtone          [-100, 100]
                   6 - AdjustWhiteSaturation  [0, 255]
                   7 - AdjustBlackSaturation  [0, 255]

         paramB - Apply ColorCurve on channels [1, 4]
                   1 - Red
                   2 - Green
                   3 - Blue
                   4 - All channels

         paramC - An adjust value within range according to paramA

   Effects marked with "!" are not yet implemented.
   Through ParamA, ParamB and ParamC, the effects can be controlled.
   Function written by Marius Șucan. Many thanks to Drugwash for the help provided,
*/

    Static gdipImgFX := {1:"633C80A4-1843-482b-9EF2-BE2834C5FDD4", 2:"63CBF3EE-C526-402c-8F71-62C540BF5142", 3:"718F2615-7933-40e3-A511-5F68FE14DD74", 4:"A7CE72A9-0F7F-40d7-B3CC-D0C02D5C3212", 5:"D3A1DBE1-8EC4-4c17-9F4C-EA97AD1C343D", 6:"8B2DD6C3-EB07-4d87-A5F0-7108E26A9C5F", 7:"99C354EC-2A31-4f3a-8C34-17A803B33A25", 8:"1077AF00-2848-4441-9489-44AD4C2D7A2C", 9:"537E597D-251E-48da-9664-29CA496B70F8", 10:"74D29D05-69A4-4266-9549-3CC52836B632", 11:"DD6A0022-58E4-4a67-9D9B-D48EB881A53D"}
    Ptr := A_PtrSize=8 ? "UPtr" : "UInt"
    Ptr2 := A_PtrSize=8 ? "Ptr*" : "PtrP"
    pEffect := 0
    r1 := COM_CLSIDfromString(eFXguid, "{" gdipImgFX[whichFX] "}" )
    If r1
       Return "err-" r1

    If (A_PtrSize=4) ; 32 bits
    {
       r2 := DllCall("gdiplus\GdipCreateEffect"
          , "UInt", NumGet(eFXguid, 0, "UInt")
          , "UInt", NumGet(eFXguid, 4, "UInt")
          , "UInt", NumGet(eFXguid, 8, "UInt")
          , "UInt", NumGet(eFXguid, 12, "UInt")
          , Ptr2, pEffect)
    } Else
    {
       r2 := DllCall("gdiplus\GdipCreateEffect"
          , Ptr, &eFXguid
          , Ptr2, pEffect)
    }
    If r2
       Return "err-" r2

    ; r2 := GetStatus(A_LineNumber ":GdipCreateEffect", r2)

    VarSetCapacity(FXparams, 16, 0)
    If (whichFX=1)   ; Blur FX
    {
       NumPut(paramA, FXparams, 0, "Float")   ; radius [0, 255]
       NumPut(paramB, FXparams, 4, "Uchar")   ; bool 0, 1
    } Else If (whichFX=2)   ; Sharpen FX
    {
       NumPut(paramA, FXparams, 0, "Float")   ; radius [0, 255]
       NumPut(paramB, FXparams, 4, "Float")   ; amount [0, 100]
    } Else If (whichFX=5)   ; Brightness / Contrast
    {
       NumPut(paramA, FXparams, 0, "Int")     ; brightness [-255, 255]
       NumPut(paramB, FXparams, 4, "Int")     ; contrast [-100, 100]
    } Else If (whichFX=6)   ; Hue / Saturation / Lightness
    {
       NumPut(paramA, FXparams, 0, "Int")     ; hue [-180, 180]
       NumPut(paramB, FXparams, 4, "Int")     ; saturation [-100, 100]
       NumPut(paramC, FXparams, 8, "Int")     ; light [-100, 100]
    } Else If (whichFX=7)   ; Levels adjust
    {
       NumPut(paramA, FXparams, 0, "Int")     ; highlights [0, 100]
       NumPut(paramB, FXparams, 4, "Int")     ; midtones [-100, 100]
       NumPut(paramC, FXparams, 8, "Int")     ; shadows [0, 100]
    } Else If (whichFX=8)   ; Tint adjust
    {
       NumPut(paramA, FXparams, 0, "Int")     ; hue [180, 180]
       NumPut(paramB, FXparams, 4, "Int")     ; amount [0, 100]
    } Else If (whichFX=9)   ; Colors balance
    {
       NumPut(paramA, FXparams, 0, "Int")     ; Cyan / Red [-100, 100]
       NumPut(paramB, FXparams, 4, "Int")     ; Magenta / Green [-100, 100]
       NumPut(paramC, FXparams, 8, "Int")     ; Yellow / Blue [-100, 100]
    } Else If (whichFX=11)   ; ColorCurve
    {
       NumPut(paramA, FXparams, 0, "Int")     ; Type of adjustment [0, 7]
       NumPut(paramB, FXparams, 4, "Int")     ; Channels to affect [1, 4]
       NumPut(paramC, FXparams, 8, "Int")     ; Adjustment value [based on the type of adjustment]
    }

    DllCall("gdiplus\GdipGetEffectParameterSize", Ptr, pEffect, "uint*", FXsize)
    r3 := DllCall("gdiplus\GdipSetEffectParameters", Ptr, pEffect, Ptr, &FXparams, "UInt", FXsize)
    If r3
    {
       Gdip_DisposeEffect(pEffect)
       Return "err-" r3
    }
    ; r3 := GetStatus(A_LineNumber ":GdipSetEffectParameters", r3)
    ; ToolTip, % r1 " -- " r2 " -- " r3 " -- " r4,,, 2
    Return pEffect
}

Gdip_DisposeEffect(pEffect) {
   Ptr := "UPtr"
   r := DllCall("gdiplus\GdipDeleteEffect", Ptr, pEffect)
   Return r
}

GenerateColorMatrix(modus, bright:=1, contrast:=0, saturation:=1, alph:=1, chnRdec:=0, chnGdec:=0, chnBdec:=0) {
; parameters ranges / intervals:
; bright:     [0.001 - 20.0]
; contrast:   [-20.0 - 1.00]
; saturation: [0.001 - 5.00]
; alph:       [0.001 - 5.00]
;
; modus options:
; 0 - personalized colors based on the bright, contrast [hue], saturation parameters
; 1 - personalized colors based on the bright, contrast, saturation parameters
; 2 - grayscale image
; 3 - grayscale R channel
; 4 - grayscale G channel
; 5 - grayscale B channel
; 6 - negative / invert image
; 7 - alpha channel as grayscale image
;
; chnRdec, chnGdec, chnBdec only apply in modus=1
; these represent offsets for the RGB channels

; in modus=0 the parameters have other ranges:
; bright:     [-5.00 - 5.00]
; hue:        [-1.57 - 1.57]  ; pi/2 - contrast stands for hue in this mode
; saturation: [0.001 - 5.00]
; formulas for modus=0 were written by Smurth
; extracted from https://autohotkey.com/board/topic/29449-gdi-standard-library-145-by-tic/page-86
;
; function written by Marius Șucan
; infos from http://www.graficaobscura.com/matrix/index.html
; real NTSC values: r := 0.300, g := 0.587, b := 0.115

    Static NTSCr := 0.308, NTSCg := 0.650, NTSCb := 0.095   ; personalized values
    matrix := ""

    If (modus=2)       ; grayscale
    {
       LGA := (bright<=1) ? bright/1.5 - 0.6666 : bright - 1
       Ra := NTSCr + LGA
       If (Ra<0)
          Ra := 0
       Ga := NTSCg + LGA
       If (Ga<0)
          Ga := 0
       Ba := NTSCb + LGA
       If (Ba<0)
          Ba := 0
       matrix := Ra "|" Ra "|" Ra "|0|0|" Ga "|" Ga "|" Ga "|0|0|" Ba "|" Ba "|" Ba "|0|0|0|0|0|" alph "|0|" contrast "|" contrast "|" contrast "|0|1"
    } Else If (modus=3)       ; grayscale R
    {
       Ga := 0, Ba := 0, GGA := 0
       Ra := bright
       matrix := Ra "|" Ra "|" Ra "|0|0|" Ga "|" Ga "|" Ga "|0|0|" Ba "|" Ba "|" Ba "|0|0|0|0|0|25|0|" GGA+0.01 "|" GGA "|" GGA "|0|1"
    } Else If (modus=4)       ; grayscale G
    {
       Ra := 0, Ba := 0, GGA := 0
       Ga := bright
       matrix := Ra "|" Ra "|" Ra "|0|0|" Ga "|" Ga "|" Ga "|0|0|" Ba "|" Ba "|" Ba "|0|0|0|0|0|25|0|" GGA "|" GGA+0.01 "|" GGA "|0|1"
    } Else If (modus=5)       ; grayscale B
    {
       Ra := 0, Ga := 0, GGA := 0
       Ba := bright
       matrix := Ra "|" Ra "|" Ra "|0|0|" Ga "|" Ga "|" Ga "|0|0|" Ba "|" Ba "|" Ba "|0|0|0|0|0|25|0|" GGA "|" GGA "|" GGA+0.01 "|0|1"
    } Else If (modus=6)  ; negative / invert
    {
       matrix := "-1|0|0|0|0|0|-1|0|0|0|0|0|-1|0|0|0|0|0|" alph "|0|1|1|1|0|1"
    } Else If (modus=1)   ; personalized saturation, contrast and brightness 
    {
       bL := bright, aL := alph
       G := contrast, sL := saturation
       sLi := 1 - saturation
       bLa := bright - 1
       If (sL>1)
       {
          z := (bL<1) ? bL : 1
          sL := sL*z
          If (sL<0.98)
             sL := 0.98

          y := z*(1 - sL)
          mA := z*(y*NTSCr + sL + bLa + chnRdec)
          mB := z*(y*NTSCr)
          mC := z*(y*NTSCr)
          mD := z*(y*NTSCg)
          mE := z*(y*NTSCg + sL + bLa + chnGdec)
          mF := z*(y*NTSCg)
          mG := z*(y*NTSCb)
          mH := z*(y*NTSCb)
          mI := z*(y*NTSCb + sL + bLa + chnBdec)
          mtrx:= mA "|" mB "|" mC "|  0   |0"
           . "|" mD "|" mE "|" mF "|  0   |0"
           . "|" mG "|" mH "|" mI "|  0   |0"
           . "|  0   |  0   |  0   |" aL "|0"
           . "|" G  "|" G  "|" G  "|  0   |1"
       } Else
       {
          z := (bL<1) ? bL : 1
          tR := NTSCr - 0.5 + bL/2
          tG := NTSCg - 0.5 + bL/2
          tB := NTSCb - 0.5 + bL/2
          rB := z*(tR*sLi+bL*(1 - sLi) + chnRdec)
          gB := z*(tG*sLi+bL*(1 - sLi) + chnGdec)
          bB := z*(tB*sLi+bL*(1 - sLi) + chnBdec)     ; Formula used: A*w + B*(1 – w)
          rF := z*(NTSCr*sLi + (bL/2 - 0.5)*sLi)
          gF := z*(NTSCg*sLi + (bL/2 - 0.5)*sLi)
          bF := z*(NTSCb*sLi + (bL/2 - 0.5)*sLi)

          rB := rB*z+rF*(1 - z)
          gB := gB*z+gF*(1 - z)
          bB := bB*z+bF*(1 - z)     ; Formula used: A*w + B*(1 – w)
          If (rB<0)
             rB := 0
          If (gB<0)
             gB := 0
          If (bB<0)
             bB := 0
          If (rF<0)
             rF := 0
 
          If (gF<0)
             gF := 0
 
          If (bF<0)
             bF := 0

          ; ToolTip, % rB " - " rF " --- " gB " - " gF
          mtrx:= rB "|" rF "|" rF "|  0   |0"
           . "|" gF "|" gB "|" gF "|  0   |0"
           . "|" bF "|" bF "|" bB "|  0   |0"
           . "|  0   |  0   |  0   |" aL "|0"
           . "|" G  "|" G  "|" G  "|  0   |1"
          ; matrix adjusted for lisibility
       }
       matrix := StrReplace(mtrx, A_Space)
    } Else If (modus=0)   ; personalized hue, saturation and brightness
    {
       s1 := contrast   ; in this mode, contrast stands for hue
       s2 := saturation
       s3 := bright
       aL := alph
 
       s1 := s2*sin(s1)
       sc := 1-s2
       r := NTSCr*sc-s1
       g := NTSCg*sc-s1
       b := NTSCb*sc-s1
 
       rB := r+s2+3*s1
       gB := g+s2+3*s1
       bB := b+s2+3*s1
       mtrx :=   rB "|" r  "|" r  "|  0   |0"
           . "|" g  "|" gB "|" g  "|  0   |0"
           . "|" b  "|" b  "|" bB "|  0   |0"
           . "|  0   |  0   |  0   |" aL "|0"
           . "|" s3 "|" s3 "|" s3 "|  0   |1"
       matrix := StrReplace(mtrx, A_Space)
    } Else If (modus=7)
    {
       mtrx := "0|0|0|0|0"
            . "|0|0|0|0|0"
            . "|0|0|0|0|0"
            . "|1|1|1|25|0"
            . "|0|0|0|0|1"
       matrix := StrReplace(mtrx, A_Space)
    }
    Return matrix
}

Gdip_CompareBitmaps(pBitmapA, pBitmapB, accuracy:=25) {
; On success, it returns the percentage of similarity between the given pBitmaps.
; If the given pBitmaps do not have the same resolution, 
; the return value is -1.
;
; Function by Tic, from June 2010
; Source: https://autohotkey.com/board/topic/29449-gdi-standard-library-145-by-tic/page-27
;
; Warning: it can be very slow with really large images and high accuracy.
;
; Updated and modified by Marius Șucan in September 2019.
; Added accuracy factor.

   If (accuracy>99)
      accuracy := 100
   Else If (accuracy<5)
      accuracy := 5

   Gdip_GetImageDimensions(pBitmapA, WidthA, HeightA)
   Gdip_GetImageDimensions(pBitmapB, WidthB, HeightB)
   If (accuracy!=100)
   {
      pBitmap1 := Gdip_ResizeBitmap(pBitmapA, Floor(WidthA*(accuracy/100)), Floor(HeightA*(accuracy/100)), 0, 5)
      pBitmap2 := Gdip_ResizeBitmap(pBitmapB, Floor(WidthB*(accuracy/100)), Floor(HeightB*(accuracy/100)), 0, 5)
   } Else
   {
      pBitmap1 := pBitmapA
      pBitmap2 := pBitmapB
   }

   Gdip_GetImageDimensions(pBitmap1, Width1, Height1)
   Gdip_GetImageDimensions(pBitmap2, Width2, Height2)
   if (!Width1 || !Height1 || !Width2 || !Height2
   || Width1 != Width2 || Height1 != Height2)
      Return -1

   E1 := Gdip_LockBits(pBitmap1, 0, 0, Width1, Height1, Stride1, Scan01, BitmapData1)
   E2 := Gdip_LockBits(pBitmap2, 0, 0, Width2, Height2, Stride2, Scan02, BitmapData2)
   z := 0
   Loop (Height1)
   {
      y++
      Loop (Width1)
      {
         Gdip_FromARGB(Gdip_GetLockBitPixel(Scan01, A_Index-1, y-1, Stride1), A1, R1, G1, B1)
         Gdip_FromARGB(Gdip_GetLockBitPixel(Scan02, A_Index-1, y-1, Stride2), A2, R2, G2, B2)
         z += Abs(A2-A1) + Abs(R2-R1) + Abs(G2-G1) + Abs(B2-B1)
      }
   }

   Gdip_UnlockBits(pBitmap1, BitmapData1), Gdip_UnlockBits(pBitmap2, BitmapData2)
   If (accuracy!=100)
   {
      Gdip_DisposeImage(pBitmap1)
      Gdip_DisposeImage(pBitmap2)
   }
   Return z/(Width1*Width2*3*255/100)
}

Gdip_RetrieveBitmapChannel(pBitmap, channel) {
; Channel to retrive:
; 1 - Red
; 2 - Green
; 3 - Blue
; 4 - Alpha
; On success, the function will return a pBitmap
; in 32-ARGB PixelFormat containing a grayscale
; rendition of the retrieved channel.

    If (channel="1")
       matrix := GenerateColorMatrix(3)
    Else If (channel="2")
       matrix := GenerateColorMatrix(4)
    Else If (channel="3")
       matrix := GenerateColorMatrix(5)
    Else If (channel="4")
       matrix := GenerateColorMatrix(7)
    Else Return

    Gdip_GetImageDimensions(pBitmap, imgW, imgH)
    If (!imgW || !imgH)
       Return

    pBrush := Gdip_BrushCreateSolid(0xff000000)
    newBitmap := Gdip_CreateBitmap(imgW, imgH)
    If !newBitmap
       Return

    G := Gdip_GraphicsFromImage(newBitmap)
    Gdip_SetInterpolationMode(G, 7)
    Gdip_FillRectangle(G, pBrush, 0, 0, imgW, imgH)
    Gdip_DrawImage(G, pBitmap, 0, 0, imgW, imgH, 0, 0, imgW, imgH, matrix)
    Gdip_DeleteBrush(pBrush)
    Gdip_DeleteGraphics(G)
    Return newBitmap
}

Gdip_RenderPixelsOpaque(pBitmap, pBrush:=0, alphaLevel:=0) {
; alphaLevel - from 0 [transparent] to 1 or beyond [opaque]
;
; This function is meant to make opaque partially transparent pixels.
; It returns a pointer to a new pBitmap.
;
; If pBrush is given, the background of the image is filled using it,
; otherwise, the pixels that are 100% transparent
; might remain transparent.

    Gdip_GetImageDimensions(pBitmap, imgW, imgH)
    newBitmap := Gdip_CreateBitmap(imgW, imgH)
    G := Gdip_GraphicsFromImage(newBitmap)
    Gdip_SetInterpolationMode(G, 7)
    If alphaLevel
       matrix := GenerateColorMatrix(0, 0, 0, 1, alphaLevel)
    Else
       matrix := GenerateColorMatrix(0, 0, 0, 1, 25)
    If pBrush
       Gdip_FillRectangle(G, pBrush, 0, 0, imgW, imgH)

    Gdip_DrawImage(G, pBitmap, 0, 0, imgW, imgH, 0, 0, imgW, imgH, matrix)
    Gdip_DeleteGraphics(G)
    Return newBitmap
}

Gdip_TestBitmapUniformity(pBitmap, HistogramFormat:=3, ByRef maxLevelIndex:=0, ByRef maxLevelPixels:=0) {
; This function tests whether the given pBitmap 
; is in a single shade [color] or not.

; If HistogramFormat parameter is set to 3, the function 
; retrieves the intensity/gray histogram and checks
; how many pixels are for each level [0, 255].
;
; If all pixels are found at a single level,
; the return value is 1, because the pBitmap is considered
; uniform, in a single shade.
;
; One can set the HistogramFormat to 4 [R], 5 [G], 6 [B] or 7 [A]
; to test for the uniformity of a specific channel.
;
; A threshold value of 0.0005% of all the pixels, is used.
; This is to ensure that a few pixels do not change the status.

   LevelsArray := []
   maxLevelIndex := maxLevelPixels := nrPixels := 9
   Gdip_GetImageDimensions(pBitmap, Width, Height)
   Gdip_GetHistogram(pBitmap, HistogramFormat, LevelsArray, 0, 0)
   Loop (256)
   {
       nrPixels := Round(LevelsArray[A_Index - 1])
       If (nrPixels>0)
          histoList .= nrPixels "." A_Index - 1 "|"
   }
   histoList := Sort(histoList, "NURD|")
   histoList := Trim(histoList, "|")
   histoListSortedArray := StrSplit(histoList, "|")
   maxLevel := StrSplit(histoListSortedArray[1], ".")
   maxLevelIndex := maxLevel[2]
   maxLevelPixels := maxLevel[1]
   ; ToolTip, % maxLevelIndex " -- " maxLevelPixels " | " histoListSortedArray[1] "`n" histoList, , , 3
   pixelsThreshold := Round((Width * Height) * 0.0005) + 1
   If (Floor(histoListSortedArray[2])<pixelsThreshold)
      Return 1
   Else 
      Return 0
}

Gdip_SetBitmapAlphaChannel(pBitmap, AlphaMaskBitmap) {
; Replaces the alpha channel of the given pBitmap
; based on the red channel of AlphaMaskBitmap.
; AlphaMaskBitmap must be grayscale for optimal results.
; Both pBitmap and AlphaMaskBitmap must be in 32-ARGB PixelFormat.

   Gdip_GetImageDimensions(pBitmap, Width1, Height1)
   Gdip_GetImageDimensions(AlphaMaskBitmap, Width2, Height2)
   if (!Width1 || !Height1 || !Width2 || !Height2
   || Width1 != Width2 || Height1 != Height2)
      Return -1

   newBitmap := Gdip_RenderPixelsOpaque(pBitmap)
   alphaUniform := Gdip_TestBitmapUniformity(AlphaMaskBitmap, 3, maxLevelIndex, maxLevelPixels)
   If (alphaUniform=1)
   {
      ; if the given AlphaMaskBitmap is only in a single shade,
      ; the opacity of the pixels in the given pBitmap is set
      ; using a ColorMatrix.
      newAlpha := Round(maxLevelIndex/255, 2)
      If (newAlpha<0.1)
         newAlpha := 0.1

      nBitmap := Gdip_RenderPixelsOpaque(pBitmap, 0 , newAlpha)
      Gdip_DisposeImage(newBitmap)
      Return nBitmap
   }

   E1 := Gdip_LockBits(newBitmap, 0, 0, Width1, Height1, Stride1, Scan01, BitmapData1)
   E2 := Gdip_LockBits(AlphaMaskBitmap, 0, 0, Width2, Height2, Stride2, Scan02, BitmapData2)
   Loop (Height1)
   {
      y++
      Loop (Width1)
      {
         pX := A_Index-1, pY := y-1
         R2 := Gdip_RFromARGB(NumGet(Scan02+0, (pX*4)+(pY*Stride2), "UInt"))       ; Gdip_GetLockBitPixel()
         If (R2>254)
            Continue
         Gdip_FromARGB(NumGet(Scan01+0, (pX*4)+(pY*Stride1), "UInt"), A1, R1, G1, B1)
         NumPut(Gdip_ToARGB(R2, R1, G1, B1), Scan01+0, (pX*4)+(pY*Stride1), "UInt")    ; Gdip_SetLockBitPixel()
      }
   }

   Gdip_UnlockBits(newBitmap, BitmapData1)
   Gdip_UnlockBits(AlphaMaskBitmap, BitmapData2)
   return newBitmap
}

calcIMGdimensions(imgW, imgH, givenW, givenH, ByRef ResizedW, ByRef ResizedH) {
; This function calculates from original imgW and imgH 
; new image dimensions that maintain the aspect ratio
; and are within the boundaries of givenW and givenH.
;
; imgW, imgH         - original image width and height
; givenW, givenH     - the width and height [in pixels] to adapt to
; ResizedW, ResizedH - the width and height resulted from adapting imgW, imgH to givenW, givenH
;                      by keeping the aspect ratio

   PicRatio := Round(imgW/imgH, 5)
   givenRatio := Round(givenW/givenH, 5)
   If (imgW <= givenW) && (imgH <= givenH)
   {
      ResizedW := givenW
      ResizedH := Round(ResizedW / PicRatio)
      If (ResizedH>givenH)
      {
         ResizedH := (imgH <= givenH) ? givenH : imgH
         ResizedW := Round(ResizedH * PicRatio)
      }   
   } Else If (PicRatio > givenRatio)
   {
      ResizedW := givenW
      ResizedH := Round(ResizedW / PicRatio)
   } Else
   {
      ResizedH := (imgH >= givenH) ? givenH : imgH         ;set the maximum picture height to the original height
      ResizedW := Round(ResizedH * PicRatio)
   }
}

GetWindowRect(hwnd, ByRef W, ByRef H) {
   ; function by GeekDude: https://gist.github.com/G33kDude/5b7ba418e685e52c3e6507e5c6972959
   ; W10 compatible function to find a window's visible boundaries
   ; modified by Marius Șucanto return an array
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

Gdip_BitmapConvertGray(pBitmap, hue:=0, vibrance:=-40, brightness:=1, contrast:=0, KeepPixelFormat:=0) {
; hue, vibrance, contrast and brightness parameters
; influence the resulted new grayscale pBitmap.
;
; KeepPixelFormat can receive a specific PixelFormat.
; The function returns a pointer to a new pBitmap.

    Gdip_GetImageDimensions(pBitmap, Width, Height)
    If (KeepPixelFormat=1)
       PixelFormat := Gdip_GetImagePixelFormat(pBitmap, 1)
    If StrLen(KeepPixelFormat)>3
       PixelFormat := KeepPixelFormat

    newBitmap := Gdip_CreateBitmap(Width, Height, PixelFormat)
    G := Gdip_GraphicsFromImage(newBitmap)
    Gdip_SetInterpolationMode(G, InterpolationMode)
    pEffect := Gdip_CreateEffect(6, hue, vibrance, 0)
    matrix := GenerateColorMatrix(2, brightness, contrast)
    r1 := Gdip_DrawImageFX(G, pBitmap, 0, 0, 0, 0, Width, Height, matrix, pEffect)
    Gdip_DisposeEffect(pEffect)
    Gdip_DeleteGraphics(G)
    Return newBitmap
}

Gdip_BitmapSetColorDepth(pBitmap, bitsDepth, useDithering:=1) {
; Return 0 = OK - Success

   ditheringMode := (useDithering=1) ? 9 : 1
   If (useDithering=1 && bitsDepth=16)
      ditheringMode := 2

   Colors := 2**bitsDepth
   If bitsDepth Between 2 and 4
      bitsDepth := "40s"
   If bitsDepth Between 5 and 8
      bitsDepth := "80s"
   If (bitsDepth="BW")
      E := Gdip_BitmapConvertFormat(pBitmap, 0x30101, ditheringMode, 2, 2, 2, 2, 0, 0)
   Else If (bitsDepth=1)
      E := Gdip_BitmapConvertFormat(pBitmap, 0x30101, ditheringMode, 1, 2, 1, 2, 0, 0)
   Else If (bitsDepth="40s")
      E := Gdip_BitmapConvertFormat(pBitmap, 0x30402, ditheringMode, 1, Colors, 1, Colors, 0, 0)
   Else If (bitsDepth="80s")
      E := Gdip_BitmapConvertFormat(pBitmap, 0x30803, ditheringMode, 1, Colors, 1, Colors, 0, 0)
   Else If (bitsDepth=16)
      E := Gdip_BitmapConvertFormat(pBitmap, 0x21005, ditheringMode, 1, Colors, 1, Colors, 0, 0)
   Else If (bitsDepth=24)
      E := Gdip_BitmapConvertFormat(pBitmap, 0x21808, 2, 1, 0, 0, 0, 0, 0)
   Else If (bitsDepth=32)
      E := Gdip_BitmapConvertFormat(pBitmap, 0x26200A, 2, 1, 0, 0, 0, 0, 0)
   Else
      E := -1
   Return E
}

Gdip_BitmapConvertFormat(pBitmap, PixelFormat, DitherType, DitherPaletteType, PaletteEntries, PaletteType, OptimalColors, UseTransparentColor:=0, AlphaThresholdPercent:=0) {
; pBitmap - Handle to a pBitmap object on which the color conversion is applied.

; PixelFormat options: see Gdip_GetImagePixelFormat()
; Pixel format constant that specifies the new pixel format.

; PaletteEntries    Number of Entries.
; OptimalColors   - Integer that specifies the number of colors you want to have in an optimal palette based on a specified pBitmap.
;                   This parameter is relevant if PaletteType parameter is set to PaletteTypeOptimal [1].
; UseTransparentColor     Boolean value that specifies whether to include the transparent color in the palette.
; AlphaThresholdPercent - Real number in the range 0.0 through 100.0 that specifies which pixels in the source bitmap will map to the transparent color in the converted bitmap.
;
; PaletteType options:
; Custom = 0   ; Arbitrary custom palette provided by caller.
; Optimal = 1   ; Optimal palette generated using a median-cut algorithm.
; FixedBW = 2   ; Black and white palette.
;
; Symmetric halftone palettes. Each of these halftone palettes will be a superset of the system palette.
; e.g. Halftone8 will have its 8-color on-off primaries and the 16 system colors added. With duplicates removed, that leaves 16 colors.
; FixedHalftone8 = 3   ; 8-color, on-off primaries
; FixedHalftone27 = 4   ; 3 intensity levels of each color
; FixedHalftone64 = 5   ; 4 intensity levels of each color
; FixedHalftone125 = 6   ; 5 intensity levels of each color
; FixedHalftone216 = 7   ; 6 intensity levels of each color
;
; Assymetric halftone palettes. These are somewhat less useful than the symmetric ones, but are included for completeness.
; These do not include all of the system colors.
; FixedHalftone252 = 8   ; 6-red, 7-green, 6-blue intensities
; FixedHalftone256 = 9   ; 8-red, 8-green, 4-blue intensities
;
; DitherType options:
; None = 0
; Solid = 1
; - it picks the nearest matching color with no attempt to halftone or dither. May be used on an arbitrary palette.
;
; Ordered dithers and spiral dithers must be used with a fixed palette.
; NOTE: DitherOrdered4x4 is unique in that it may apply to 16bpp conversions also.
; Ordered4x4 = 2
; Ordered8x8 = 3
; Ordered16x16 = 4
; Ordered91x91 = 5
; Spiral4x4 = 6
; Spiral8x8 = 7
; DualSpiral4x4 = 8
; DualSpiral8x8 = 9
; ErrorDiffusion = 10   ; may be used with any palette
; Return 0 = OK - Success

   VarSetCapacity(hPalette, 4 * PaletteEntries + 8, 0)

;   tPalette := DllStructCreate("uint Flags; uint Count; uint ARGB[" & $iEntries & "];")
   NumPut(PaletteType, &hPalette, 0, "uint")
   NumPut(PaletteEntries, &hPalette, 4, "uint")
   NumPut(0, &hPalette, 8, "uint")

   Ptr := "UPtr"
   E1 := DllCall("gdiplus\GdipInitializePalette", "UPtr", &hPalette, "uint", PaletteType, "uint", OptimalColors, "Int", UseTransparentColor, Ptr, pBitmap)
   E2 := DllCall("gdiplus\GdipBitmapConvertFormat", Ptr, pBitmap, "uint", PixelFormat, "uint", DitherType, "uint", DitherPaletteType, "uPtr", &hPalette, "float", AlphaThresholdPercent)
   E := E1 ? E1 : E2
   Return E
}

Gdip_GetImageThumbnail(pBitmap, W, H) {
; by jballi, source
; https://www.autohotkey.com/boards/viewtopic.php?style=7&t=70508

    DllCall("gdiplus\GdipGetImageThumbnail"
        ,"UPtr",pBitmap                         ;-- *image
        ,"UInt",W                               ;-- thumbWidth
        ,"UInt",H                               ;-- thumbHeight
        ,"UPtr*",pThumbnail                     ;-- **thumbImage
        ,"UPtr",0                               ;-- callback
        ,"UPtr",0)                              ;-- callbackData

   Return pThumbnail
}

; =================================================
; The following functions were written by Tidbit
; handed to me by himself to be included here.
; =================================================

ConvertRGBtoHSL(R, G, B) {
; http://www.easyrgb.com/index.php?X=MATH&H=18#text18
   R := (R / 255)
   G := (G / 255)
   B := (B / 255)

   Min     := min(R, G, B)
   Max     := max(R, G, B)
   del_Max := Max - Min

   L := (Max + Min) / 2

   if (del_Max = 0)
   {
      H := S := 0
   } else
   {
      if (L < 0.5)
         S := del_Max / (Max + Min)
      else
         S := del_Max / (2 - Max - Min)

      del_R := (((Max - R) / 6) + (del_Max / 2)) / del_Max
      del_G := (((Max - G) / 6) + (del_Max / 2)) / del_Max
      del_B := (((Max - B) / 6) + (del_Max / 2)) / del_Max

      if (R = Max)
      {
         H := del_B - del_G
      } else
      {
         if (G = Max)
            H := (1 / 3) + del_R - del_B
         else if (B = Max)
            H := (2 / 3) + del_G - del_R
      }
      if (H < 0)
         H += 1
      if (H > 1)
         H -= 1
   }
   ; return round(h*360) "," s "," l
   ; return (h*360) "," s "," l
   return [abs(round(h*360)), abs(s), abs(l)]
}

ConvertHSLtoRGB(H, S, L) {
; http://www.had2know.com/technology/hsl-rgb-color-converter.html

   H := H/360
   if (S == 0)
   {
      R := L*255
      G := L*255
      B := L*255
   } else
   {
      if (L < 0.5)
         var_2 := L * (1 + S)
      else
         var_2 := (L + S) - (S * L)
       var_1 := 2 * L - var_2

       R := 255 * ConvertHueToRGB(var_1, var_2, H + (1 / 3))
       G := 255 * ConvertHueToRGB(var_1, var_2, H)
       B := 255 * ConvertHueToRGB(var_1, var_2, H - (1 / 3))
   }
   ; Return round(R) "," round(G) "," round(B)
   ; Return (R) "," (G) "," (B)
   Return [round(R), round(G), round(B)]
}

ConvertHueToRGB(v1, v2, vH) {
   vH := ((vH<0) ? ++vH : vH)
   vH := ((vH>1) ? --vH : vH)
   return  ((6 * vH) < 1) ? (v1 + (v2 - v1) * 6 * vH)
         : ((2 * vH) < 1) ? (v2)
         : ((3 * vH) < 2) ? (v1 + (v2 - v1) * ((2 / 3) - vH) * 6)
         : v1
}

Gdip_ErrrorHandler(errCode, throwErrorMsg, additionalInfo:="") {
   Static errList := {1:"Generic_Error", 2:"Invalid_Parameter"
         , 3:"Out_Of_Memory", 4:"Object_Busy"
         , 5:"Insufficient_Buffer", 6:"Not_Implemented"
         , 7:"Win32_Error", 8:"Wrong_State"
         , 9:"Aborted", 10:"File_Not_Found"
         , 11:"Value_Overflow", 12:"Access_Denied"
         , 13:"Unknown_Image_Format", 14:"Font_Family_Not_Found"
         , 15:"Font_Style_Not_Found", 16:"Not_TrueType_Font"
         , 17:"Unsupported_GdiPlus_Version", 18:"Not_Initialized"
         , 19:"Property_Not_Found", 20:"Property_Not_Supported"
         , 21:"Profile_Not_Found", 100:"Unknown_Wrapper_Error"}

   If !errCode
      Return

   aerrCode := (errCode<0) ? 100 : errCode
   If errList.HasKey(aerrCode)
      GdipErrMsg := "GDI+ ERROR: " errList[aerrCode]  " [CODE: " aerrCode "]" additionalInfo
   Else
      GdipErrMsg := "GDI+ UNKNOWN ERROR: " aerrCode additionalInfo

   If (throwErrorMsg=1)
      MsgBox(GdipErrMsg, "GDI+ ERROR")

   Return GdipErrMsg
}
