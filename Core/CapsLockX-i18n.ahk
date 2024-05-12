
global CLX_Lang := CLX_Config("Core", "Language", "auto", "语言切换")
global CLX_i18nConfigPath := "Core/lang.ini"
清洗为_UTF16_WITH_BOM_型编码(CLX_i18nConfigPath)

; - [Language Codes \| AutoHotkey v1]( https://www.autohotkey.com/docs/v1/misc/Languages.htm )
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

    lang := CLX_Lang
    if (!lang) {
        lang:="auto"
    }
    if ( lang == "auto" ) {
        lang := "en"
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
    return i18n_translated(lang, key)
}

i18n_translated(lang, key)
{
    ; user translation
    ; translated := CLX_ConfigGet("lang-" . lang, key, "")
    ; if (translated) {
    ;     return translated
    ; }
    ; system translation
    translated := CLX_i18n_ConfigGet("lang-" . lang, key, "")
    if (translated) {
        return translated
    }

    question := key . "`n`nTry translate to " . lang

    global brainstorm_origin
    if (!brainstorm_origin) {
        brainstorm_origin := CLX_Config("BrainStorm", "Website", "https://brainstorm.snomiao.com")
    }
    endpoint := brainstorm_origin . "/ai/chat?ret=text"
    xhr := ComObjCreate("Msxml2.XMLHTTP")
    xhr.Open("POST", endpoint)
    xhr.setRequestHeader("Authorization", "Bearer " . brainstormApiKey)
    xhr.onreadystatechange := Func("brainstorm_translatePostResult").Bind(lang, key, xhr)
    xhr.Send(question)
    return "…[" . key . "]"
}
brainstorm_translatePostResult(lang, key, xhr)
{
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
    CLX_i18n_ConfigSet("lang-" . lang, key, transcript)
    ; CLX_ConfigSet("lang-" . lang, key, transcript)
}

i18n_changeLanguage(lang := "auto")
{
    CLX_Lang := lang
    if (!lang) {
        lang:="auto"
    }
    CLX_ConfigSet("Core", "Language", lang)
}
CLX_i18n_ConfigGet(field, varName, defaultValue)
{
    global CLX_ConfigChangedTickCount
    CLX_ConfigChangedTickCount := A_TickCount
    ; user locales
    global CLX_ConfigDir
    IniRead, content, % CLX_ConfigDir . "/" . field . ".ini", %field%, %varName%, %defaultValue%
    if (content == "ERROR") {
        content := ""
    }
    if (content) {
        return content
    }
    ; clx pre-installed locales
    IniRead, content, % CLX_i18nConfigPath, %field%, %varName%, %defaultValue%
    if (content == "ERROR") {
        content := ""
    }
    if (content) {
        return content
    }
}
CLX_i18n_ConfigSet(field, varName, value)
{
    global CLX_ConfigChangedTickCount
    CLX_ConfigChangedTickCount := A_TickCount
    global CLX_ConfigDir
    IniSave(value, CLX_ConfigDir . "/" . field . ".ini", field, varName)
    ; 清洗为_UTF16_WITH_BOM_型编码(CLX_ConfigDir)
}
