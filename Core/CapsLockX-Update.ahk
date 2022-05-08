; ========== CapsLockX ==========
; Note：Save as UTF-8 with BOM please
; 名称：CapsLockX 更新模块
; 作者：snomiao (snomiao@gmail.com)
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2021.03.24
; 版权：Copyright © 2017-2022 Snowstar Laboratory. All Rights Reserved.
; LICENCE: GNU GPLv3
; ========== CapsLockX ==========

; 防多开
#SingleInstance, ignore

; 加载模块（这里更新模块可能由 CapsLockX 加载也可能自己启动所以需要动态加载配置ahk）
#Include *i Core/CapsLockX-Config.ahk
#Include *i Core/CapsLockX-RunSilent.ahk
#Include *i ./CapsLockX-Config.ahk
#Include *i ./CapsLockX-RunSilent.ahk

if(FileExist(CapsLockX_配置路径)) {
    global T_CheckUpdate
    global T_DownloadUpdate
    ; 用ConfigGet防止触发自动重载
    T_CheckUpdate := CapsLockX_ConfigGet("Update", "T_CheckUpdate", 1)
    T_DownloadUpdate := CapsLockX_ConfigGet("Update", "T_DownloadUpdate", 1)
}
global CapsLockX_Update_Updated := 0x01
global CapsLockX_Update_Fail := 0x02
global CapsLockX_Update_AlreadyLatest := 0x04
global CapsLockX_Update_NeedUpdate := 0x08
global CapsLockX_Update_Stop := 0x10

; Sleep, 5000
if (T_CheckUpdate)
    CapsLockX_更新()
; CapsLockX_更新记录("发现新版本！准备更新：" "`n仓库版本：" remote "`n我的版本：" local)
; TODO
; TrayTip, CapsLockX 更新模块, 更新完成
; Sleep, 5000
return

