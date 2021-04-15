; ========== CapsLockX ==========
; 名称：OneNote 2016 增强
; 版本：v2020.06.27
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版权：Copyright © 2017-2021 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========
; 
; save as utf8 with bom

if !CapsLockX
    ExitApp

; 引用剪贴板依赖
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

StrJoin(sep, params*){
    for index, param in params
        str .= param . sep
    Return SubStr(str, 2, -StrLen(sep))
}
GetFocusControlName(){
    ControlGetFocus, name, A
    Return name
}

; 获取与IME无冲的编码字符串，用于 SendEvent （SEO： SendRaw SendInput）
getAscStr(str){
    charList := StrSplit(str)
    for key, val in charList
        out .= "{Asc " . asc(val) . "}"
    Return out
}

; 打开快速笔记主页
OneNote2016_OpenHomePage(){
    SendEvent #n
    ; if !WinExist(".* - OneNote ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE")
    ; WinWait .* - OneNote ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE
    ; WinActivate ; Uses the last found window.
    WinWaitActive .* - OneNote ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE , , 5 ; wait for 5 seconds
    if(ErrorLevel){
        TrayTip, 错误, 未找到OneNote窗口
        return
    }
    SendEvent !{Home}
    SendEvent ^{Home}
    ; SendEvent ^{End}{Enter}
    Return
}

CopySearchResultSectionAndPagesThenPaste(){
    CopySearchResultSectionAndPages()
    ; WinWaitNotActive ahk_class NUIDialog ahk_exe ONENOTE.EXE,, 2
    WinWaitActive ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE , , 5 ; wait for 5 seconds
    if(ErrorLevel){
        TrayTip, 错误, 未找到OneNote窗口
        return
    }
    SendEvent ^v
}

; 复制链接笔记页面的搜索结果
CopySearchResultSectionAndPages(){
    WinWaitActive ahk_class NUIDialog ahk_exe ONENOTE.EXE,, 2
    if(ErrorLevel){
        TrayTip, 错误, 搜索结果窗口不正确
        return
    }
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
        this_link_html := "<a title=""" title_html """ href=""" addr """>" title_html "</a>" "<br />`n"

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
            ; OneNote搜索默认倒字母序排列，这里把它正过来 /(20210401)发现不是这样的 ，决定在下面另外排序
            links := prev_link . links
            prev_link := this_link

            ; 这里用 prev_link_html 意在去掉最后一条（一般是新建笔记）
            ; OneNote搜索默认倒字母序排列，这里把它正过来 /(20210401)发现不是这样的 ，决定在下面另外排序
            links_html := prev_link_html . links_html
            prev_link_html := this_link_html
        }
    }

    ; links_html
    ; Clipboard := links
    Sort links
    Sort links_html
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
$#!n:: OneNote2016_OpenHomePage()
; 打开 OneNote 并精确匹配查找搜索笔记
$#+n:: OneNote2016_OpenSearch()
OneNote2016_OpenSearch(){
    winTitle := "ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE"
    needActive := 1
    hWnd := WinExist(winTitle)
    if(!hWnd){
        SendEvent #n
        WinWaitActive %winTitle%, , 1 ; wait for 1 seconds
        if(ErrorLevel){
            needActive := 1
            WinWait %winTitle%, , 1 ; wait for 1 seconds
            if(ErrorLevel){
                TrayTip, 错误, 未找到OneNote窗口
                return
            }
        }
        hWnd := LastFound
    }
    if(needActive)
        WinActivate ahk_id %hWnd%
    SendEvent ^e{Text}""
    SendEvent {Left}
    Return
}
; 打开 UWP 版 OneNote 的快速笔记
; $#+n:: Run "onenote-cmd://quicknote?onOpen=typing"

; #If !!(CapsLockXMode & CM_FN)
; h:: Run "https://support.office.com/zh-cn/article/OneNote-2013-%25E4%25B8%25AD%25E7%259A%2584%25E9%2594%25AE%25E7%259B%2598%25E5%25BF%25AB%25E6%258D%25B7%25E6%2596%25B9%25E5%25BC%258F-65dc79fa-de36-4ca0-9a6e-dfe7f3452ff8?ui=zh-CN&rs=zh-CN&ad=CN&fromAR=1"
; 和编辑增强冲突

#If (!CapsLockX)
    ^+!F12:: ExitApp ; 退出脚本

; OneNote2016搜索界面
#If (WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE") || WinActive("ahk_class ahk_class OneNote`:`:NavigationUIPopup ahk_exe ONENOTE.EXE"))

; $^f::
; $^e::
; enhanced_search(){
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

; /:: CapsLockX_ShowHelp OneNote2016窗口

; OneNote2016创建链接窗口
#If WinActive("ahk_class NUIDialog ahk_exe ONENOTE.EXE")

; /:: CapsLockX_ShowHelp OneNote2016创建链接窗口

; 复制链接笔记页面的搜索结果
!+s:: CopySearchResultSectionAndPagesThenPaste()
!s:: CopySearchResultSectionAndPages()

#If !CapsLockXMode && WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE")

; 按回车后1秒内，如果出现了安全警告窗口，则自动按 Yes
$~Enter::
    waitWindow := "ahk_class NUIDialog ahk_exe ONENOTE.EXE"
    WinWaitActive %waitWindow%,, 1
    if (!ErrorLevel)
        SendEvent !y
Return

; 自动2维化公式
$!-::
    SendEvent !=
    Sleep, 200
    altSend("jp")
Return

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

; 复制段落链接（并清洗成 onenote 链接
$!p::
    Clipboard := ""
    SendEvent {AppsKey}pp{Enter}
    ClipWait, 1
    if(ErrorLevel)
        Return
    Clipboard := Func("SafetyEvalJavascript").Call("``" Clipboard "``.match(/^(onenote:.*)$/mi)?.[0]||""""")
