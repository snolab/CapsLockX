Return

; 需要在控制台窗口右键属性选启用Ctrl+Shift+C/V才管用

#IfWinActive ahk_class ConsoleWindowClass ; 控制台窗口内
^v:: Send ^+v
