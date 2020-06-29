#SingleInstance, Force
if (!CapsLockX)
    ExitApp
Return

#If (CapsLockXMode)
t::
    Clipboard =
    SendEvent ^c
    ClipWait
    SendEvent !l
    Return