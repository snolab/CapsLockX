; ========== CapsLockX ==========
; 名称：OneNote 2016 增强
; 版本：v2020.06.27
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版权：Copyright © 2018-2020 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========
; 
; save as utf8 with bom

If(!CapsLockX)
    ExitApp
Return


; 定义应用顶栏用到的函数
altSend(altKeys){
    SetKeyDelay, 1, 60 ; 配置纠错
    SendEvent {AltDown}%altKeys%{AltUp}
}

altSendEx(altKeys, suffix){
    SetKeyDelay, 1, 60 ; 配置纠错
    SendEvent {AltDown}%altKeys%{AltUp}%suffix%
}

StrJoin(sep, params*) {
    for index, param in params
        str .= param . sep
    return SubStr(str, 2, -StrLen(sep))
}
GetFocusControlName(){
    ControlGetFocus, name, A
    return name
}

; 获取与IME无冲的编码字符串，用于 Send （SEO： SendRaw SendInput）
getAscStr(str)
{
    charList := StrSplit(str)
    for key, val in charList
        out .= "{Asc " . asc(val) . "}"
    return out
}

; 快速添加事项清单
OpenToDoList(){
    if !WinExist("TODO - OneNote ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE")
        Run "onenote:#TODO"
    WinWait TODO - OneNote ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE
    WinActivate ; Uses the last found window.
    Send ^{End}{Enter}
    Return
}

; 原热键，打开快速笔记
; $#n:: Send #n

; 打开 TODO
$#!n:: OpenToDoList()
; 打开 UWP 版 OneNote
$#+!n:: Run "onenote-cmd://quicknote?onOpen=typing"

; #If !!(CapsLockXMode & CM_FN)
; h:: Run "https://support.office.com/zh-cn/article/OneNote-2013-%25E4%25B8%25AD%25E7%259A%2584%25E9%2594%25AE%25E7%259B%2598%25E5%25BF%25AB%25E6%258D%25B7%25E6%2596%25B9%25E5%25BC%258F-65dc79fa-de36-4ca0-9a6e-dfe7f3452ff8?ui=zh-CN&rs=zh-CN&ad=CN&fromAR=1"
; 和编辑增强冲突

#If (!CapsLockX)
    ^+!F12:: ExitApp ; 退出脚本

; #If CapsLockXMode and (WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE") or WinActive("ahk_class ahk_class OneNote`:`:NavigationUIPopup ahk_exe ONENOTE.EXE"))
#If (WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE") || WinActive("ahk_class ahk_class OneNote`:`:NavigationUIPopup ahk_exe ONENOTE.EXE"))
    
; $^f::
; $^e::
; enhanced_search() {
; ; 增强搜索
; if (A_ThisHotkey = "$^e"){
; Send ^e
; }else{
; Send ^f
; }
; If ("RICHEDIT60W1" != GetFocusControlName()){
; Return
; }
; Clipboard =
; Send ^c
; ClipWait, 1

; If ErrorLevel {
; Return
; }

; s1 := Clipboard

; a := StrSplit(s1)
; s2 := StrJoin(" ", a*)
; str := """" s1 """" " OR " """" s2 """"

; Clipboard := str
; Send ^v
; Return
; }

#If CapsLockXMode == CM_CapsLockX || CapsLockXMode == CM_FN && WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE")
    
; 上下左右
; 不知为啥这个kj在OneNote里有时候会不管用, 于是就设定了特殊的编辑操作
k:: Send {Home}{Left}
j:: Send {End}{Right}

