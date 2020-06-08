
If(!CapslockX)
    ExitApp
Return

; in the FN Mode
#If CapslockXMode

/::
    WinGetActiveTitle, title
    Run https://github.com/snomiao/CapslockX#readme
Return
