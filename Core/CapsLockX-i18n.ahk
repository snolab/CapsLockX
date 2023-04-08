
; ; Implementing i18n with autohotkey
; #Include <i18n.ahk>

; ; Set the default language to English
; i18n_SetLanguage("en")

; ; Define translations for different languages
; i18n_AddTranslation("en", "Hello", "Hello")
; i18n_AddTranslation("fr", "Hello", "Bonjour")
; i18n_AddTranslation("es", "Hello", "Hola")

; ; Use the translation function to display text in the correct language
; MsgBox % i18n("Hello")
t(s){
    return s
}