#IfWinActive .*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE
/:: CapsLockShowHelp("
(
OneNote 笔记界面（由于太多）故
!-  | 自动2维化公式
^+c | 复制纯文本
^+v | 粘贴纯文本
F2  | 重命名笔记
+F2 | 重命名分区
!m  | 移动笔记
!+m | 移动分区
^n  | 新建笔记
^!n | 
!f | 搜索标记
; 选择页面
    $^+PgUp:: Send ^+g{Up}{Enter}
    $^+PgDn::
^s:: ; 同步此笔记本
    
!n |  切换为无色背景
+!n::

!Delete ; 快速删除当前页面
+Delete  ; 快速删除当前行
^w  | 快速关闭窗口
; 选中行
    $^+l
; 输入、套锁、橡皮
    $!q
    $!w
    $!e
    ; 输入、套锁、橡皮
    $!s
    ; $!d
; 视图 - 缩放到页面宽度
    $!r
    $!y
; 视图 - 缩放到1
; $!+r
; $!+r

; 换笔
$!a ; 换到第2支笔
$!d ; 打开换笔盘，定位到第一支笔（只在非全屏时管用）


!1 ... !7 | 大纲折叠展开
; 自定义颜色
    $!`
    $!+`
    $!v
; 画笔粗细
    $!t
    $!g
^!+- 和 ^!+= ; 调整页面缩放
^[ 和 ^] | 调整字体大小
    ^[
    ^]
    )")

    ; 自动2维化公式
    $!-::
        Send !=
        Sleep, 200
        altSend("jp")
    return
    ; Send !={AppsKey}p
    ; 复制纯文本
    $^+c::
        Clipboard =
        Send ^c
        ClipWait, 1
        Clipboard := Clipboard
    return
    ; 粘贴纯文本
    $^+v::
        Clipboard := Clipboard
        Send ^v
    return
    
    ; ; 选择页面
    ; ^PgUp:: Send ^{PgUp}^+a
    ; ^PgDn:: Send ^{PgDn}^+a
    ; ; 将此页面向上合并
    ; $!j::
    ;     Send ^+t^a^x
    ;     Send ^{PgUp}^+t^{End}^v
    ;     Send ^{PgDn}^+a{Del}
    ;     Send ^{PgUp}
    ; Return
    
    $!c::
        SetKeyDelay, 1, 1 ; 配置纠错
        SendEvent, {AltDown}wi{AltUp}
    Return

    ; $!+x::
    ;     Clipboard := ""
    ;     SendEvent {AppsKey}y
    ;     ClipWait 1
    ;     Clipboard := RegExReplace(Clipboard, "([一-龥]) ", "$1")
    ;     SendEvent ^!{Right 3}^v
    ; Return
    
    ; 重命名笔记
    $F2:: Send ^+t

    ; 重命名分区
    $+F2:: Send ^+g{AppsKey}r
    
    ; 移动笔记
    $!m:: SendEvent ^!m
    
    ; 移动分区
    $!+m:: Send ^+g{AppsKey}m
    
    ; 交换新建笔记热键
    $^n:: Send ^!n
    $^!n:: Send ^n
    
    ; 移动页面
    ; $!+Up:: Send ^+a!+{Up}
    ; $!+Down:: Send ^+a!+{Down}
    ; $!+Left:: Send ^+a!+{Left}
    ; $!+Right:: Send ^+a!+{Right}
    
    ; 搜索标记
    $!f:: altSend("hg")
    
    ; 选择页面
    $^+PgUp:: Send ^+g{Up}{Enter}
    $^+PgDn:: Send ^+g{Down}{Enter}
    
    ; 同步此笔记本
    ; $^s:: Send +{F9}
    
    ; 切换为无色背景
    $!n:: altSend("wpcn")
    $+!n:: altSend("wre")
    
    ; 快速删除当前页面
    ;$!Delete:: altSend("pd")
    $!Delete:: Send ^+a{Delete}
    
    ; 快速删除当前行
    ;$!Delete:: altSend("pd")
    ; $+Delete:: Send ^!{Down}^!{Up}{Delete}
    $+Delete:: Send {Escape}^a{Del}
    
    ; 快速关闭窗口
    $^w:: altSend("{F4}")
    
    ; 选中行
    $^+l:: Send !+{Down}!+{Up}
    ; 输入、套锁、橡皮
    ; $!q:: altSend("dl")
    $!q:: altSend("dh") ;换成手形Tools
    $!w:: altSend("dn")
    $!e:: altSend("dek")
    
    ; 输入、套锁、橡皮
    $!s:: altSend("dt")
    ; $!d:: altSend("dh")
    
    ; 视图 - 缩放到页面宽度
    $!r:: altSend("w1")
    $!y:: altSend("wi")
    ; 视图 - 缩放到1
    ; $!+r:: altSend("w1")
    ; $!+r::
    ; 	SendEvent !w
    ; 	Sleep, 60
    ; 	SendEvent !1
    ; 	Return
    
    ; 换笔
    $!d:: altSendEx("dp", "{Home}") ; 打开换笔盘，定位到第一支笔
    $!a:: altSendEx("dp", "{Right 1}{Enter}")   ; 笔悬停时是下一支笔，没有笔时是选红色笔
    
    ; 换笔（只在非全屏时管用）
    ; $!a::
    ; 	Send {Alt}
    ; 	Sleep 60
    ; 	SendEvent dp{Left}{Enter}
    ; 	Return
    
    ; $!d::
    ; 	Send {Alt}
    ; 	Sleep 60
    ; 	SendEvent dp{Right}{Enter}
    ; 	Return
    
    ; 大纲折叠展开
    $!1:: SendEvent !+1
    $!2:: SendEvent !+2
    $!3:: SendEvent !+3
    $!4:: SendEvent !+4
    $!5:: SendEvent !+5
    $!6:: SendEvent !+6
    $!7:: SendEvent !+7
    
    ; 笔收藏夹第一排
    ; $!1:: altSendEx("dp", "{Home}{Right 0}{Enter}")
    ; $!2:: altSendEx("dp", "{Home}{Right 1}{Enter}")
    ; $!3:: altSendEx("dp", "{Home}{Right 2}{Enter}")
    ; $!4:: altSendEx("dp", "{Home}{Right 3}{Enter}")
    ; $!5:: altSendEx("dp", "{Home}{Right 4}{Enter}")
    ; $!6:: altSendEx("dp", "{Home}{Right 5}{Enter}")
    ; $!7:: altSendEx("dp", "{Home}{Right 6}{Enter}")
    
    ; 收藏夹第二排
    ; $!+1:: altSendEx("dp", "{Home}{Down 1}{Right 0}{Enter}")
    ; $!+2:: altSendEx("dp", "{Home}{Down 1}{Right 1}{Enter}")
    ; $!+3:: altSendEx("dp", "{Home}{Down 1}{Right 2}{Enter}")
    ; $!+4:: altSendEx("dp", "{Home}{Down 1}{Right 3}{Enter}")
    ; $!+5:: altSendEx("dp", "{Home}{Down 1}{Right 4}{Enter}")
    ; $!+6:: altSendEx("dp", "{Home}{Down 1}{Right 5}{Enter}")
    ; $!+7:: altSendEx("dp", "{Home}{Down 1}{Right 6}{Enter}")
    
    ; 自定义颜色
    $!`:: altSendEx("dp", "{Down 2}{Left}")
    $!+`:: altSend("dc")
    $!v:: SendEvent !h!i

    ; 画笔粗细
    $!t:: altSendEx("d", "{Down}{Tab 13}{Enter}")
    $!g:: altSendEx("d", "{Down}{Tab 11}{Enter}")
    
    ; 调整缩放
    $![:: altSendEx("w", "{Down}{Tab 3}{Enter}")
    $!]:: altSendEx("w", "{Down}{Tab 4}{Enter}")
    $!\:: altSendEx("w", "{Down}{Tab 5}{Enter}")
    
    ; 调整字体
    $^[:: altSendEx("h", "{Down}{Tab 1}{Up 2}{Enter}")
    $^]:: altSendEx("h", "{Down}{Tab 1}{Down 2}{Enter}")
    $^\:: altSendEx("h", "{Down}+{Tab 1}{Enter}")

#IfWinActive ahk_class Net UI Tool Window ahk_exe ONENOTE.EXE
    /:: CapsLockShowHelp("
    (
    换笔盘界面
    1::  ; 换到第 1 行的 1 支笔
    2::  ; 换到第 1 行的 2 支笔
    3::  ; 换到第 1 行的 3 支笔
    4::  ; 换到第 1 行的 4 支笔
    5::  ; 换到第 1 行的 5 支笔
    6::  ; 换到第 1 行的 6 支笔
    7::  ; 换到第 1 行的 7 支笔
    !1:: ; 换到第 2 行的 1 支笔
    !2:: ; 换到第 2 行的 2 支笔
    !3:: ; 换到第 2 行的 3 支笔
    !4:: ; 换到第 2 行的 4 支笔
    !5:: ; 换到第 2 行的 5 支笔
    !6:: ; 换到第 2 行的 6 支笔
    !7:: ; 换到第 2 行的 7 支笔
    )")

    $1::  ; 换到第 1 行的 1 支笔
        if(A_PriorHotkey=="!d")
            Send {Right 0}{Enter}
        Return
    $2::  ; 换到第 1 行的 2 支笔
        if(A_PriorHotkey="!d")
            Send {Right 1}{Enter}
        Return
    $3::  ; 换到第 1 行的 3 支笔
        if(A_PriorHotkey="!d")
            Send {Right 2}{Enter}
        Return
    $4::  ; 换到第 1 行的 4 支笔
        if(A_PriorHotkey="!d")
            Send {Right 3}{Enter}
        Return
    $5::  ; 换到第 1 行的 5 支笔
        if(A_PriorHotkey="!d")
            Send {Right 4}{Enter}
        Return
    $6::  ; 换到第 1 行的 6 支笔
        if(A_PriorHotkey="!d")
            Send {Right 5}{Enter}
        Return
    $7::  ; 换到第 1 行的 7 支笔
        if(A_PriorHotkey="!d")
            Send {Right 6}{Enter}
        Return
    $!1:: ; 换到第 2 行的 1 支笔
        if(A_PriorHotkey=="!d")
            Send {Down 1}{Right 0}{Enter}
        Return
    $!2:: ; 换到第 2 行的 2 支笔
        if(A_PriorHotkey="!d")
            Send {Down 1}{Right 1}{Enter}
        Return
    $!3:: ; 换到第 2 行的 3 支笔
        if(A_PriorHotkey="!d")
            Send {Down 1}{Right 2}{Enter}
        Return
    $!4:: ; 换到第 2 行的 4 支笔
        if(A_PriorHotkey="!d")
            Send {Down 1}{Right 3}{Enter}
        Return
    $!5:: ; 换到第 2 行的 5 支笔
        if(A_PriorHotkey="!d")
            Send {Down 1}{Right 4}{Enter}
        Return
    $!6:: ; 换到第 2 行的 6 支笔
        if(A_PriorHotkey="!d")
            Send {Down 1}{Right 5}{Enter}
        Return
    $!7:: ; 换到第 2 行的 7 支笔
        if(A_PriorHotkey="!d")
            Send {Down 1}{Right 6}{Enter}
        Return