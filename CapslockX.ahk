; Encoding: UTF-8 with BOM
; Name: CapslockX
; Description: 程序入口
; Author: snomiao@gmail.com
; 
; Copyright © 2017-2020 snomiao@gmail.com
;
; 创建：Snowstar QQ: 997596439
; 参与完善：张工 QQ: 45289331
; LICENCE: GNU GPLv3
; 
;
; 模块名: 不能有这几个字符 " ,``"

Process Priority, , High     ; 脚本高优先级
#SingleInstance Force        ; 跳过对话框并自动替换旧实例
#NoTrayIcon                ; 隐藏托盘图标
#Include CapslockX-Settings.ahk

global PathModules := "模块"
global PathCore    := "核心"

global CapslockX_Version := "v1.3 Alpha"
global loadingTips := ""

LoadingTips(msg, clear = 0){
    If(clear || loadingTips == "")
        loadingTips := "CapslockX " CapslockX_Version "`n"
    loadingTips .= msg "`n"
}
LoadingTipsShow(){
    ToolTip % loadingTips
}

TryLoadModuleHelp(ModuleFileName, ModuleName){
    If(FileExist(PathModules "\" ModuleName ".md")){
        FileRead, ModuleHelp, %PathModules%\%ModuleName%.md
        Return ModuleHelp
    }
    If(FileExist(PathModules "\" ModuleFileName ".md")){
        FileRead, ModuleHelp, %PathModules%\%ModuleFileName%.md
        Return ModuleHelp
    }
    Return ""
}

; 加载模块帮助
UpdateModulesHelp(sourceREADME){
    FileEncoding UTF-8
    ; 列出模块文件
    ModuleFiles  := ""
    Loop, Files, %PathModules%\*.ahk, R ; Recurse into subfolders.
        ModuleFiles .= A_LoopFileName "`n"
    ModuleFiles := Trim(ModuleFiles, "`n")
    Sort ModuleFiles
    
    ; 生成帮助
    help := ""
    i := 0
    Loop, Parse, ModuleFiles, `n
    {
        i++
        ; 匹配模块名
        ModuleFile := A_LoopField
        re := RegExMatch(A_LoopField, "O)((?:.*[.-])*)(.*)\.ahk", Match)
        If(!re)
            Continue
        ModuleFileName := Match[1] Match[2]
        ModuleName     := Match[2]
        
        ModuleHelp := TryLoadModuleHelp(ModuleFileName, ModuleName)
        If (!ModuleHelp)
            Continue
        ModuleHelp := Trim(ModuleHelp, " `t`n")
        LoadingTips("加载模块帮助：" + i + "-" + ModuleName)

        If (T%ModuleName%_Disabled){
            help .= "### " ModuleName "模块（此模块默认禁用）"
        }Else{
            help .= "<!-- 模块帮助文件名：" Match[1] Match[2] ".ahk" "-->" "`n"
            help .= "### " ModuleName "模块"
        }
        help .= ModuleHelp "`n`n"
    }
    LoadingTipsShow()
    help := Trim(help, " `t`n")
    
    ; 生成替换代码
    NeedleRegEx := "m)(\s*)(<!-- 开始：抽取模块帮助 -->)([\s\S]*)\r?\n(\s*)(<!-- 结束：抽取模块帮助 -->)"
    Replacement := "$1$2`n" help "`n$4$5"
    targetREADME := RegExReplace(sourceREADME, NeedleRegEx, Replacement, Replaces)
    
    ; MsgBox, asdfasdf
    ; 检查替换情况
    If(!Replaces){
        MsgBox % "加载模块帮助遇到错误。`n请更新 CapslockX"
        MsgBox % targetREADME
        return sourceREADME
    }

    Return targetREADME
}

; 加载模块改为外置文件加载
; 加载   模块
global ModulesLoader := PathCore "\CapslockX-LoadModules.ahk"
LoadModules(ModulesLoader)
LoadModules(ModulesLoader){
    FileEncoding UTF-8
    ; 列出模块文件
    ModuleFiles  := ""
    Loop, Files, %PathModules%\*.ahk, R ; Recurse into subfolders.
        ModuleFiles .= A_LoopFileName "`n"
    ModuleFiles := Trim(ModuleFiles, "`n")
    Sort ModuleFiles
    
    ; 生成加载代码
    code_setup   := ""
    code_include := ""
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
            ; 这里引入模块代码
            ; 清洗为 UTF-8 WITH BOM 型编码
            FileRead ModuleCode,  %PathModules%\%ModuleFile%
            FileDelete %PathModules%\%ModuleFile%
            FileAppend %ModuleCode%,  %PathModules%\%ModuleFile%

            ; 导入模块
            code_setup   .= "    GoSub Setup_" i      "`n"
            code_include .= "    #If" "`n"
            code_include .= "        Setup_" i ":"  "`n"
            code_include .= "            #Include " PathModules "\" ModuleFile "`n"
            LoadingTips("运行模块：" i " " ModuleName)
        }
    }
    LoadingTipsShow()

    ; 拼接代码
    code := ""
    code .= code_setup
    code .= "    Return`n"
    code .= code_include 
    
    FileDelete %ModulesLoader%
    FileAppend %code%, %ModulesLoader%
}


; 编译README.md
README_FILE := "README.md"
FileRead, source, %README_FILE%
target := UpdateModulesHelp(source)
If(target != source){
    LoadingTips("模块帮助有变更")

    ; 稳定性检查
    source := UpdateModulesHelp(target)
    If(target != source){
        MsgBox % "如果你看到了这个，请联系雪星（QQ:997596439），这里肯定有 BUG……"
    }

    FileDelete %README_FILE%
    FileAppend %target%, %README_FILE%
    ; Reload
    ; ExitApp
}
; 编译核心文件
global CoreAHK := PathCore "\CapslockX-Core.ahk"

; 运行核心   
Send ^!+{F12} ; 把之前的实例关了
Run %PathCore%\AutoHotkeyU32.exe %CoreAHK%, %A_WorkingDir%

; 显示Tips 2秒
Sleep 2000
ExitApp
