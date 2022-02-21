; 关于普通正则表达式的例子, 请参阅 正则表达式快速参考.
SetTitleMatchMode RegEx
SetKeyDelay, 0, 0
;#UseHook On

Return

#IfWinActive .*- OneNote ahk_class ApplicationFrameWindow ahk_exe ApplicationFrameHost.exe ; OneNote_UWP_笔记本内

$F2::SendEvent ^+t

$+F2::SendEvent {AppsKey}{Down}{Enter}

; 待定未定
$!`::SendEvent !r

; view
$!n::SendEvent !w{Down}{Tab 7}{Enter}{Up 2}{Enter}

; 套锁、空间、橡皮
$!q::SendEvent !d{Down}{Tab 1}{Enter}
$!w::SendEvent !d{Down}{Tab 2}{Enter}
$!e::SendEvent !d{Down}{Tab 3}{Enter}

; 换笔
$!a::SendEvent !d{Down}{Tab 5}+{Left}{Enter}
$!s::SendEvent !d{Down}{Tab 5}{Enter}
$!d::SendEvent !d{Down}{Tab 5}+{Right}{Enter}

; 颜色
$+!1::SendEvent !d{Down}{Right 4}{Enter}
$+!2::SendEvent !d{Down}{Right 5}{Enter}
$+!3::SendEvent !d{Down}{Right 6}{Enter}
$+!4::SendEvent !d{Down}{Right 7}{Enter}
$+!5::SendEvent !d{Down}{Right 8}{Enter}
$+!6::SendEvent !d{Down}{Right 9}{Enter}

; 展开
$!1:: SendEvent !+1
$!2:: SendEvent !+2
$!3:: SendEvent !+3
$!4:: SendEvent !+4
$!5:: SendEvent !+5
$!6:: SendEvent !+6

; 拍照
$!p::
    SendEvent !n{Down}{Tab 4}{Enter}
    Sleep, 200
    SendEvent {Down 1}{Enter}
    Sleep, 2000
    SendEvent +{Tab 3}
Return

; 画笔粗细
$!t::SendEvent !d{Down}{Tab 13}{Enter}
$!g::SendEvent !d{Down}{Tab 11}{Enter}

; 调整缩放
$![::SendEvent !w{Down}{Tab 3}{Enter}
$!]::SendEvent !w{Down}{Tab 4}{Enter}
$!\::SendEvent !w{Down}{Tab 5}{Enter}

; 调整字体
$^[::SendEvent !h{Down}{Tab 1}{Up   2}{Enter}
$^]::SendEvent !h{Down}{Tab 1}{Down 2}{Enter}
$^\::SendEvent !h{Down}+{Tab 1}{Enter}

$!Delete:: SendEvent ^+a{Delete}