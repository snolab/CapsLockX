Process Priority, , High     ; 脚本高优先级
#SingleInstance Force        ; 跳过对话框并自动替换旧实例
#NoTrayIcon                ; 隐藏托盘图标
#Include CapsX-Settings.ahk

global CapsX_Version := "v1.1 Alpha"
LoadingTips(msg, clear = 0){
    static tips
    If(clear || tips == "")
        tips := "CapsX " CapsX_Version "`n"
    tips .= msg "`n"
    ToolTip % tips
}
; 加载模块

CodeLoadModules(source){
    FileEncoding UTF-8
    ; 列出模块文件
    code_setup   := ""
    code_include := ""
    ModuleFiles  := ""
    Loop, Files, Modules\*.ahk, R ; Recurse into subfolders.
        ModuleFiles .= A_LoopFileName "`n"
    ModuleFiles := Trim(ModuleFiles, "`n")
    Sort ModuleFiles
    
    ; 生成加载代码
    i := 0
    Loop, Parse, ModuleFiles, `n
    {
        i++
        ; 匹配模块名
        ModuleFile := A_LoopField
        re := RegExMatch(A_LoopField, "O)(?:.*[.-])*(.*)\.ahk", Match)
        If(!re)
            Continue
        ModuleName := Match[1]
        
        If(T%ModuleName%_Disabled)
            LoadingTips("禁用模块：" i " " ModuleName)
        Else{
            code_include .= "    #If" "`n"
            ; code .= "    global MF_" ModuleName " := " 1 << (i - 1) "`n"
            code_setup   .= "        GoSub Setup_" ModuleName "`n"
            code_include .= "        Setup_" ModuleName ":"  "`n"
            code_include .= "        #Include Modules\" ModuleFile "`n"

            ; FileRead ModuleCode, Modules\%ModuleFile%

            ; If(RegExMatch(ModuleCode, "m)^\s*T" ModuleName "_Setup:$")){
            ;     code_setup .= "    GoSub T" ModuleName "_Setup`n"
            ;     LoadingTips("运行模块：" i " " ModuleName)
            ; }Else{
            ;     LoadingTips("加载模块：" i " " ModuleName)
            ; }

        }
    }
    ; 拼接代码
    code := ""
    code .= code_setup
    code .= "    Return`n"
    code .= code_include 

    ; 生成替换代码
    ;NeedleRegEx := "(\s*)(; 动态开始：载入模块)([\s\S]*)\n\1(; 动态结束；)"
    NeedleRegEx := "m)^(\s*)(; 动态开始：载入模块)([\s\S]*)`n\1(; 动态结束；)"
    Replacement := "$1$2`n" code "$1$4"
    
    target := RegExReplace(source, NeedleRegEx, Replacement, Replaces)

    ; 检查替换情况
    If(Replaces == 0)
        MsgBox, "加载模块遇到错误。`n请更新 CapsX"
    Return target
}

CoreAHK := "Core\CapsX-Core.ahk"
FileRead, source, %CoreAHK%
target := CodeLoadModules(source)
If(target != source){
    LoadingTips("模块设定有变更", 1)

    ; 稳定性检查
    source := CodeLoadModules(target)
    If(target != source)
        MsgBox % "如果你看到了这个，请联系雪星（QQ:997596439），这里肯定有 BUG……"

    FileDelete %CoreAHK%
    FileAppend %target%, %CoreAHK%
    ; Reload
    ; ExitApp
}

Send ^!+{F12} ; 把之前的实例关了
Run %CoreAHK%, %A_WorkingDir%
Sleep 2000
ExitApp

;"(^\s*);#T_" Module "_" SettingName "{{ ([\s\S]*)\1;}}"

; #T_COMDING = 1

; XBEGIN := "; #T_" Module "_" SettingName

; NeedleRegEx := "(\s*)(" XBEGIN ")((?:\1.*\n|\n)*)\1(" XEND ")"
; Replacement := "$1$2" code "$1$4"

; !F12:: ExitApp