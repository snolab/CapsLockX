; ========== CapsLockX ==========
; 名称：CapsLockX 程序入口
; 描述：用于生成模块加载组件、集成模块帮助文件、智能重载 CapsLockX 核心等功能。
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 编码：UTF-8 with BOM
; 版权：Copyright © 2017-2021 Snowstar Laboratory. All Rights Reserved.
; 鸣谢：张工 QQ: 45289331 参与调试
; LICENCE: GNU GPLv3
; ========== CapsLockX ==========

#SingleInstance Force ; 跳过对话框并自动替换旧实例
#NoTrayIcon ; 隐藏托盘图标

SendMode Event
SetWorkingDir, %A_ScriptDir%

global CapsLockX_PathModules := "./Modules"
global CapsLockX_PathCore := "./Core"
global CapsLockX_Version
FileRead, CapsLockX_Version, ./Tools/version.txt
if(!CapsLockX_Version)
    CapsLockX_Version := "未知版本"
global CapsLockX_VersionName := "v" CapsLockX_Version
global loadingTips := ""

; 对 核心模块 进行 编码清洗
清洗为_UTF8_WITH_BOM_型编码(CapsLockX_PathCore "/CapsLockX-Config.ahk")
清洗为_UTF8_WITH_BOM_型编码(CapsLockX_PathCore "/CapsLockX-Core.ahk")
清洗为_UTF8_WITH_BOM_型编码(CapsLockX_PathCore "/CapsLockX-RunSilent.ahk")
清洗为_UTF8_WITH_BOM_型编码(CapsLockX_PathCore "/CapsLockX-Update.ahk")

; 加载模块
global ModulesRunner := CapsLockX_PathCore "\CapsLockX-ModulesRunner.ahk"
global ModulesLoader := CapsLockX_PathCore "\CapsLockX-ModulesLoader.ahk"
LoadModules(ModulesRunner, ModulesLoader)

; 编译README.md
INPUT_README_FILE := "./docs/README.md"
FileRead, source, %INPUT_README_FILE%

; 加载模块帮助
target := 模块帮助README更新(source)
if (target != source){
    加载提示追加("模块帮助有变更")

    ; 稳定性检查
    source := 模块帮助README更新(target)
    if (target != source){
        MsgBox % "如果你看到了这个，请联系雪星（QQ:997596439），这里肯定有 BUG……(20200228)"
    }
    ; 输出到 docs/readme.md （用于 github-pages ）
    ; docs_target := 模块帮助README更新(source, 1)
    FileDelete ./docs/README.md
    FileAppend %target%, ./docs/README.md, UTF-8-Raw

    ; 输出根目录 README.md （用于 github 首页）
    FileDelete ./README.md
    PREFIX := "<!-- THIS FILE IS GENERATED PLEASE MODIFY DOCS/README -->`n`n"
    StringReplace, target, target, ./media/, ./docs/media/, All
    FileAppend %PREFIX%%target%, ./README.md, UTF-8-Raw
    ; Reload
    ; ExitApp
}
; 隐藏 ToolTip
ToolTip

; 当 CI_TEST 启用时，仅测试编译效果，不启动核心
EnvGet ENVIROMENT, ENVIROMENT
if("CI_TEST" == ENVIROMENT){
    OutputDebug, % "[INFO] MODULE LOAD OK, SKIP CORE"
    ExitApp
}
CapslockX_启动()

Return

