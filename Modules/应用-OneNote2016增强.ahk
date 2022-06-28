; ========== CapsLockX ==========
; 名称：OneNote 2016 增强
; 版本：v2020.06.27
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版权：Copyright © 2017-2022 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========
;
; save as utf8 with bom

if (!CapsLockX) {
    MsgBox, % "本模块只为 CapsLockX 工作"
    ExitApp
}

; 引用剪贴板依赖
#Include Modules/WinClip/WinClipAPI.ahk
#Include Modules/WinClip/WinClip.ahk
global wc := new WinClip

Return

; 定义应用顶栏用到的函数
altSend(altKeys)
{
    SetKeyDelay, 1, 60 ; 配置纠错
    SendEvent {AltDown}%altKeys%{AltUp}
}

altSendEx(altKeys, suffix)
{
    SetKeyDelay, 1, 60 ; 配置纠错
    SendEvent {AltDown}%altKeys%{AltUp}%suffix%
}

StrJoin(sep, params*)
{
    for idx, param in params {
        str .= param . sep
    }
    Return SubStr(str, 2, -StrLen(sep))
}
GetFocusControlName()
{
    ControlGetFocus, name, A
    Return name
}

; 获取与IME无冲的编码字符串，用于 SendEvent （SEO： SendRaw SendInput）
getAscStr(str)
{
    charList := StrSplit(str)
    for key, val in charList {
        out .= "{Asc " . asc(val) . "}"
    }
    Return out
}

OneNote2016_Win11_Detect()
{
    IServiceProvider := ComObjCreate("{C2F03A33-21F5-47FA-B4BB-156362A2F239}", "{6D5140C1-7436-11CE-8034-00AA006009FA}")
    IVirtualDesktopManagerInternal := ComObjQuery(IServiceProvider, "{C5E0CDCA-7B6E-41B2-9FC4-D93975CC467B}", "{F31574D6-B682-4CDC-BD56-1827860ABEC6}")
    ObjRelease(IServiceProvider)
    if (IVirtualDesktopManagerInternal) {
        ; win10
        return false
    }
    ; it is Win11 maybe
    return true
}

; 打开快速笔记主页
OneNote快速笔记窗口启动(){
    if (OneNote2016_Win11_Detect()) {
        SendEvent #!n
    } else {
        ; SendEvent #n
        SendEvent #!n
    }
    
    OneNote窗口匹配串 := ".* - OneNote ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE"
    WinWaitActive %OneNote窗口匹配串%, , 1 ; wait seconds
    if (ErrorLevel) {
        WinWait %OneNote窗口匹配串%, , 5 ; wait seconds
        if (ErrorLevel) {
            TrayTip, 错误, 未找到OneNote窗口
            return
        }
        WinActivate %OneNote窗口匹配串%
    }
}
OneNote2016主页启动(){
    OneNote快速笔记窗口启动()
    SendEvent !{Home}^{Home}!{Enter}{Left}
    ; SendEvent ^{End}!{Enter}
    Return
}
OneNote2016搜索启动() {
    OneNote快速笔记窗口启动()
    SendEvent ^e{Text}""
    SendEvent {Left}
    Return
}

笔记条目搜索结果复制整理向页面粘贴条数(){
    OneNote窗口匹配串 := ".* - OneNote ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE"
    条数 := 笔记条目搜索结果复制整理条数()
    ; WinWaitNotActive ahk_class NUIDialog ahk_exe ONENOTE.EXE, , 2
    WinWaitActive %OneNote窗口匹配串%, , 5 ; wait for 5 seconds
    if(ErrorLevel) {
        TrayTip, 错误, 未找到OneNote窗口
        return
    }
    if (条数 >= 1) {
        SendEvent ^v{Left}{Delete}
    }
    return 条数
}

