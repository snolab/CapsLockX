return
; clipStack = []
; clipStack.Push([1,2,3])
; return


; *^c::
;     ; ClipboardAll=
;     ; ClipWait, [ SecondsToWait, 1]
;     clip:= ClipboardAll

;     tooltip %clip%
;     clipStack.Push(clip)
;     ; ToolTip, [ Text, X, Y, WhichToolTip]
;     return
; *^v::
;     clip1:= ClipboardAll
;     clip:= clipStack.Pop()
;     tooltip %clip1% - %clip%
;     ; ClipboardAll=
;     ; ClipWait, [ SecondsToWait, 1]
;     ; clip:= ClipboardAll
;     return