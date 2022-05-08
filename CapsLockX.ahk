; ========== CapsLockX ==========
; 名称：CapsLockX 程序入口
; 描述：用于生成模块加载组件、集成模块帮助文件、智能重载 CapsLockX 核心等功能。
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 编码：UTF-8 with BOM
; 版权：Copyright © 2017-2022 Snowstar Laboratory. All Rights Reserved.
; 感谢：张工 QQ: 45289331 参与调试
; LICENCE: GNU GPLv3
; ========== CapsLockX ==========
if (A_IsAdmin) {
    #SingleInstance Force ; 管理员权限下跳过对话框并自动替换旧实例
} else {
    #SingleInstance, Off  ; 普通用户无法替换管理员权限实例，故 Off，使用多开的方式修改配置文件触发管理员权限实例重载
}
#NoTrayIcon ; 隐藏托盘图标
SetWorkingDir, %A_ScriptDir%

#Include %A_ScriptDir%/Core/CapsLockX-Config.ahk

#Include %A_ScriptDir%/Core/CapsLockX-RunSilent.ahk

global CapsLockX_模块路径 := "./Modules"
global CapsLockX_核心路径 := "./Core"
; 版本
global CapsLockX_Version
FileRead, CapsLockX_Version, ./Core/version.txt
CapsLockX_Version := CapsLockX_Version ? CapsLockX_Version : "未知版本"

global CapsLockX_VersionName := "v" CapsLockX_Version
; 加载过程提示
global 显示加载提示 := 1
global loadingTips := ""

; 对 核心模块 进行 编码清洗
清洗为_UTF8_WITH_BOM_型编码(CapsLockX_核心路径 "/CapsLockX-Config.ahk")
清洗为_UTF8_WITH_BOM_型编码(CapsLockX_核心路径 "/CapsLockX-Core.ahk")
清洗为_UTF8_WITH_BOM_型编码(CapsLockX_核心路径 "/CapsLockX-RunSilent.ahk")
清洗为_UTF8_WITH_BOM_型编码(CapsLockX_核心路径 "/CapsLockX-Update.ahk")

