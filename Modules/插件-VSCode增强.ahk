; save as utf8 with bom
Return	 

#If WinActive("ahk_class Chrome_WidgetWin_1 ahk_exe Code.exe")

; ![:: Send ^k^[
; !]:: Send ^k^]
!1:: SendEvent ^k^[{Left}
!2:: SendEvent ^k^]
; !3:: Send ^k^j^k^3
; !4:: Send ^k^j^k^4
; !5:: Send ^k^j^k^5
; !6:: Send ^k^j^k^6
; !7:: Send ^k^j^k^7
; !8:: Send ^k^j^k^8
; !9:: Send ^k^j^k^9
; !0:: Send ^k^j^k^0