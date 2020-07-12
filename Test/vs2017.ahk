
SetTitleMatchMode, RegEx
#IfWinActive .*- Microsoft Visual Studio ahk_class HwndWrapper\[DefaultDomain.*\]
    ^F12:: ExitApp
    F4:: Send ^+{F12}