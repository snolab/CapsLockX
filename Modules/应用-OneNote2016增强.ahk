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

if !CapsLockX
    ExitApp

#Include Modules/WinClip/WinClipAPI.ahk
#Include Modules/WinClip/WinClip.ahk
global wc := new WinClip

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
    Return SubStr(str, 2, -StrLen(sep))
}
GetFocusControlName(){
    ControlGetFocus, name, A
    Return name
}

; 获取与IME无冲的编码字符串，用于 SendEvent （SEO： SendRaw SendInput）
getAscStr(str)
{
    charList := StrSplit(str)
    for key, val in charList
        out .= "{Asc " . asc(val) . "}"
    Return out
}

; 快速添加事项清单
OpenToDoList_old(){
    if !WinExist("TODO - OneNote ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE")
        Run "onenote:#TODO" ; 打开默认分区的 TODO 页面
    WinWait TODO - OneNote ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE
    WinActivate ; Uses the last found window.
    SendEvent ^{End}{Enter}
    Return
}

; 打开快速笔记主页
OpenHomePage(){
    SendEvent #n
    ; if !WinExist(".* - OneNote ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE")
    ; WinWait .* - OneNote ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE
    ; WinActivate ; Uses the last found window.
    WinWaitActive .* - OneNote ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE
    SendEvent !{Home}
    SendEvent ^{Home}
    ; SendEvent ^{End}{Enter}
    Return
}

CopySearchResultSectionAndPagesThenPaste(){
    CopySearchResultSectionAndPages()
    WinWaitNotActive ahk_class NUIDialog ahk_exe ONENOTE.EXE,, 2
    SendEvent ^v
}

