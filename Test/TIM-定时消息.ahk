SetTitleMatchMode RegEx

FaXiaoXi(title, msg){
    win := title . " ahk_class TXGuiFoundation ahk_exe TIM.exe"
    ControlSend, , %msg%, %win%
    Sleep, 64
    ControlSend, , ^{Enter}, %win%
}

;qun := "杠把子.*"
;qun := "Snowstar"
qun := "20146.*"
FaXiaoXi(qun, "真是一科比一科难了……")
Sleep, 10000
FaXiaoXi(qun, "英语……大家加油吧！")
Sleep, 10000
FaXiaoXi(qun, "哈哈哈哈哈……")
Sleep, 10000
FaXiaoXi(qun, "哈哈…")
Sleep, 10000
FaXiaoXi(qun, "哈…")
Sleep, 10000
FaXiaoXi(qun, "…")
Sleep, 20000
FaXiaoXi(qun, "呜呜")
Sleep, 120000
FaXiaoXi(qun, "嗯")
; FaXiaoXi(qun, "嗯，上面这堆是自动发的……（这句也是")
;WinActive("杠把子.* ahk_class TXGuiFoundation ahk_exe TIM.exe")

;WinActivate %win%
;WinWaitActive %win%

Esc:: Exitapp