Return

; 重命名笔记
$F2:: SendEvent ^+t

; 重命名分区
$+F2:: SendEvent ^+g{AppsKey}r

; 复制页面链接
$!F2:: SendEvent ^+a{AppsKey}l

; 精确查找笔记
$F3::
    SendEvent ^e{Text}""
    SendEvent {Left}
Return

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

; 切换为无格子背景
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

; 拖动
$!q:: altSend("dh") ;换成手形Tools
; 套锁
$!w:: altSend("dl")
; 橡皮
$!e:: altSend("dek")
; 输入
$!s:: altSend("dt")
; 增加空白
$!a:: altSend("dn")
; 视图 - 缩放到1
$!+r:: altSend("w1")
; 视图 - 缩放到页面宽度
$!r:: altSend("wi")

; 换笔
$!d:: altSendEx("dp", "{Home}") ; 打开换笔盘，定位到第一支笔
; $!a:: altSendEx("dp", "{Right 1}{Enter}") ; 笔悬停时是下一支笔，没有笔时是选红色笔

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
    KeyWait, Alt, D T60 ; wait for 60 seconds
    if(ErrorLevel){
        ToolTip
        TrayTip, 错误, 超时未按下Alt键
        return
    }
    KeyWait, Alt, T60 ; wait for 60 seconds
    if(ErrorLevel){
        ToolTip
        TrayTip, 错误, 超时未放开Alt键
        return
    }
    ToolTip
    CopySearchResultSectionAndPagesThenPaste()
Return

; 快速将内容做成单独链接
$!+k::
    SendEvent {Home}[[{End}]]
Return

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

; 调整缩放
$![:: altSendEx("w", "{Down}{Tab 3}{Enter}")
$!]:: altSendEx("w", "{Down}{Tab 4}{Enter}")
$!\:: altSendEx("w", "{Down}{Tab 5}{Enter}")

; 调整字体
$^[:: altSendEx("h", "{Down}{Tab 1}{Up 2}{Enter}")
$^]:: altSendEx("h", "{Down}{Tab 1}{Down 2}{Enter}")
$^\:: altSendEx("h", "{Down}+{Tab 1}{Enter}")

#if WinActive("ahk_class Net UI Tool Window ahk_exe ONENOTE.EXE") && A_PriorHotkey=="!d"

; 换到第 1 行的 1 支笔
$1:: SendEvent {Right 0}{Enter}
; 换到第 1 行的 2 支笔
$2:: SendEvent {Right 1}{Enter}
; 换到第 1 行的 3 支笔
$3:: SendEvent {Right 2}{Enter}
; 换到第 1 行的 4 支笔
$4:: SendEvent {Right 3}{Enter}
; 换到第 1 行的 5 支笔
$5:: SendEvent {Right 4}{Enter}
; 换到第 1 行的 6 支笔
$6:: SendEvent {Right 5}{Enter}
; 换到第 1 行的 7 支笔
$7:: SendEvent {Right 6}{Enter}
; 换到第 2 行的 1 支笔
$+1:: SendEvent {Down 1}{Right 0}{Enter}
; 换到第 2 行的 2 支笔
$+2:: SendEvent {Down 1}{Right 1}{Enter}
; 换到第 2 行的 3 支笔
$+3:: SendEvent {Down 1}{Right 2}{Enter}
; 换到第 2 行的 4 支笔e
$+4:: SendEvent {Down 1}{Right 3}{Enter}
; 换到第 2 行的 5 支笔
$+5:: SendEvent {Down 1}{Right 4}{Enter}
; 换到第 2 行的 6 支笔
$+6:: SendEvent {Down 1}{Right 5}{Enter}
; 换到第 2 行的 7 支笔
$+7:: Send {Down 1}{Right 6}{Enter}

#if WinActive("ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE")

!t:: 把笔记时间显式填充到标题()
把笔记时间显式填充到标题(){
    backup := ClipboardAll
    Clipboard:=""
    ; copy date then focus to title
    SetKeyDelay, 0, 0
    SendEvent ^+t^{Down}^c{Left}^{Up}
    ClipWait ,2
    if(ErrorLevel){
        Clipboard := backup
        return
    }
    dateString := Trim(Clipboard, "`r`n")
    Clipboard := backup
    calcCode =
    (
    '('
    + new Date(+new Date("%dateString%".replace(/年|月/g, '-').replace(/日|星期./g, '').trim())+8*3600e3)
    .toISOString()
    .slice(0,10)
    .replace(/-/g,'')
    + ')'
    )
    result := Func("SafetyEvalJavascript").Call(calcCode)
    SendEvent, {Text}%result%
}

#If WinExist("剪贴板.*|Clipboard ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE")

~^c::
    hwndOneNote := WinExist("剪贴板.*|Clipboard ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE")
    if (!hwndOneNote)
        Return
    ; ; 通常在弹起时触发
    ; Clipboard := ""
    ClipWait, 2, 1 ; 2 secons
    if(ErrorLevel){
        TrayTip, error, The attempt 2 copy text onto the clipboard failed.
        Return
    }
    WinGet, current, ID, A
    WinActivate, ahk_id %hwndOneNote%
    FormatTime, timeString, , (yyyyMMdd.HHmmss)
    SendEvent, ^{End}{Enter}
    SendEvent, {text}%timeString%
    SendEvent, ^v
    Sleep 128
    WinActivate, ahk_id %current%
Return