; 复制链接笔记页面的搜索结果
CopySearchResultSectionAndPages(){
    WinWaitActive ahk_class NUIDialog ahk_exe ONENOTE.EXE,, 2
    ; 标题 ClassNN:	RICHEDIT60W3
    ; 地址 ClassNN:	RICHEDIT60W2
    ; 定位到第一项

    prev_addr := ""
    this_addr := ""

    links := ""
    prev_link := ""
    this_link := ""

    links_html := ""
    prev_link_html := ""
    this_link_html := ""

    samecount := 0
    k := -1
    ; 这里不加{Blind}{AltUp} 会出现连 ctrl也一起按下的bug...原因未明
    SendEvent {Blind}{AltUp}!o{Down}{Home}

    Loop, 10000 {
        ControlGetText, title, RICHEDIT60W3, A
        ControlGetText, addr , RICHEDIT60W2, A
        this_addr := addr
        isPage := !!RegExMatch(addr, "page-id=")
        Transform, title_html, HTML, %title%
        if (!isPage ){
            title := "§ " title
            title_html := "§ " title_html
        }

        this_link := "[" title_html "]" "( " addr " )" "`n"
        this_link_html := "<a href=""" addr """>" title_html "</a>" "<br />`n"

        SendEvent {Down}
        Sleep, 32
        if (this_addr == prev_addr){
            samecount++
            if (samecount >= 2){
                Break
            }
        }else{
            samecount := 0
            prev_addr := this_addr
            k += 1

            ; 这里用 prev_addr 意在去掉最后一条（一般是新建笔记）
            ; OneNote搜索默认倒字母序排列，这里把它正过来
            links := prev_link . links
            prev_link := this_link

            ; 这里用 prev_link_html 意在去掉最后一条（一般是新建笔记）
            ; OneNote搜索默认倒字母序排列，这里把它正过来
            links_html := prev_link_html . links_html
            prev_link_html := this_link_html
        }
    }

    ; links_html
    ; Clipboard := links
    Clipboard := links
    ; Sleep 128
    wc.SetText(links)
    wc.SetHTML(links_html)
    SendEvent {Escape}

    TrayTip, %k% 条笔记链接已复制, %links%, 1
}

; 原热键，打开快速笔记
; $#n:: SendEvent #n
; 打开 主页
$#!n:: OpenHomePage()
; 打开 UWP 版 OneNote 的快速笔记
$#+n:: Run "onenote-cmd://quicknote?onOpen=typing"

; #If !!(CapsLockXMode & CM_FN)
; h:: Run "https://support.office.com/zh-cn/article/OneNote-2013-%25E4%25B8%25AD%25E7%259A%2584%25E9%2594%25AE%25E7%259B%2598%25E5%25BF%25AB%25E6%258D%25B7%25E6%2596%25B9%25E5%25BC%258F-65dc79fa-de36-4ca0-9a6e-dfe7f3452ff8?ui=zh-CN&rs=zh-CN&ad=CN&fromAR=1"
; 和编辑增强冲突

#If (!CapsLockX)
    ^+!F12:: ExitApp ; 退出脚本

; OneNote2016搜索界面
#If (WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE") || WinActive("ahk_class ahk_class OneNote`:`:NavigationUIPopup ahk_exe ONENOTE.EXE"))

; $^f::
; $^e::
; enhanced_search() {
; ; 增强搜索，自动替换空格和引号等。
; if (A_ThisHotkey = "$^e"){
; SendEvent ^e
; }else{
; SendEvent ^f
; }
; If ("RICHEDIT60W1" != GetFocusControlName()){
; Return
; }
; Clipboard =
; SendEvent ^c
; ClipWait, 1

; If ErrorLevel {
; Return
; }

; s1 := Clipboard

; a := StrSplit(s1)
; s2 := StrJoin(" ", a*)
; str := """" s1 """" " OR " """" s2 """"

; Clipboard := str
; SendEvent ^v
; Return
; }

#If CapsLockXMode == CM_CapsLockX || CapsLockXMode == CM_FN && WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE")

; 上下左右
; 不知为啥这个kj在OneNote里有时候会不管用, 于是就设定了特殊的编辑操作
; k:: SendEvent {Home}{Left}
; j:: SendEvent {End}{Right}
; k:: ControlSend, OneNote::DocumentCanvas1, {Up}, ahk_exe ONENOTE.EXE
; j:: ControlSend, OneNote::DocumentCanvas1, {Down}, ahk_exe ONENOTE.EXE

#IF (CapsLockXMode && WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE"))

; /:: CapslockXShowHelp OneNote2016窗口

; OneNote2016创建链接窗口
#If WinActive("ahk_class NUIDialog ahk_exe ONENOTE.EXE")

; /:: CapslockXShowHelp OneNote2016创建链接窗口

; 复制链接笔记页面的搜索结果
!+s:: CopySearchResultSectionAndPagesThenPaste()
!s:: CopySearchResultSectionAndPages()

#If ((CapsLockXMode != CM_CapsLockX && CapsLockXMode != CM_FN) && WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE"))
    ; 自动2维化公式
$!-::
    SendEvent !=
    Sleep, 200
    altSend("jp")
Return
; SendEvent !={AppsKey}p
; 复制纯文本
$^+c::
    Clipboard =
    SendEvent ^c
    ClipWait, 1
    Clipboard := Clipboard
Return
; 粘贴纯文本
$^+v::
    Clipboard := Clipboard
    SendEvent ^v
Return

; ; 选择页面
; ^PgUp:: SendEvent ^{PgUp}^+a
; ^PgDn:: SendEvent ^{PgDn}^+a
; ; 将此页面向上合并
; $!j::
;     SendEvent ^+t^a^x
;     SendEvent ^{PgUp}^+t^{End}^v
;     SendEvent ^{PgDn}^+a{Del}
;     SendEvent ^{PgUp}
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
$F2:: SendEvent ^+t

; 重命名分区
$+F2:: SendEvent ^+g{AppsKey}r

; 复制页面链接
$!F2:: Send ^+a{AppsKey}l

; 移动笔记
$!m:: SendEvent ^!m

; 移动分区
$!+m:: SendEvent ^+g{AppsKey}m

; 搜索标记
$!f:: altSend("hg")

; 选择页面
$^+PgUp:: SendEvent ^+g{Up}{Enter}
$^+PgDn:: SendEvent ^+g{Down}{Enter}

; 同步此笔记本
$^s:: SendEvent +{F9}

; 切换为无色背景
$!n:: altSend("wpcn")
$+!n:: altSend("wre")

; 快速删除当前行
$+Delete:: SendEvent {Escape}^a{Del}

; 快速删除当前页面
$!Delete:: SendEvent ^+a{Delete}

; 快速删除当前分区（并要求确认）
$!+Delete:: SendEvent ^+g{AppsKey}d

; 快速关闭窗口
$^w:: altSend("{F4}")

; 选中行
$^+l:: SendEvent !+{Down}!+{Up}

; 选中当前词（目前来说会带上词右边的空格）
$^d:: SendEvent {Right}^{Left}^+{Right}

; 输入
$!s:: altSend("dt")
; 拖动
$!q:: altSend("dh") ;换成手形Tools
; 套锁
$!w:: altSend("dn")
; 橡皮
$!e:: altSend("dek")

; 视图 - 缩放到1
$!r:: altSend("w1")
; 视图 - 缩放到页面宽度
$!y:: altSend("wi")

; 换笔
$!d:: altSendEx("dp", "{Home}") ; 打开换笔盘，定位到第一支笔
$!a:: altSendEx("dp", "{Right 1}{Enter}") ; 笔悬停时是下一支笔，没有笔时是选红色笔

; 换笔（只在非全屏时管用）

; 展开当前关键词的相关页面链接
$!k:: 
    Clipboard := ""
    SendEvent ^a^c
    ClipWait, 1
    if(ErrorLevel){
        TrayTip, 错误, OneNote 内容未能复制成功，5秒内按 Ctrl + C 可手动复制并尝试继续流程
        ClipWait, 5
        if(ErrorLevel){
            TrayTip, 错误, OneNote 内容未能复制成功
            return
        }
    }
    SendEvent {Right}{Enter}{Tab}^k
    WinWaitActive ahk_class NUIDialog ahk_exe ONENOTE.EXE,, 2
    ; 输入搜索内容
    ControlSetText, RICHEDIT60W1, %Clipboard%, A
    ToolTip, 放开Alt键继续
    KeyWait, Alt, D
    KeyWait, Alt
    ToolTip
    CopySearchResultSectionAndPagesThenPaste()
Return

; 快速将内容做成单独链接
$!+k::
    SendEvent {Home}[[{End}]]
Return

; ; 将当前内容追加到相关页面
; $!+k:: 
;     ; 复制当前内容
;     Clipboard := ""
;     SendEvent ^a^x{Left}{Enter}^k
;     ClipWait, 2

;     ; 可能新建一个页面
;     WinWaitActive ahk_class NUIDialog ahk_exe ONENOTE.EXE,, 2
;     ; 输入搜索内容
;     ControlSetText, RICHEDIT60W1, %Clipboard%, A
;     ; 等结果出来

;     KeyWait, Alt       ; 放开Alt确认

;     SendEvent {Enter}
;     WinWaitNotActive ahk_class NUIDialog ahk_exe ONENOTE.EXE,, 2

;     ; ; （如果是新建生成的链接可能出bug不能直接点过去）
;     ; Sleep, 1000
;     ; ; 所以这里等新页面好了之后再来一次就能点进去了
;     SendEvent ^a{Delete}{Left}{Enter}^k

;     WinWaitActive ahk_class NUIDialog ahk_exe ONENOTE.EXE,, 2
;     ; 输入搜索内容
;     ControlSetText, RICHEDIT60W1, %Clipboard%, A
;     ; 等结果出来
;     KeyWait, Alt, D  ; 按Alt确认
;     SendEvent {Enter}
;     WinWaitNotActive ahk_class NUIDialog ahk_exe ONENOTE.EXE,, 2
;     SendEvent {Left}{Enter}
;     ; 在新页面末尾追加粘贴内容
;     SendEvent ^{Home}^{End}+{Tab}{Enter}^v
;     KeyWait, Alt  ; 放开Alt确认
;     SendEvent !{Left}
;     Return
; ; $!d:: altSend("dh")

; 大纲折叠展开
$!1:: SendEvent !+1
$!2:: SendEvent !+2
$!3:: SendEvent !+3
$!4:: SendEvent !+4
$!5:: SendEvent !+5
$!6:: SendEvent !+6
$!7:: SendEvent !+7

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

$1:: ; 换到第 1 行的 1 支笔
    if (A_PriorHotkey=="!d")
        SendEvent {Right 0}{Enter}
Return
$2:: ; 换到第 1 行的 2 支笔
    if (A_PriorHotkey="!d")
        SendEvent {Right 1}{Enter}
Return
$3:: ; 换到第 1 行的 3 支笔
    if (A_PriorHotkey="!d")
        SendEvent {Right 2}{Enter}
Return
$4:: ; 换到第 1 行的 4 支笔
    if (A_PriorHotkey="!d")
        SendEvent {Right 3}{Enter}
Return
$5:: ; 换到第 1 行的 5 支笔
    if (A_PriorHotkey="!d")
        SendEvent {Right 4}{Enter}
Return
$6:: ; 换到第 1 行的 6 支笔
    if (A_PriorHotkey="!d")
        SendEvent {Right 5}{Enter}
Return
$7:: ; 换到第 1 行的 7 支笔
    if (A_PriorHotkey="!d")
        SendEvent {Right 6}{Enter}
Return
$+1:: ; 换到第 2 行的 1 支笔
    if (A_PriorHotkey=="!d")
        SendEvent {Down 1}{Right 0}{Enter}
Return
$+2:: ; 换到第 2 行的 2 支笔
    if (A_PriorHotkey="!d")
        SendEvent {Down 1}{Right 1}{Enter}
Return
$+3:: ; 换到第 2 行的 3 支笔
    if (A_PriorHotkey="!d")
        SendEvent {Down 1}{Right 2}{Enter}
Return
$+4:: ; 换到第 2 行的 4 支笔
    if (A_PriorHotkey="!d")
        SendEvent {Down 1}{Right 3}{Enter}
Return
$+5:: ; 换到第 2 行的 5 支笔
    if (A_PriorHotkey="!d")
        SendEvent {Down 1}{Right 4}{Enter}
Return
$+6:: ; 换到第 2 行的 6 支笔
    if (A_PriorHotkey="!d")
        SendEvent {Down 1}{Right 5}{Enter}
Return
$+7:: ; 换到第 2 行的 7 支笔
    if (A_PriorHotkey="!d")
        Send {Down 1}{Right 6}{Enter}
Return

#IfWinExist 剪贴板.*|Clipboard ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE

~^c::
    hwndOneNote := WinExist("剪贴板.*|Clipboard ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE")
    if (!hwndOneNote)
        Return
    ; 通常在弹起时触发
    Clipboard := ""
    ClipWait, 2, 1 ; 2 secons
    if ErrorLevel {
        ; MsgBox, The attempt 2 copy text onto the clipboard failed.
        Return
    }
    WinGet, current, ID, A
    WinActivate, ahk_id %hwndOneNote%
    FormatTime, timeString, , (yyyyMMdd.HHmmss)
    SendEvent, ^{End}{Enter}
    SendEvent, {text}%timeString%
    SendEvent, ^v
    Sleep 16
    WinActivate, ahk_id %current%
Return
