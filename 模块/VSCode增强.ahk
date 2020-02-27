; save as utf8 with bom
Return	 
#IfWinActive ahk_class Chrome_WidgetWin_1 ahk_exe Code.exe
![::Send ^k^[
!]::Send ^k^]
