Return

#IfWinActive ahk_class Chrome_WidgetWin_1 ahk_exe chrome.exe
    ; 不知道是干什么用的...
    !F3::
        Send ^f{Esc}^{Enter}{F3}
        Return

    ; 批量标签内搜索
    ^F3::
        Send ^{Tab}^f{Enter}
        Return