; 复制链接笔记页面的搜索结果
笔记条目搜索结果复制整理条数(){
    WinWaitActive ahk_class NUIDialog ahk_exe ONENOTE.EXE, , 2
    if(ErrorLevel) {
        TrayTip, 错误, 搜索结果窗口不正确
        return 0
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
    
    loop, 10000 {
        ControlGetText, title, RICHEDIT60W3, A
        ControlGetText, addr, RICHEDIT60W2, A
        this_addr := addr
        isPage := !!RegExMatch(addr, "page-id=")
        Transform, title_html, HTML, %title%
        if (!isPage ) {
            title := "§ " title
            title_html := "§ " title_html
        }
        
        this_link := "[" title_html "]" "( " addr " )" "`n"
        this_link_html := "<a title=""" title_html """ href=""" addr """>" title_html "</a>" "<br />`n"
        
        SendEvent {Down}
        Sleep, 32
        if (this_addr == prev_addr) {
            samecount++
            if (samecount >= 2) {
                Break
            }
        } else {
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
    ; remove the last break, 尽管没啥用
    links_html := RegExReplace(links_html, "\<br \/\>\n*?$", "")
    Clipboard := links
    ; Sleep 128
    wc.SetText(links)
    wc.SetHTML(links_html)
    SendEvent {Escape}
    
    TrayTip, %k% 条笔记链接已复制, %links%, 1
    return k
}

; 注意win11下OneNote启动热键变成了Win+Alt+N，而Win+N的新功能为打开消息中心
; 原热键，打开快速笔记
; $#n:: SendEvent #n
; 打开 主页
#!n:: OneNote2016主页启动()
; 打开 OneNote 并精确匹配查找搜索笔记
#+n:: OneNote2016搜索启动()
; 打开 UWP 版 OneNote 的快速笔记
; $#+n:: Run "onenote-cmd://quicknote?onOpen=typing"

; 单独运行
#if (!CapsLockX)
    
^+!F12:: ExitApp ; 退出脚本

;
#if OneNote2016搜索界面内()

OneNote2016搜索界面内(){
    return (WinActive(".*- OneNote ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE") || WinActive("ahk_class ahk_class OneNote`:`:NavigationUIPopup ahk_exe ONENOTE.EXE"))
}
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

#if OneNote2016创建链接窗口内()

OneNote2016创建链接窗口内(){
    return WinActive("ahk_class NUIDialog ahk_exe ONENOTE.EXE")
}

; /:: CapsLockX_ShowHelp OneNote2016创建链接窗口

; 复制链接笔记页面的搜索结果
!+s:: 笔记条目搜索结果复制整理向页面粘贴条数()
!s:: 笔记条目搜索结果复制整理条数()

#if OneNote2016笔记编辑窗口内()

OneNote2016笔记编辑窗口内(){
    return !CapsLockXMode && WinActive(".*- OneNote ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE")
}

F3:: 精确查找笔记()

!m:: SendEvent ^!m ; 移动笔记
!+m:: SendEvent ^+g{AppsKey}m ; 移动分区
!f:: altSend("hg") ; 搜索标记
^+PgUp:: SendEvent ^+g{Up}{Enter} ; 上一个页面切换
^+PgDn:: SendEvent ^+g{Down}{Enter} ; 下一个页面切换
^s:: SendEvent +{F9} ; 同步此笔记本
!n:: altSend("wpcn") ; 切换为无色背景
+!n:: altSend("wre") ; 切换为无格子背景
+Delete:: SendEvent {Escape}^a{Del} ; 快速删除当前行
!Delete:: SendEvent ^+a{Delete} ; 快速删除当前页面
!+Delete:: SendEvent ^+g{AppsKey}d ; 快速删除当前分区（并要求确认）
^w:: altSend("{F4}") ; 快速关闭窗口
^+l:: SendEvent !+{Down}!+{Up} ; 选中行
^d:: SendEvent {Right}^{Left}^+{Right} ; 选中当前词（目前来说会带上词右边的空格）
!q:: altSend("dh") ;换成手形Tools ; 拖动
!w:: altSend("dl") ; 套锁
!e:: altSend("dek") ; 橡皮
!s:: altSend("dt") ; 输入
!a:: altSend("dn") ; 增加空白
!+r:: altSend("w1") ; 视图 - 缩放到1
!r:: altSend("wi2") ; 视图 - 缩放到页面宽度
!d:: altSendEx("dp", "{Home}") ; 打开换笔盘，定位到第一支笔
!k:: 将当前关键词搜索到的相关页面链接在下方展开()
!t:: 把笔记时间显式填充到标题()
!-:: 自动2维化公式()
^+c:: 纯文本复制()
^+v:: 纯文本粘贴()
~Enter:: 链接安全警告自动确认()
!+p:: 段落链接复制()
!p:: 页面链接复制()
F2:: SendEvent ^+t ; 重命名笔记
+F2:: SendEvent ^+g{AppsKey}r ; 重命名分区
!F2:: SendEvent ^+a{AppsKey}l ; 页面链接复制
把笔记时间显式填充到标题(){
    backup := ClipboardAll
    Clipboard:=""
    ; copy date then focus to title
    SetKeyDelay, 0, 0
    SendEvent ^+t^{Down}^c{Left}^{Up}
    ClipWait, 2
    if(ErrorLevel) {
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
    .slice(0, 10)
    .replace(/-/g, '')
    + ')'
    )
    result := Func("SafetyEvalJavascript").Call(calcCode)
    SendEvent, {Text}%result%
}

; 按回车后1秒内，如果出现了安全警告窗口，则自动按 Yes
链接安全警告自动确认(){
    waitWindow := "ahk_class NUIDialog ahk_exe ONENOTE.EXE"
    WinWaitActive %waitWindow%, , 1
    if (!ErrorLevel)
        SendEvent !y
}

; 自动2维化公式
自动2维化公式(){
    SendEvent !=
    Sleep, 200
    altSend("jp")
}

; 纯文本复制
纯文本复制(){
    Clipboard =
    SendEvent ^c
    ClipWait, 1
    Clipboard := Clipboard
}

; 纯文本粘贴
纯文本粘贴(){
    Clipboard := Clipboard
    SendEvent ^v
}

; 复制段落链接（并清洗成 onenote 链接（段落链接的url不管用。。
段落链接复制(){
    Clipboard := ""
    SendEvent {AppsKey}pp{Enter}
    ClipWait, 1
    if(ErrorLevel)
        Return
    Clipboard := Func("SafetyEvalJavascript").Call("``" Clipboard "``.match(/^(onenote:.*)$/mi)?.[0]||""""")
}

; 复制页面链接（并清洗成 onenote 链接
页面链接复制(){
    Clipboard := ""
    SendEvent ^+a{AppsKey}l
    ClipWait, 1
    if (ErrorLevel)
        Return
    Clipboard := Func("SafetyEvalJavascript").Call("``" Clipboard "``.match(/^(onenote:.*)$/mi)?.[0]||""""")
}
精确查找笔记(){
    SendEvent ^e{Text}""
    SendEvent {Left}
}

将当前关键词搜索到的相关页面链接在下方展开(){
    Clipboard := ""
    SendEvent ^a^c
    ClipWait, 1
    if(ErrorLevel) {
        TrayTip, 错误, OneNote 内容未能复制成功，5秒内按 Ctrl + C 可手动复制并尝试继续流程
        ClipWait, 5
        if(ErrorLevel) {
            TrayTip, 错误, OneNote 内容未能复制成功
            return
        }
    }
    SendEvent {Right}{Enter}{Tab}^k
    WinWaitActive ahk_class NUIDialog ahk_exe ONENOTE.EXE, , 2
    ; 输入搜索内容
    ControlSetText, RICHEDIT60W1, %Clipboard%, A
    ToolTip, 放开Alt键继续
    KeyWait, Alt, D T60 ; wait for 60 seconds
    if(ErrorLevel) {
        ToolTip
        TrayTip, 错误, 超时未按下Alt键
        return
    }
    KeyWait, Alt, T60 ; wait for 60 seconds
    if(ErrorLevel) {
        ToolTip
        TrayTip, 错误, 超时未放开Alt键
        return
    }
    ToolTip
    条数 := 笔记条目搜索结果复制整理向页面粘贴条数()
    if(条数 == 1) {
        SendEvent +{Tab}
        SendEvent {Home}{Left}^a{Delete}
    }
}

$!+k:: SendEvent {Home}[[{End}]] ; 快速将内容做成单独链接
$!1:: SendEvent !+1 ; 大纲折叠展开到1
$!2:: SendEvent !+2 ; 大纲折叠展开到2
$!3:: SendEvent !+3 ; 大纲折叠展开到3
$!4:: SendEvent !+4 ; 大纲折叠展开到4
$!5:: SendEvent !+5 ; 大纲折叠展开到5
$!6:: SendEvent !+6 ; 大纲折叠展开到6
$!7:: SendEvent !+7 ; 大纲折叠展开到7
$!`:: altSendEx("dp", "{Down 2}{Left}")      ; 自定义颜色
$!+`:: altSend("dc")                         ; 自定义颜色
$!v:: SendEvent !h!i                         ; 自定义颜色
$![:: altSendEx("w", "{Down}{Tab 3}{Enter}") ; 调整缩放+
$!]:: altSendEx("w", "{Down}{Tab 4}{Enter}") ; 调整缩放-
$!\:: altSendEx("w", "{Down}{Tab 5}{Enter}") ; 调整缩放复原

; 调整字体
$^[:: altSendEx("h", "{Down}{Tab 1}{Up 2}{Enter}")
$^]:: altSendEx("h", "{Down}{Tab 1}{Down 2}{Enter}")
$^\:: altSendEx("h", "{Down}+{Tab 1}{Enter}")

#if OneNote2016换笔界面()

OneNote2016换笔界面(){
    return WinActive("ahk_class Net UI Tool Window ahk_exe ONENOTE.EXE") && A_PriorHotkey=="!d"
}
$1:: SendEvent {Right 0}{Enter}           ; 向第1行第1支笔切换
$2:: SendEvent {Right 1}{Enter}           ; 向第1行第2支笔切换
$3:: SendEvent {Right 2}{Enter}           ; 向第1行第3支笔切换
$4:: SendEvent {Right 3}{Enter}           ; 向第1行第4支笔切换
$5:: SendEvent {Right 4}{Enter}           ; 向第1行第5支笔切换
$6:: SendEvent {Right 5}{Enter}           ; 向第1行第6支笔切换
$7:: SendEvent {Right 6}{Enter}           ; 向第1行第7支笔切换
$+1:: SendEvent {Down 1}{Right 0}{Enter}  ; 向第2行第1支笔切换
$+2:: SendEvent {Down 1}{Right 1}{Enter}  ; 向第2行第2支笔切换
$+3:: SendEvent {Down 1}{Right 2}{Enter}  ; 向第2行第3支笔切换
$+4:: SendEvent {Down 1}{Right 3}{Enter}  ; 向第2行第4支笔切换
$+5:: SendEvent {Down 1}{Right 4}{Enter}  ; 向第2行第5支笔切换
$+6:: SendEvent {Down 1}{Right 5}{Enter}  ; 向第2行第6支笔切换
$+7:: Send {Down 1}{Right 6}{Enter}       ; 向第2行第7支笔切换

#if 名为剪贴板的OneNote窗口存在()

名为剪贴板的OneNote窗口存在(){
    return WinExist("剪贴板.*|Clipboard ahk_class Framework`:`:CFrame ahk_exe ONENOTE.EXE")
}

^c:: OneNote剪贴板收集()

OneNote剪贴板收集(){
    Clipboard := ""
    SendEvent, ^c
    hwndOneNote := 名为剪贴板的OneNote窗口存在()
    if (!hwndOneNote)
        Return
    ; ; 通常在弹起时触发
    ClipWait, 2, 1 ; 2 secons
    if(ErrorLevel) {
        TrayTip, error, 复制失败
        Return
    }
    WinGet, current, ID, A
    BlockInput, on
    WinActivate, ahk_id %hwndOneNote%
    WinSet, AlwaysOnTop, On, ahk_id %hwndOneNote%
    FormatTime, timeString, , (yyyyMMdd.HHmmss)
    SendEvent, ^{End}^{Enter}
    SendEvent, {text}%timeString%
    SendEvent, {Tab}^v
    Sleep 128
    WinActivate, ahk_id %current%
    BlockInput, off
}

