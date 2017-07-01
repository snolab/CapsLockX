SetTitleMatchMode RegEx

#IfWinActive Mona_.* ahk_class Qt5QWindowIcon ahk_exe Mona.exe
    $~`:: Send {MButton Down}
    $~` Up:: Send {MButton Up}
    