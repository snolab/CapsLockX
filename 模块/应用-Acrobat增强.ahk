
SetTitleMatchMode RegEx
SetDefaultMouseSpeed 0
Return
#IfWinActive .*- Adobe Acrobat (Pro|Reader) DC ahk_class ahk_class AcrobatSDIWindow
    ^!F12:: ExitApp ; 退出脚本

    ; 自动滚动
    !a:: Send !vpy

    ; 双页滚动视图（Double）
    !d:: Send !vpt^h^3

    ; 单页滚动视图（Single）
    !s:: Send !vpc^h^3
    f11:: ^l

;debug
;F12:: ExitApp
