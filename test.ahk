!`::MsgBox, 64, Cursor Check, % "Cursor: " CursorCheck() A_Space "(" A_Cursor ")"



CursorCheck()

{

; Credit: shimanov, http://www.autohotkey.com/forum/post-47747.html#47747

 VarSetCapacity(ci, 20, 0)

 ci := Chr(20)

 s := DllCall("GetCursorInfo", "uint", ci)

 f := DllCall("GetLastError", "uint")
 MsgBox, % s " "  f
 ;ErrorLevel := mod(ErrorLevel + 1, 2)

 h_cursor := *(&ci+8)+(*(&ci+9) << 8)+(*(&ci+10) << 16)+(*(&ci+11) << 24)

 return h_cursor

}