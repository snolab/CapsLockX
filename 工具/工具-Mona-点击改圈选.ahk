SetTitleMatchMode RegEx

#IfWinActive Mona_.* ahk_class Qt5QWindowIcon ahk_exe Mona.exe
    $LButton:: Send {LButton}
    $LButton Up:: Send {LButton}
