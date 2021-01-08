; Encoding: UTF-8 with BOM
; Name: CapsLockX
; Description: 程序入口
; Author: snomiao@gmail.com
;
; Copyright © 2017-2020 snomiao@gmail.com
; 创建：Snowstar QQ: 997596439
; 鸣谢：张工 QQ: 45289331 参与调试
; LICENCE: GNU GPLv3
;
;
; 模块名: 不能有这几个字符 ", ``"

#SingleInstance Force        ; 跳过对话框并自动替换旧实例
#NoTrayIcon                ; 隐藏托盘图标
#Include CapsLockX-Settings.ahk
Process Priority, , High     ; 脚本高优先级

global CapsLockX_PathModules := "./Modules"
global CapsLockX_PathCore    := "./Core"
global CapsLockX_Version := "v1.7.0 Beta"

global loadingTips := ""

LoadingTips(msg, clear = 0)
{
    if (clear || loadingTips == "") {
        
        loadingTips := "CapsLockX " CapsLockX_Version "`n"
    }
    loadingTips .= msg "`n"
}
ShowLoadingTips()
{
    ToolTip % loadingTips
}
TryLoadModuleHelp(ModuleFileName, ModuleName)
{
    if (FileExist(CapsLockX_PathModules "\" ModuleName ".md")) {
        FileRead, ModuleHelp, %CapsLockX_PathModules%\%ModuleName%.md
        Return ModuleHelp
    }
    if (FileExist(CapsLockX_PathModules "\" ModuleFileName ".md")) {
        FileRead, ModuleHelp, %CapsLockX_PathModules%\%ModuleFileName%.md
        Return ModuleHelp
    }
    Return ""
}
UpdateModulesHelp(sourceREADME, docs="")
{
    FileEncoding UTF-8
    ; 列出模块文件
    ModuleFiles  := ""
    ; loop, Files, %CapsLockX_PathModules%\*.ahk, R ; Recurse into subfolders.
    loop, Files, %CapsLockX_PathModules%\*.ahk, ; Do not Recurse into subfolders.
    ModuleFiles .= A_LoopFileName "`n"
    ModuleFiles := Trim(ModuleFiles, "`n")
    Sort ModuleFiles
    
    ; 生成帮助
    help := ""
    i := 0
    loop, Parse, ModuleFiles, `n
    {
        i++
        ; 匹配模块名
        ModuleFile := A_LoopField
        re := RegExMatch(A_LoopField, "O)((?:.*[.-])*)(.*)\.ahk", Match)
        if (!re) {
            
            Continue
        }
        ModuleFileName := Match[1] Match[2]
        ModuleName     := Match[2]
        
        ModuleHelp := TryLoadModuleHelp(ModuleFileName, ModuleName)
        if (!ModuleHelp) {
            
            Continue
        }
        ModuleHelp := Trim(ModuleHelp, " `t`n")
        LoadingTips("加载模块帮助：" + i + "-" + ModuleName)
        
        help .= "<!-- 模块文件名：" Match[1] Match[2] ".ahk" "-->" "`n"
        ; 替换标题层级
        ModuleHelp := RegExReplace(ModuleHelp, "m)^#", "###")
        
        ; 替换资源链接的相对目录（图片gif等）
        
        ; position := RegExMatch(ModuleHelp, "Om)\[(.*)\]\(\s*?\.\/(.*?)\)", matchObject)
        ; loop, matchObject.Count()
        ; {
        ;     MsgBox % matchObject[A_Index]
        ; }
        FileCopy, %CapsLockX_PathModules%\*.gif, .\docs\media\, 1
        FileCopy, %CapsLockX_PathModules%\*.png, .\docs\media\, 1
        ModuleHelp := RegExReplace(ModuleHelp, "m)\[(.*)\]\(\s*?\.\/(.*?)\)", "[$1]( ./media/$2 )")
        
        ; if (docs){
        ;     ModuleHelp := RegExReplace(ModuleHelp, "m)\[(.*)\]\(\s*?\.\/(.*?)\)", "[$1]( ./$2 )")
        ; }else{
        ;     ModuleHelp := RegExReplace(ModuleHelp, "m)\[(.*)\]\(\s*?\.\/(.*?)\)", "[$1]( ./" CapsLockX_PathModules " /$2 ")
        ; }
        
        if (!RegExMatch(ModuleHelp, "^#")) {
            if (T%ModuleName%_Disabled) {
                help .=  "`n" "### " ModuleName "模块（禁用）" "`n"
            } else {
                help .=  "`n" "### " ModuleName "模块" "`n"
            }
        }
        help .= ModuleHelp "`n`n"
    }
    ShowLoadingTips()
    help := Trim(help, " `t`n")
    
    ; 生成替换代码
    NeedleRegEx := "m)(\s*)(<!-- 开始：抽取模块帮助 -->)([\s\S]*)\r?\n(\s*)(<!-- 结束：抽取模块帮助 -->)"
    Replacement := "$1$2`n" help "`n$4$5"
    targetREADME := RegExReplace(sourceREADME, NeedleRegEx, Replacement, Replaces)
    
    ; 检查替换情况
    if (!Replaces) {
        MsgBox % "加载模块帮助遇到错误。`n请更新 CapsLockX"
        MsgBox % targetREADME
        Return sourceREADME
    }
    
    Return targetREADME
}
LoadModules(ModulesLoader)
{
    FileEncoding UTF-8
    ; 列出模块文件
    ModuleFiles  := ""
    ; loop, Files, %CapsLockX_PathModules%\*.ahk, R ; Recurse into subfolders.
    loop, Files, %CapsLockX_PathModules%\*.ahk, ; NOT Recurse into subfolders.
    ModuleFiles .= A_LoopFileName "`n"
    ModuleFiles := Trim(ModuleFiles, "`n")
    Sort ModuleFiles
    
    ; 生成加载代码
    code_setup   := ""
    code_include := ""
    i := 0
    loop, Parse, ModuleFiles, `n
    {
        i++
        ; 匹配模块名
        ModuleFile := A_LoopField
        re := RegExMatch(A_LoopField, "O)(?:.*[.-])*(.*)\.ahk", Match)
        if (!re) {
            
            Continue
        }
        ModuleName := Match[1]
        
        if (T%ModuleName%_Disabled) {
            LoadingTips("禁用模块：" i " " ModuleName)
        } else {
            ; 这里引入模块代码
            ; 清洗为 UTF-8 WITH BOM 型编码
            FileRead ModuleCode, %CapsLockX_PathModules%\%ModuleFile%
            FileDelete %CapsLockX_PathModules%\%ModuleFile%
            FileAppend %ModuleCode%, %CapsLockX_PathModules%\%ModuleFile%
            
            ; 导入模块
            code_setup   .= "GoSub Setup_" i      "`n"
            code_include .= "#If" "`n"
            code_include .= "    Setup_" i ":"  "`n"
            code_include .= "        #Include " CapsLockX_PathModules "\" ModuleFile "`n"
            LoadingTips("运行模块：" i " " ModuleName)
        }
    }
    ShowLoadingTips()
    
    ; 拼接代码
    code_consts .= "global CapsLockX_PathModules := " """" CapsLockX_PathModules """" "`n"
    code_consts .= "global CapsLockX_PathCore := "    """" CapsLockX_PathCore    """" "`n"
    code_consts .= "global CapsLockX_Version := "     """" CapsLockX_Version     """" "`n"
    
    code := ""
    code .= code_consts "`n"
    code .= code_setup  "`n"
    code .= "Return" "`n"
    code .= code_include "`n"
    
    FileDelete %ModulesLoader%
    FileAppend %code%, %ModulesLoader%
}

; 加载模块
global ModulesLoader := CapsLockX_PathCore "\CapsLockX-LoadModules.ahk"
LoadModules(ModulesLoader)

; 编译README.md
INPUT_README_FILE := "./docs/README.md"
FileRead, source, %INPUT_README_FILE%

; 加载模块帮助
target := UpdateModulesHelp(source)
if (target != source) {
    LoadingTips("模块帮助有变更")
    
    ; 稳定性检查
    source := UpdateModulesHelp(target)
    if (target != source) {
        MsgBox % "如果你看到了这个，请联系雪星（QQ:997596439），这里肯定有 BUG……(20200228)"
    }
    ; 输出根目录 README.md
    FileDelete ./README.md
    PREFIX := "<!-- THIS FILE IS GENERATED PLEASE MODIFY DOCS/README --->`n"
    FileAppend %PREFIX% %target%, ./README.md
    
    ; 输出到 docs/readme.md （用于 github-pages ）
    ; docs_target := UpdateModulesHelp(source, 1)
    FileDelete ./docs/README.md
    FileAppend %target%, ./docs/README.md
    
    ; Reload
    ; ExitApp
}
; 编译核心文件
global CoreAHK := CapsLockX_PathCore "\CapsLockX-Core.ahk"

; 运行核心
Send ^!+{F12} ; 把之前的实例关了

Run %CapsLockX_PathCore%\AutoHotkeyU32.exe %CoreAHK%, %A_WorkingDir%

; 显示Tips 2秒
Sleep 2000
ExitApp
