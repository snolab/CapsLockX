#SingleInstance, Force

global brainstorming := false
global brainstorm_origin := CLX_Config("BrainStorm", "Website", "https://brainstorm.snomiao.com")
global brainstormApiKey := CLX_Config("BrainStorm", "Key", "FREE", t("CLX BrainStorm 的功能激活碼，填FREE使用免費版本"))
global brainstormLastQuestion := CLX_Config("BrainStorm", "LastQuestion", "", t("Brainstorm 上次提问"))

return

#if CapsLockXMode

; Brainstorm

b:: brainstorm()
+b:: brainstorm_show()
!b:: brainstorm_set_key()

#if brainstorming

esc:: stop_brainstorm()

#if

brainstorm_show()
{
    static BrainStormSite
    if (!BrainStormSite) {
        Gui BrainStorm:Destroy
        Gui BrainStorm:Add, ActiveX, xm w980 h640 vBrainStormSite, Shell.Explorer
        BrainStormSite.Silent := True
    }
    content := brainstorm_copy()
    BrainStormSite.Navigate(brainstorm_origin . "/?q=" . brainstorm_EncodeDecodeURI(content))
    Gui, BrainStorm:Show, , CapsLockX BrainStorm
}

stop_brainstorm()
{
    global brainstorming
    brainstorming := false
    ; traytip brainstorming stopped
}
brainstorm_set_key()
{
    msg := t("訪問官方網站来取得激活碼，在此輸入，或者填 FREE 使用免費版，網址如下：")
    InputBox, key, % t("激活碼輸入"), % msg "`n" brainstorm_origin
    if (ErrorLevel == 1) {
        Return
    }
    CLX_ConfigSet("BrainStorm", "Key", key)
}
brainstorm_copy()
{
    originalClipboard := ClipboardAll
    Clipboard=
    SendEvent, ^c
    ClipWait, 3 ; wait for 3 seconds
    content := Clipboard
    Clipboard := originalClipboard
    return content
}
brainstorm()
{
    content:=brainstorm_copy()

    prompt := ""
    prompt .= t("'例1：Translate to english：'")  . "`n"
    prompt .= t("'例2：解釈这句話：'")  . "`n"
    prompt .= t("'例3：总结5点：'")  . "`n"
    prompt .= "--- " . t("以下为提問内容") . " ---`n" . content
    InputBox, cmd,  % t("请輸入文本指令"), %prompt%, , 500, 600,,,,,% brainstormLastQuestion

    ; if escape
    if (ErrorLevel == 1) {
        Return
    }
    CLX_ConfigSet("BrainStorm", "LastQuestion", cmd)
    msg := Trim(content . "`n`n" . cmd, OmitChars = " `t`n")

    global brainstorming := true
    brainstorm_questionPost(msg)
}
brainstorm_questionPost(question)
{
    global brainstorming
    if (!brainstorming) {
        return
    }
    global brainstorm_origin
    endpoint := brainstorm_origin "/ai/chat?ret=polling"
    xhr := ComObjCreate("Msxml2.XMLHTTP")
    xhr.Open("POST", endpoint)
    xhr.setRequestHeader("Authorization", "Bearer " . brainstormApiKey)
    xhr.onreadystatechange := Func("BS_questionPost_onReadyStateChange").Bind(xhr)
    xhr.Send(question)
}
BS_questionPost_onReadyStateChange(xhr)
{
    global brainstorming
    if (!brainstorming)
        return
    if (xhr.readyState != 4)
        return
    if (xhr.status != 200) {
        if (xhr.status == 403) {
            MsgBox, % xhr.responseText . " " . t("请检查激活码是否正确")
            brainstorm_set_key()
        }
        if (xhr.status == 429) {
            MsgBox, % xhr.responseText . " " .  t("请等待一段时间后再试")
        }
        MsgBox, % xhr.responseText . " " . t("Unknown Error")
        return
    }
    global questionId := xhr.responseText
    if (!questionId) {
        MsgBox, t("Fail to ask ai")
        return
    }
    ; tooltip askAiSucc with question %questionId%
    tokenAppend(questionId)
}
tokenAppend(questionId)
{
    global brainstorming
    if(!brainstorming)
        return
    global brainstorm_origin
    endpoint := brainstorm_origin "/ai/" questionId
    xhra := ComObjCreate("Msxml2.XMLHTTP")
    xhra.open("GET", endpoint)
    xhra.onreadystatechange := Func("tokenAppend_onReadyStateChange").Bind(xhra)
    xhra.Send()
}
tokenAppend_onReadyStateChange(xhra)
{
    global brainstorming
    if (!brainstorming)
        return
    global questionId
    ; global brainstorm_response
    if (xhra.readyState != 4) {
        return
    }
    if (xhra.status != 200) {
        ; ToolTip
        ; TrayTip AI Response copied, %brainstorm_response%
        ; Clipboard:=brainstorm_response
        return
    }
    token := xhra.responseText
    ; brainstorm_response .= token
    ; ToolTip response %brainstorm_response%%brainstorm_response%

    SetKeyDelay, 0, 0
    SendEvent {text}%token%

    tokenAppend(questionId)
}

brainstorm_EncodeDecodeURI(str, encode := true, component := true)
{
    ; https://www.autohotkey.com/boards/viewtopic.php?t=84825
    static Doc, JS
    if !Doc {
        Doc := ComObjCreate("htmlfile")
        Doc.write("<meta http-equiv=""X-UA-Compatible"" content=""IE=9"">")
        JS := Doc.parentWindow
        ( Doc.documentMode < 9 && JS.execScript() )
    }
    Return JS[ (encode ? "en" : "de") . "codeURI" . (component ? "Component" : "") ](str)
}