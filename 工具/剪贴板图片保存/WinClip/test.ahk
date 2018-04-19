#NoEnv  ; Recommended for performance and compatibility with future AutoHotkey releases.
SendMode Input  ; Recommended for new scripts due to its superior speed and reliability.
SetWorkingDir %A_ScriptDir%  ; Ensures a consistent starting directory.

#Include WinClipAPI.ahk
#Include WinClip.ahk
wc := new WinClip
return

!d::
WinClip.Copy()
t := WinClip.GetText()
loop,2 
  t := RegExReplace( t, "m)(`t+|^)([\.\d]+)(`t+|$)", "`t<td>$2</td>`t" )
loop,2
  t := RegExReplace( t, "m)`t+([^<>`n`r]+?)(`t+|$)", "`t<td rowspan=""4"">$1</td>`t" )
t := RegExReplace( t, "`t" )
t := RegExReplace( t, "(`r`n)(?=.+)", "</tr>`n<tr align=""center"">" )
t := RegExReplace( t, "^", "<tr align=""center"">" )
StringTrimRight, t, t, 2
t := RegExReplace( t, "$", "</tr>" )
WinClip.SetText( t )
return

msgbox copy some text on clipboard
msgbox % text := WinClip.GetText()

msgbox copy some SMALL text piece from browser
msgbox % html := WinClip.GetHTML()

msgbox copy some picture
hBitmap := WinClip.GetBitmap()
Gui, Add, Picture,% "hwndPicHwnd +" SS_BITMAP := 0xE
SendMessage,% STM_SETIMAGE := 0x0172,% IMAGE_BITMAP := 0,% hBitmap,, ahk_id %PicHwnd%
DllCall("DeleteObject", "Ptr", hBitmap )
Gui, Show, w1000 h700

msgbox copy few files from explorer window
msgbox % fileslist := WinClip.GetFiles()

inputbox, textData, Input Something,Enter text you want to put on clipboard,,200,150
if textData
  WinClip.SetText( textData )
msgbox % text := WinClip.GetText()

inputbox, textData, Input Something,Enter text you want to APPEND to the one currently on clipboard,,200,150
if textData
  WinClip.AppendText( textData )
msgbox % text := WinClip.GetText()

inputbox, picPath, Input path to Picture,Enter path to the picture you want to place on clipboard ( without quotes ),,200,150
if picPath
{
  WinClip.SetBitmap( picPath )
  msgbox the picture should be on clipboard now
}

msgbox copy some data to clipboard
msgbox % "All clipboard data has been saved to clip.txt`nSize: " WinClip.Save( "clip.txt" )

WinClip.Clear()
msgbox clipboard should now be empty

bytes := WinClip.Load( "clip.txt" )
msgbox % bytes " bytes loaded from 'clip.txt' file to clipboard"

inputbox, html, Html to clipboard,Enter some text you want to place on clipboard as HTML`nfor example:`n<a href="www.hello.com">link</a>,,300,200
if html
{
  WinClip.Clear()
  WinClip.SetHTML( html )
  msgbox html data should be on clipboard
}

; see docs for other examples here:
; http://www.autohotkey.net/~Deo/index.html