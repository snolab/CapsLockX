; - [Language Codes \| AutoHotkey v1]( https://www.autohotkey.com/docs/v1/misc/Languages.htm )

global CLX_Lang := CLX_Config("Core", "Language", "auto", "语言切换")

; Hans
LCID_7804 := "Chinese"  ; zh
LCID_0004 := "Chinese (Simplified)" ; zh-Hans
LCID_0804 := "Chinese (Simplified, China)" ; zh-CN
LCID_1004 := "Chinese (Simplified, Singapore)" ; zh-SG
LCID_7C04 := "Chinese (Traditional)"  ; zh-Hant
LCID_0C04 := "Chinese (Traditional, Hong Kong SAR)"  ; zh-HK
LCID_1404 := "Chinese (Traditional, Macao SAR)"  ; zh-MO
LCID_0404 := "Chinese (Traditional, Taiwan)"  ; zh-TW
LCID_0011 := "Japanese"  ; ja
LCID_0411 := "Japanese (Japan)"  ; ja-JP

; TODO: converts
t(s)
{
    global CLX_Lang

    key := s
    defaultValue := s
    explain := s

    ; for dev, autotranslate
    ; run node "prompts/translate-en.md"
    lang := "en"
    if ( CLX_Lang == "auto" ) {
        if (A_Language == "7804") {
            lang := "zh"
        }
        if (A_Language == "0004") {
            lang := "zh"
        }
        if (A_Language == "0804") {
            lang := "zh"
        }
        if (A_Language == "1004") {
            lang := "zh"
        }
        if (A_Language == "7C04") {
            lang := "zh"
        }
        if (A_Language == "0C04") {
            lang := "zh"
        }
        if (A_Language == "1404") {
            lang := "zh"
        }
        if (A_Language == "0404") {
            lang := "zh"
        }
        if (A_Language == "0011") {
            lang := "ja"
        }
        if (A_Language == "0411") {
            lang := "ja"
        }
    }
    clx_i18n_TranslateByAsync(lang, key)
    return CLX_ConfigGet("lang-" lang, key, key)
}
clx_i18n_TranslateByAsync(lang, key){
    translated := CLX_ConfigGet("lang-" lang, key, "")
    if (translated) {
        return
    }
    question := key . "`n`nTranslate to " . lang

    ; TrayTip, % "CapsLockX i18n [" . lang "]", % key "-->" lang,

    global brainstorm_origin
    endpoint := brainstorm_origin "/ai/chat?ret=text"
    xhr := ComObjCreate("Msxml2.XMLHTTP")
    xhr.Open("POST", endpoint)
    xhr.setRequestHeader("Authorization", "Bearer " . brainstormApiKey)
    xhr.onreadystatechange := Func("brainstorm_translatePostResult").Bind(lang, key, xhr)
    xhr.Send(question)
}
brainstorm_translatePostResult(lang, key, xhr){
    if (xhr.readyState != 4)
        return
    if (xhr.status != 200) {
        if (xhr.status == 429) {
            MsgBox, % xhr.responseText " Please wait a moment then try again."
        }
        MsgBox, % xhr.responseText " Unknown Error"
        return
    }
    global transcript := xhr.responseText
    if (!transcript) {
        MsgBox, fail to ask ai
        return
    }
    TrayTip, % "CapsLockX i18n [" . lang . "]", % key "=>" transcript,

    CLX_ConfigSet("lang-" lang, key, transcript)
}

i18n_changeLanguage(lang := "auto")
{
    CLX_Lang := lang
    CLX_ConfigSet("Core", "Language", lang)
}