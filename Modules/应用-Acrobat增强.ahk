
SetTitleMatchMode RegEx
SetDefaultMouseSpeed 0
Return

#if WinActive(".*- Adobe Acrobat (Pro|Reader) DC ahk_class ahk_class AcrobatSDIWindow") ; AdobeAcrobat窗口内

^!F12:: ExitApp        ; 退出脚本
!a:: Send !vpy         ; 自动滚动
!d:: Send !vpt^h^3     ; 双页滚动视图（Double）
!s:: Send !vpc^h^3     ; 单页滚动视图（Single）
f11:: ^l               ; 全屏
`:: Send ^h^3          ; 自动缩放
