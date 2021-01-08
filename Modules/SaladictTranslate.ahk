#SingleInstance, Force
if (!CapsLockX)
    ExitApp
Return

#If (CapsLockXMode)
^c::
    Clipboard =
    SendEvent ^c
    ClipWait
    ; Saladict
    SendEvent !l
    Return
