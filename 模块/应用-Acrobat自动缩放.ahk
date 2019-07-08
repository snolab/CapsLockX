
SetTitleMatchMode RegEx
SetDefaultMouseSpeed 0
Return
#IfWinActive .*- Adobe Acrobat Reader DC ahk_class ahk_class AcrobatSDIWindow
    ^!F12:: ExitApp ; 退出脚本

    `:: Send ^h^3

;debug
;F12:: ExitApp
