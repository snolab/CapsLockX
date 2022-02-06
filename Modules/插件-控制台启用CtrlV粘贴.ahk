Return

; 需要在控制台窗口右键属性选启用Ctrl+Shift+C/V才管用

#IfWinActive ahk_class ConsoleWindowClass
^v:: Send ^+v