加载提示追加(msg, clear = 0){
    if (clear || loadingTips == ""){
        loadingTips := "CapsLockX " CapsLockX_Version "`n"
    }
    loadingTips .= msg "`n"
}
加载提示显示(){
    ToolTip % loadingTips
}
模块帮助加载尝试(ModuleFileName, ModuleName){
    if (FileExist(CapsLockX_PathModules "\" ModuleName ".md")){
        FileRead, ModuleHelp, %CapsLockX_PathModules%\%ModuleName%.md
        Return ModuleHelp
    }
    if (FileExist(CapsLockX_PathModules "\" ModuleFileName ".md")){
        FileRead, ModuleHelp, %CapsLockX_PathModules%\%ModuleFileName%.md
        Return ModuleHelp
    }
    Return ""
}
清洗为_UTF8_WITH_BOM_型编码(path){
    FileRead ModuleCode, %path%
    FileDelete %path%
    FileAppend %ModuleCode%, %path%, UTF-8
}
模块帮助README更新(sourceREADME, docs=""){
    FileEncoding UTF-8
    ; 列出模块文件
    ModuleFiles := ""
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
        if (!re){
            Continue
        }
        ModuleFileName := Match[1] Match[2]
        ModuleName := Match[2]

        ModuleHelp := 模块帮助加载尝试(ModuleFileName, ModuleName)
        if (!ModuleHelp){
            Continue
        }
        ModuleHelp := Trim(ModuleHelp, " `t`n")
        加载提示追加("加载模块帮助：" + i + "-" + ModuleName)

        help .= "<!-- 模块文件名：" Match[1] Match[2] ".ahk" "-->" "`n`n"
        ; 替换标题层级
        ModuleHelp := RegExReplace(ModuleHelp, "m)^#", "###")

        ; 替换资源链接的相对目录（图片gif等）
        FileCopy, %CapsLockX_PathModules%\*.gif, .\docs\media\, 1
        FileCopy, %CapsLockX_PathModules%\*.png, .\docs\media\, 1
        ModuleHelp := RegExReplace(ModuleHelp, "m)\[(.*)\]\(\s*?\.\/(.*?)\)", "[$1]( ./media/$2 )")

        ; if (docs){
        ;     ModuleHelp := RegExReplace(ModuleHelp, "m)\[(.*)\]\(\s*?\.\/(.*?)\)", "[$1]( ./$2 )")
        ; }else{
        ;     ModuleHelp := RegExReplace(ModuleHelp, "m)\[(.*)\]\(\s*?\.\/(.*?)\)", "[$1]( ./" CapsLockX_PathModules " /$2 ")
        ; }

        ; 没有标题的，给自动加标题
        if (!RegExMatch(ModuleHelp, "^#")){
            if (T%ModuleName%_Disabled){
                help .= "### " ModuleName "模块（禁用）" "`n"
            } else {
                help .= "### " ModuleName "模块" "`n"
            }
        }
        help .= ModuleHelp "`n`n"
    }
    加载提示显示()
    help := Trim(help, " `t`n")

    ; 生成替换代码
    NeedleRegEx := "m)(\s*)(<!-- 开始：抽取模块帮助 -->)([\s\S]*)\r?\n(\s*)(<!-- 结束：抽取模块帮助 -->)"
    Replacement := "$1$2`n" help "`n$4$5"
    targetREADME := RegExReplace(sourceREADME, NeedleRegEx, Replacement, Replaces)

    ; 检查替换情况
    if (!Replaces){
        MsgBox % "加载模块帮助遇到错误。`n请更新 CapsLockX"
        MsgBox % targetREADME
        Return sourceREADME
    }

    Return targetREADME
}
LoadModules(ModulesRunner, ModulesLoader){
    FileEncoding UTF-8
    ; 列出模块文件 然后 排序
    ModuleFiles := ""
    ; loop, Files, %CapsLockX_PathModules%\*.ahk, R ; Recurse into subfolders.
    loop, Files, %CapsLockX_PathModules%\*.ahk, ; NOT Recurse into subfolders.
        ModuleFiles .= A_LoopFileName "`n"
    ModuleFiles := Trim(ModuleFiles, "`n")
    Sort ModuleFiles

    ; 生成模块加载代码
    code_setup := ""
    code_include := ""
    i := 0
    loop, Parse, ModuleFiles, `n
    {
        i++
        ; 匹配模块名
        ModuleFile := A_LoopField
        re := RegExMatch(A_LoopField, "O)(?:.*[.-])*(.*)\.ahk", Match)
        if (!re){

            Continue
        }
        ModuleName := Match[1]

        if (T%ModuleName%_Disabled){
            加载提示追加("禁用模块：" i " " ModuleName)
        } else {
            ; 这里引入模块代码
            清洗为_UTF8_WITH_BOM_型编码(CapsLockX_PathModules "\" ModuleFile)

            ; 导入模块
            code_setup .= "GoSub CapsLockX_ModuleSetup_" i "`n"
            code_include .= "`n" "#If" "`n" "`n"
            code_include .= "CapsLockX_ModuleSetup_" i ":" "`n"
            code_include .= " " " " " " " " "#Include " CapsLockX_PathModules "\" ModuleFile "`n"
            code_include .= "Return" "`n"
            加载提示追加("运行模块：" i " " ModuleName)
        }
    }
    加载提示显示()

    ; 拼接代码
    code_consts .= "; 请勿直接编辑本文件，以下内容由核心加载器自动生成。雪星/(20210318)" "`n"
    code_consts .= "global CapsLockX_PathModules := " """" CapsLockX_PathModules """" "`n"
    code_consts .= "global CapsLockX_PathCore := " """" CapsLockX_PathCore """" "`n"
    code_consts .= "global CapsLockX_Version := " """" CapsLockX_Version """" "`n"
    code_consts .= "global CapsLockX_VersionName := " """" CapsLockX_VersionName """" "`n"

    codeRunner .= code_consts "`n" code_setup
    codeLoader .= "Return" "`n" code_include

    FileDelete %ModulesRunner%
    FileAppend %codeRunner%, %ModulesRunner%
    FileDelete %ModulesLoader%
    FileAppend %codeLoader%, %ModulesLoader%
}

CapslockX_启动(){
    CoreAHK := CapsLockX_PathCore "\CapsLockX-Core.ahk"
    UpdatorAHK := CapsLockX_PathCore "\CapsLockX-Update.ahk"
    ; 为了避免运行时对更新模块的影响，先把 EXE 文件扔到 Temp 目录，然后再运行核心。
    AHK_EXE_ROOT_PATH := "CapsLockX.exe"
    AHK_EXE_CORE_PATH := "./Core/CapsLockX.exe"
    AHK_EXE_TEMP_PATH := A_Temp "/$CapsLockX.exe"
    FileCopy, %AHK_EXE_ROOT_PATH%, %AHK_EXE_TEMP_PATH%, 1
    if !FileExist(AHK_EXE_TEMP_PATH)
        FileCopy, %AHK_EXE_CORE_PATH%, %AHK_EXE_TEMP_PATH%, 1
    if !FileExist(AHK_EXE_TEMP_PATH)
        AHK_EXE_TEMP_PATH := AHK_EXE_ROOT_PATH
    ; 运行更新组件
    ; ToolTip % A_ScriptDir
    Run %AHK_EXE_TEMP_PATH% %UpdatorAHK%, %A_ScriptDir%
    ; 运行核心
    RunWait %AHK_EXE_TEMP_PATH% %CoreAHK%, %A_ScriptDir%

    if (ErrorLevel){
        MsgBox, 4, CapsLockX 错误, CapsLockX 异常退出，是否重载？
        IfMsgBox No
        return
        Reload
    }else{
        TrayTip, CapsLockX 退出, CapsLockX 已退出。
        Sleep, 1000
    }
    ExitApp
}