; 复制用户模块
; TODO FIX：如果CLX已经开了的话，这一步会触发重启，这可能会导致一些文件冲突的BUG……
FileDelete, %CapsLockX_模块路径%/*.user.ahk
FileDelete, %CapsLockX_模块路径%/*.user.md
FileCopy %CapsLockX_配置目录%/*.user.ahk, %CapsLockX_模块路径%/, 1
FileCopy %CapsLockX_配置目录%/*.user.md, %CapsLockX_模块路径%/, 1

; 加载模块
global CapsLockX_ModulesRunner := CapsLockX_核心路径 "/CapsLockX-ModulesRunner.ahk"
global CapsLockX_ModulesLoader := CapsLockX_核心路径 "/CapsLockX-ModulesLoader.ahk"
; LoadModules(CapsLockX_ModulesRunner, CapsLockX_ModulesLoader)

; 判断安装方式
global NPM全局安装也 := InStr(A_ScriptFullPath, APPDATA) == 1 && InStr("node_modules", A_ScriptFullPath)
global GIT仓库安装也 := "true" == Trim(CapsLockX_RunSilent("cmd /c git rev-parse --is-inside-work-tree"), "`r`n`t` ")

;
模块帮助向README编译()
; 隐藏 ToolTip
ToolTip

; 当 CI_TEST 启用时，仅测试编译效果，不启动核心
EnvGet ENVIROMENT, ENVIROMENT
; msgbox % DllCall("GetCommandLine", "str")
; msgbox % !!RegExMatch(DllCall("GetCommandLine", "str"), "/CI_TEST")

if ("CI_TEST" == ENVIROMENT || !!RegExMatch(DllCall("GetCommandLine", "str"), "/CI_TEST")) {
    tooltip % "[INFO] MODULE LOAD OK, SKIP CORE"
    OutputDebug, % "[INFO] MODULE LOAD OK, SKIP CORE"
    ExitApp
} else {
    CapsLockX启动()
}
; #Persistent
; SetTimer, CapsLockX启动, -1
Return

模块帮助向README编译(){
    ; 编译README.md
    INPUT_README_FILE := "./docs/README.md"
    FileEncoding, UTF-8-Raw
    FileRead, source, %INPUT_README_FILE%
    
    ; 编译一次
    global 显示加载提示
    显示加载提示 := 0
    target := 模块编译和帮助README更新(source)
    if (target == source) {
        return "NOT_CHANGED"
    }
    ; 如果不一样，就再编译一次，并且显示加载提示
    显示加载提示 := 1
    加载提示追加("模块帮助有变更")
    ; 然后进行稳定性检查
    source := 模块编译和帮助README更新(target)
    if (target != source) {
        MsgBox % "如果你看到了这个，请联系雪星（QQ:997596439），这里肯定有 BUG……(20200228)"
    }
    ; 输出到 docs/readme.md （用于 github-pages ）
    ; docs_target := 模块编译和帮助README更新(source, 1)
    FileDelete ./docs/README.md
    FileAppend %target%, ./docs/README.md, UTF-8-Raw
    
    ; 输出根目录 README.md （用于 github 首页）
    FileDelete ./README.md
    PREFIX := "<!-- THIS FILE IS GENERATED PLEASE MODIFY DOCS/README -->`n`n"
    
    ; replace docs/media
    StringReplace, target, target, % "(./", % "(./docs/", All
    StringReplace, target, target, % "( ./", % "( ./docs/", All
    ; StringReplace, target, target, ./media/, ./docs/media/, All
    ; StringReplace, target, target, ./docs/, ./docs/, All
    FileAppend %PREFIX%%target%, ./README.md, UTF-8-Raw
    ; Reload
    ; ExitApp
}
加载提示追加(msg, clear = 0){
    global 显示加载提示
    if(!显示加载提示) return
    if (clear || loadingTips == "") {
        loadingTips := "CapsLockX " CapsLockX_Version "`n"
    }
    loadingTips .= msg "`n"
}
加载提示显示(){
    ToolTip % loadingTips
    sleep 2000
}
模块编译和帮助README更新(sourceREADME, docs=""){
    FileEncoding UTF-8-Raw
    ; 列出模块文件
    ModuleFiles := ""
    loop, Files, %CapsLockX_模块路径%\*.ahk
    {
        ; Do not Recurse into subfolders. 子文件夹由模块自己去include去加载
        ModuleFiles .= A_LoopFileName "`n"
    }
    ModuleFiles := Trim(ModuleFiles, "`n")
    Sort ModuleFiles
    ; 生成帮助
    全部帮助 := ""
    i := 0
    loop, Parse, ModuleFiles, `n
    {
        i++
        ; 匹配模块名
        模块文件 := A_LoopField
        匹配结果 := false
        匹配结果 := 匹配结果 || RegExMatch(A_LoopField, "O)((?:.*[-])*)(.*?)(?:\.user)?\.ahk", Match)
        匹配结果 := 匹配结果 || RegExMatch(A_LoopField, "O)((?:.*[-])*)(.*?)(?:\.用户)?\.ahk", Match)
        if (!匹配结果) {
            Continue
        }
        模块文件名称 := Match[1] Match[2]
        模块名称 := Match[2]
        模块帮助内容 := ""
        模块帮助文件 := ""
        if (!模块帮助内容) {
            模块帮助文件 := CapsLockX_模块路径 "/" 模块名称 ".md"
            if (FileExist(模块帮助文件)) {
                FileRead, 模块帮助内容, %模块帮助文件%
            }
        }
        if (!模块帮助内容) {
            模块帮助文件 := CapsLockX_模块路径 "/" 模块文件名称 ".md"
            if (FileExist(模块帮助文件)) {
                FileRead, 模块帮助内容, %模块帮助文件%
            }
        }
        
        ; 加载模块描述
        FileRead, 模块文件内容, % CapsLockX_模块路径 "/" 模块文件
        matchPos := RegExMatch(模块文件内容, "mi)^; 描述：(.*)", 模块描述)
        T%模块名称%_Disabled := CapsLockX_Config("ModuleDisable", "T" 模块名称 "_Disabled", 0, "是否禁用模块：" 模块名称 (模块描述1 ? " - " 模块描述1 : "") )
        
        if (模块帮助内容) {
            模块帮助内容 := Trim(模块帮助内容, " `t`n")
            加载提示追加("加载模块帮助：" + i + "-" + 模块名称)
            
            全部帮助 .= "<!-- 模块文件名：" Match[1] Match[2] ".ahk" "-->" "`n`n"
            ; 替换标题层级
            模块帮助内容 := RegExReplace(模块帮助内容, "m)^#", "###")
            
            ; 替换资源链接的相对目录（图片gif等）
            FileCopy, %CapsLockX_模块路径%\*.gif, .\docs\media\, 1
            FileCopy, %CapsLockX_模块路径%\*.png, .\docs\media\, 1
            模块帮助内容 := RegExReplace(模块帮助内容, "m)\[(.*)\]\(\s*?\.\/(.*?)\)", "[$1]( ./media/$2 )")
            ; 没有标题的，给自动加标题
            if (!RegExMatch(模块帮助内容, "^#")) {
                if (T%模块名称%_Disabled) {
                    全部帮助 .= "### " 模块名称 "模块（禁用）" "`n"
                } else {
                    全部帮助 .= "### " 模块名称 "模块" "`n"
                }
            }
            全部帮助 .= 模块帮助内容 "`n`n"
        }
        if (T%模块名称%_Disabled) {
            加载提示追加("跳过模块：" i " " 模块名称)
        } else {
            ; 这里引入模块代码
            清洗为_UTF8_WITH_BOM_型编码(CapsLockX_模块路径 "/" 模块文件)
            ; 导入模块
            模块初始化代码 .= "GoSub CapsLockX_ModuleSetup_" i "`n"
            模块导入代码 .= "`n" "#If" "`n" "`n"
            模块导入代码 .= "CapsLockX_ModuleSetup_" i ":" "`n"
            if (模块帮助内容 && 模块帮助文件) {
                模块导入代码 .= " " " " " " " " "CapsLockX_THIS_MODULE_HELP_FILE_PATH = " 模块帮助文件 "`n"
            } else {
                模块导入代码 .= " " " " " " " " "CapsLockX_THIS_MODULE_HELP_FILE_PATH := """"" "`n"
            }
            模块导入代码 .= " " " " " " " " "#Include " CapsLockX_模块路径 "/" 模块文件 "`n"
            模块导入代码 .= "Return" "`n"
            加载提示追加("运行模块：" i " " 模块名称)
        }
    }
    加载提示显示()
    
    ; 拼接模块加载器代码
    常量语句 .= "; 请勿直接编辑本文件，以下内容由核心加载器自动生成。雪星/(20210318)" "`n"
    常量语句 .= "global CapsLockX_模块路径 := " """" CapsLockX_模块路径 """" "`n"
    常量语句 .= "global CapsLockX_核心路径 := " """" CapsLockX_核心路径 """" "`n"
    常量语句 .= "global CapsLockX_Version := " """" CapsLockX_Version """" "`n"
    常量语句 .= "global CapsLockX_VersionName := " """" CapsLockX_VersionName """" "`n"
    
    模块运行器 .= 常量语句 "`n" 模块初始化代码
    模块加载器 .= "Return" "`n" 模块导入代码
    
    FileEncoding UTF-8
    FileDelete %CapsLockX_ModulesRunner%
    FileAppend %模块运行器%, %CapsLockX_ModulesRunner%
    FileDelete %CapsLockX_ModulesLoader%
    FileAppend %模块加载器%, %CapsLockX_ModulesLoader%
    
    加载提示显示()
    全部帮助 := Trim(全部帮助, " `t`n")
    
    ; 生成 README 替换代码
    NeedleRegEx := "m)(\s*)(<!-- 开始：抽取模块帮助 -->)([\s\S]*)\r?\n(\s*)(<!-- 结束：抽取模块帮助 -->)"
    Replacement := "$1$2`n" 全部帮助 "`n$4$5"
    targetREADME := RegExReplace(sourceREADME, NeedleRegEx, Replacement, Replaces)
    
    ; 检查 README 替换情况
    if (!Replaces) {
        Run https://capslockx.snomiao.com/
        MsgBox % "加载模块帮助遇到错误。`n已为你打开 CapsLockX 主页，请手动更新 CapsLockX"
        MsgBox % targetREADME
        Return sourceREADME
    }
    
    Return targetREADME
}
CapsLockX启动(){
    CoreAHK := CapsLockX_核心路径 "\CapsLockX-Core.ahk"
    UpdatorAHK := CapsLockX_核心路径 "\CapsLockX-Update.ahk"
    ; 为了避免运行时对更新模块的影响，先把 EXE 文件扔到 Temp 目录，然后再使用 Temp 里的 AHK 来运行本核心。
    AHK_EXE_ROOT_PATH := "CapsLockX.exe"
    AHK_EXE_CORE_PATH := "./Core/CapsLockX.exe"
    AHK_EXE_TEMP_PATH := A_Temp "/CapsLockX-AHK.exe"
    FileCopy, %AHK_EXE_ROOT_PATH%, %AHK_EXE_TEMP_PATH%, 1
    if (!FileExist(AHK_EXE_TEMP_PATH)) {
        FileCopy, %AHK_EXE_CORE_PATH%, %AHK_EXE_TEMP_PATH%, 1
    }
    if (!FileExist(AHK_EXE_TEMP_PATH)) {
        AHK_EXE_TEMP_PATH := AHK_EXE_ROOT_PATH
    }
    ; 运行更新组件
    Run %AHK_EXE_TEMP_PATH% %UpdatorAHK%, %A_ScriptDir%
    
    ; 运行核心
    ; 启动
    global T_AskRunAsAdmin := CapsLockX_ConfigGet("Core", "T_AskRunAsAdmin", 0)
    adminCommand := RegExMatch(DllCall("GetCommandLine", "str"), "/admin")
    if (!A_IsAdmin && T_AskRunAsAdmin || adminCommand) {
        RunWait *RunAs %AHK_EXE_TEMP_PATH% %CoreAHK%, %A_ScriptDir%
    } else {
        RunWait %AHK_EXE_TEMP_PATH% %CoreAHK%, %A_ScriptDir%
    }
    if (ErrorLevel) {
        MsgBox, 4, CapsLockX 错误, CapsLockX 异常退出，是否重载？
        IfMsgBox No
        return
        Reload
    } else {
        TrayTip, CapsLockX 退出, CapsLockX 已退出。
        Sleep, 1000
    }
    ExitApp
}