CapsLockX_更新记录(msg){
    ; TrayTip CapsLockX 更新, %msg%
    ; ToolTip, CapsLockX 更新：%msg%
    ; MsgBox, CapsLockX 更新：%msg%
}
CapsLockX_更新提示(msg){
    ; TrayTip CapsLockX 更新, %msg%
    ; ToolTip, CapsLockX 更新：%msg%
    MsgBox, CapsLockX 更新：%msg%
}
CapsLockX_仓库版本号比对(remote, local){
    ; from [(1) Simple version comparison - AutoHotkey Community](https://www.autohotkey.com/boards/viewtopic.php?f=6&t=5959)
    ver_local := StrSplit(local, ".")
    ver_other := StrSplit(remote, ".")
    for _ in dex, _num in ver_local {
        if ( (ver_other[_index]+0) > (_num+0) ) {
            CapsLockX_更新记录("发现新版本！准备更新：" "`n仓库版本：" remote "`n我的版本：" local)
            return 1
        } else if ( (ver_other[_index]+0) < (_num+0) ) {
            ; CapsLockX_更新记录("当前已经是最新版本" "`n仓库版本：" remote "`n我的版本：" local)
            return -1
        }
        ; CapsLockX_更新记录("当前已经是最新版本" "`n仓库版本：" remote "`n我的版本：" local)
        return 0
    }
}
CapsLockX_通过npm更新尝试(){
    EnvGet, APPDATA, APPDATA
    NPM全局安装也 := InStr(A_ScriptFullPath, APPDATA) == 1 && InStr("node_modules", A_ScriptFullPath)
    if(!NPM全局安装也) {
        return CapsLockX_Update_Fail
    }
    CapsLockX_更新记录("当前版本由 npm i -g 安装，正在尝试通过 npm update -g capslockx 更新")
    更新成功 := 0
    || "SUCC" == CapsLockX_RunSilent("cmd /c cnpm update -g capslockx && echo SUCC || echo FAIL")
    || "SUCC" == CapsLockX_RunSilent("cmd /c npm update -g capslockx && echo SUCC || echo FAIL")
    return 更新成功 ? CapsLockX_Update_AlreadyLatest : CapsLockX_Update_Fail
}
CapsLockX_通过gitpull更新(tryAgainFlag := 0){
    GIT仓库安装也 := "true" == Trim(CapsLockX_RunSilent("cmd /c git rev-parse --is-inside-work-tree"), "`r`n`t` ")
    if(!GIT仓库安装也) {
        return CapsLockX_Update_Fail
    }
    CapsLockX_更新记录("当前版本由 git clone 安装，正在尝试通过 git pull 更新" tryAgainFlag)
    命令返回 := CapsLockX_RunSilent("cmd /c git fetch && git pull")
    if(Trim(命令返回, "`t`r`n ") == "Already up to date.") {
        CapsLockX_更新记录("CapsLockX 已是最新")
        return CapsLockX_Update_AlreadyLatest
    }
    if(命令返回) {
        if(tryAgainFlag) {
            ; 通常是有错误发生。
            CapsLockX_更新记录("git pull 错误：" 命令返回)
            Return CapsLockX_Update_Fail | CapsLockX_Update_Stop
        } else {
            return CapsLockX_通过gitpull更新("tryAgainFlag")
        }
    }
    return CapsLockX_Update_Fail | CapsLockX_Update_Stop
}
CapsLockX_通过发布包_更新(版本文件地址, 包网址){
    CapsLockX_更新记录("正在获取新版本号...地址：" 版本文件地址)
    UrlDownloadToFile, %版本文件地址%, Core/version-remote.txt
    FileRead, version, Core/version.txt
    FileRead, remoteVersion, Core/version-remote.txt
    if(!remoteVersion || !version) {
        return CapsLockX_Update_Fail
    }
    CapsLockX_更新记录("正在比对版本号...地址：" 版本文件地址)
    ver_cmp := CapsLockX_仓库版本号比对(remoteVersion, version)
    if(ver_cmp<0) {
        return CapsLockX_Update_Updated
    }
    ; if(ver_cmp==0)
    ;     return CapsLockX_Update_AlreadyLatest
    if(!T_DownloadUpdate) {
        return
    }
    包路径 := A_Temp "/CapsLockX-UpdatePackage" "/" remoteVersion ".zip"
    解压目录 := A_Temp "/CapsLockX-UpdatePackage" "/CapsLockX-" remoteVersion
    程序目录 := A_Temp "/CapsLockX-UpdatePackage" "/CapsLockX-" remoteVersion
    return CapsLockX_ZIP下载解压更新(包网址, 包路径, 解压目录, 程序目录)
}
CapsLockX_通过git仓库包_更新(版本文件地址, 归档文件前缀){
    CapsLockX_更新记录("正在获取新版本号...地址：" 版本文件地址)
    UrlDownloadToFile, %版本文件地址%, Core/version-remote.txt
    FileRead, version, Core/version.txt
    FileRead, remoteVersion, Core/version-remote.txt
    if(!remoteVersion || !version) {
        return CapsLockX_Update_Fail
    }
    CapsLockX_更新记录("正在比对版本号...地址：" 版本文件地址)
    ver_cmp := CapsLockX_仓库版本号比对(remoteVersion, version)
    if(ver_cmp<0) {
        return CapsLockX_Update_Updated
    }
    ; if(ver_cmp==0)
    ;     return CapsLockX_Update_AlreadyLatest
    if(!T_DownloadUpdate) {
        return
    }
    包网址 := 归档文件前缀 "/v" remoteVersion ".zip" ; release
    包路径 := A_Temp "\CapsLockX-UpdateArchive" "/" remoteVersion ".zip"
    解压目录 := A_Temp "\CapsLockX-UpdateArchive" "/" remoteVersion
    程序目录 := A_Temp "\CapsLockX-UpdateArchive" "/" remoteVersion "/CapsLockX-" remoteVersion
    return CapsLockX_ZIP下载解压更新(包网址, 包路径, 解压目录, 程序目录)
}
CapsLockX_ZIP下载解压更新(包网址, 包路径, 解压目录, 程序目录){
    FileCreateDir %解压目录%
    CapsLockX_更新记录("正在下载新版本...地址：" 包网址)
    UrlDownloadToFile %包网址%, %包路径%
    if(ErrorLevel) {
        return CapsLockX_Update_Fail
    }
    CapsLockX_更新记录("下载完成，正在解压...")
    RunWait PowerShell.exe -Command Expand-Archive -LiteralPath '%包路径%' -DestinationPath '%解压目录%' -Force, , Hide
    if(ErrorLevel) {
        msgbox CapsLockX 更新解压错误
        Run explorer /select`, %包路径%
        return CapsLockX_Update_Fail | CapsLockX_Update_Stop
    }
    ; 删除压缩包
    FileDelete, %包路径%
    
    if(!FileExist(程序目录)) {
        CapsLockX_更新记录("解压错误：未找到程序目录，将打开解压目录")
        Run explorer %解压目录%
        return CapsLockX_Update_Fail | CapsLockX_Update_Stop
    }
    CapsLockX_更新记录("解压完成...")
    
    CapsLockX_更新提示("解压完成，将打开新版本文件夹，请把它手动复制到当前软件目录。")
    Run explorer /select`, %程序目录%
    Run explorer /select`, %A_ScriptDir%
    ; TODO REPLACE CURRENT FOLDER
    Return CapsLockX_Update_Updated
}
CapsLockX_更新(){
    stopFlag := CapsLockX_Update_AlreadyLatest | CapsLockX_Update_Updated | CapsLockX_Update_Stop
    return false ; 依次尝试，直到更新成功或已是最新
    || stopFlag & CapsLockX_通过gitpull更新()
    || stopFlag & CapsLockX_通过npm更新尝试()
    || stopFlag & CapsLockX_通过jsdelivr发布包更新()
    || stopFlag & CapsLockX_通过snomiao发布包更新()
    || stopFlag & CapsLockX_通过github发布包更新()
    || stopFlag & CapsLockX_通过github仓库包更新()
    ; || stopFlag & CapsLockX_通过gitee仓库包更新()
    ; || stopFlag & CapsLockX_通过gitlab仓库包更新()
}
CapsLockX_更新测试(){
    msgbox 将 通过gitpull更新
    通过gitpull更新结果 := CapsLockX_通过gitpull更新()
    msgbox 通过gitpull更新结果
    msgbox 将 通过npm更新
    通过npm更新结果 := CapsLockX_通过npm更新尝试()
    msgbox 通过npm更新结果
    msgbox 将 通过jsdelivr发布包更新
    通过jsdelivr发布包更新结果 := CapsLockX_通过jsdelivr发布包更新()
    msgbox 通过jsdelivr发布包更新结果
    msgbox 将 通过snomiao发布包更新
    通过snomiao发布包更新结果 := CapsLockX_通过snomiao发布包更新()
    msgbox 通过snomiao发布包更新结果
    msgbox 将 通过github发布包更新
    通过github发布包更新结果 := CapsLockX_通过github发布包更新()
    msgbox 通过github发布包更新结果
    msgbox 将 通过github仓库包更新
    通过github仓库包更新结果 := CapsLockX_通过github仓库包更新()
    msgbox 通过github仓库包更新结果
    ; msgbox 将 通过gitee仓库包更新
    ; ; 通过gitee仓库包更新结果 := CapsLockX_通过gitee仓库包更新()
    ; msgbox 通过gitee仓库包更新结果
    ; msgbox 将 通过gitlab仓库包更新
    ; ; 通过gitlab仓库包更新结果 := CapsLockX_通过gitlab仓库包更新()
    ; msgbox 通过gitlab仓库包更新结果
}

CapsLockX_通过jsdelivr发布包更新(){
    CapsLockX_更新记录("正在通过jsdelivr发布包更新")
    版本文件地址:="https://cdn.jsdelivr.net/gh/snolab/CapsLockX@gh-pages/version.txt"
    发布包文件地址:="https://cdn.jsdelivr.net/gh/snolab/CapsLockX@gh-pages/CapsLockX-latest.zip"
    return CapsLockX_通过发布包_更新(版本文件地址, 发布包文件地址)
}
CapsLockX_通过snomiao发布包更新(){
    CapsLockX_更新记录("正在通过snomiao发布包更新")
    版本文件地址:="https://capslockx.snomiao.com/version.txt"
    发布包文件地址:="https://capslockx.snomiao.com/CapsLockX-latest.zip"
    return CapsLockX_通过发布包_更新(版本文件地址, 发布包文件地址)
}
CapsLockX_通过github发布包更新(){
    CapsLockX_更新记录("正在通过github发布包更新")
    版本文件地址:="https://github.com/snolab/CapsLockX/raw/gh-pages/version.txt"
    发布包文件地址:="https://github.com/snolab/CapsLockX/raw/gh-pages/CapsLockX-latest.zip"
    return CapsLockX_通过发布包_更新(版本文件地址, 发布包文件地址)
}

CapsLockX_通过github仓库包更新(){
    CapsLockX_更新记录("正在检查更新： github")
    版本文件地址:="https://github.com/snomiao/CapsLockX/raw/master/Core/version.txt"
    归档文件前缀:="https://github.com/snomiao/CapsLockX/archive"
    return CapsLockX_通过git仓库包_更新(版本文件地址, 归档文件前缀)
}
; CapsLockX_通过gitee仓库包更新(){
;     CapsLockX_更新记录("正在检查更新： gitee")
;     版本文件地址:="https://gitee.com/snomiao/CapslockX/raw/master/Core/version.txt"
;     归档文件前缀:="https://gitee.com/snomiao/CapslockX/repository/archive"
;     return CapsLockX_通过git仓库包_更新(版本文件地址, 归档文件前缀)
; }
; CapsLockX_通过gitlab仓库包更新(){
;     CapsLockX_更新记录("正在检查更新： gitlab")
;     版本文件地址:="https://gitlab.com/snomiao/CapsLockX/-/raw/master/Core/version.txt"
;     归档文件前缀:="https://gitlab.com/snomiao/CapsLockX/-/archive/master/CapsLockX-master.zip"
;     return CapsLockX_通过git仓库包_更新(版本文件地址, 归档文件前缀)
; }
