Return

#IfWinActive ahk_class XLMAIN ahk_exe EXCEL.EXE
    `::
        SendEvent {Right}^+{Down}^q
        Sleep, 200
        SendEvent {Tab}{Enter}
        Sleep, 200
        Return