; ========== CapsLockX ==========
; Note：Save as UTF-8 with BOM please
; 名称：CapsLockX 更新模块
; 作者：snomiao (snomiao@gmail.com)
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2021.03.24
; 版权：Copyright © 2017-2021 Snowstar Laboratory. All Rights Reserved.
; LICENCE: GNU GPLv3
; ========== CapsLockX ==========

; 防多开
#SingleInstance, ignore

; 载入设定、检查配置文件
global CapsLockX_配置路径
CapsLockX_配置路径 := "./CapsLockX-Config.ini"
if(!FileExist(CapsLockX_配置路径))
    CapsLockX_配置路径 := "../CapsLockX-Config.ini"
if(!FileExist(CapsLockX_配置路径)){
    MsgBox 更新失败：配置文件不存在
    ExitApp
}
; 加载模块（这里更新模块可能由 CapsLockX 加载也可能自己启动）
#Include *i Core/CapsLockX-Config.ahk
#Include *i Core/CapsLockX-RunSilent.ahk
#Include *i ./CapsLockX-Config.ahk
#Include *i ./CapsLockX-RunSilent.ahk

if(FileExist(CapsLockX_配置路径)){
    global T_CheckUpdate
    global T_DownloadUpdate
    T_CheckUpdate := CapsLockX_Config("Update", "T_CheckUpdate", 1, "自动检查更新")
    T_DownloadUpdate := CapsLockX_Config("Update", "T_DownloadUpdate", 1, "自动下载更新包")
}
global CapsLockX_Update_Updated :=1
global CapsLockX_Update_Fail := 2
global CapsLockX_Update_AlreadyLatest := 4
; Sleep, 5000
if(T_CheckUpdate)
    CapsLockX_更新检查()
Sleep, 5000
return

CapsLockX_更新提示(msg){
    ; TrayTip CapsLockX 更新, %msg%
    ToolTip, CapsLockX 更新：%msg%
}
CapsLockX_仓库版本号比对(remote, local){
    ; from [(1) Simple version comparison - AutoHotkey Community](https://www.autohotkey.com/boards/viewtopic.php?f=6&t=5959)
    ver_local := StrSplit(local,".")
    ver_other := StrSplit(remote,".")
    for _index, _num in ver_local{
        if ( (ver_other[_index]+0) > (_num+0) ){
            CapsLockX_更新提示("发现新版本！准备更新：" "`n仓库版本：" remote "`n我的版本：" local)
            return 1
        }else if ( (ver_other[_index]+0) < (_num+0) ){
            ; CapsLockX_更新提示("当前已经是最新版本" "`n仓库版本：" remote "`n我的版本：" local)
            return -1
        }
        ; CapsLockX_更新提示("当前已经是最新版本" "`n仓库版本：" remote "`n我的版本：" local)
        return 0
    }
}
CapsLockX_尝试通过gitpull_更新(tryAgainFlag:=0){
    gitUpdateResult := CapsLockX_RunSilent("cmd /c git fetch && git pull")
    if(Trim(gitUpdateResult, "`t`r`n ") == "Already up to date."){
        ; CapsLockX_更新提示("CapsLockX 已是最新")
        return CapsLockX_Update_AlreadyLatest
    }
    if(gitUpdateResult){
        ; MsgBox, %gitUpdateResult%
        if(tryAgainFlag){
            ; 通常是有错误发生。
            ; CapsLockX_更新提示("错误：" gitUpdateResult)
            Return CapsLockX_Update_Fail
        }else{
            return CapsLockX_尝试通过gitpull_更新("tryAgainFlag")
        }
    }
    return CapsLockX_Update_Fail
}
CapsLockX_通过git仓库HTTP_更新(版本文件地址, 归档文件前缀){
    ; get latest version
    CapsLockX_更新提示("正在获取新版本号...地址：" 版本文件地址)
    UrlDownloadToFile, %版本文件地址%, Tools/new-version.txt
    FileRead, version, Tools/version.txt
    FileRead, remoteVersion, Tools/new-version.txt
    if(!remoteVersion)
        return CapsLockX_Update_Fail
    if(!version)
        return CapsLockX_Update_Fail
    ; version compare
    ver_cmp := CapsLockX_仓库版本号比对(remoteVersion, version)
    if(ver_cmp<0)
        return CapsLockX_Update_AlreadyLatest
    ; if(ver_cmp==0)
    ;     return CapsLockX_Update_AlreadyLatest
    if(!T_DownloadUpdate)
        return
    ; url := 归档文件前缀 "/master.zip" ; latest
    url := 归档文件前缀 "/v" remoteVersion ".zip" ; release
    ; download and unzip
    file := A_Temp "\CapsLockX-Update-" remoteVersion ".zip"
    unzipFolder := A_Temp "\CapsLockX-Update-" remoteVersion
    programFolder := A_Temp "\CapsLockX-Update-" remoteVersion "\CapsLockX-" remoteVersion
    FileCreateDir %unzipFolder%
    CapsLockX_更新提示("正在下载新版本...地址：" 归档文件前缀)
    UrlDownloadToFile %url%, %file%
    CapsLockX_更新提示("正在解压...")
    RunWait PowerShell.exe -Command Expand-Archive -LiteralPath '%file%' -DestinationPath '%unzipFolder%' -Force,, Hide
    CapsLockX_更新提示("解压完成...")
    ; 迁移配置
    FileCopy, ./CapsLockX-Config.ini, %programFolder%/CapsLockX-Config.ini, 1
    FileCopy, ./Modules/*.user.ahk, %programFolder%/Modules/, 1
    FileCopy, ./Modules/*.user.md, %programFolder%/Modules/, 1
    Run explorer /select`, %programFolder%
    Run explorer /select`, %A_ScriptDir%
    CapsLockX_更新提示("解压完成，已打开新版本文件夹，请把它手动复制到当前软件目录。")
    ; TODO REPLACE CURRENT FOLDER
    Return CapsLockX_Update_Updated
}

CapsLockX_更新检查(){
    ; CapsLockX_更新提示("正在检查更新： gitpull")
    if(CapsLockX_Update_AlreadyLatest & CapsLockX_尝试通过gitpull_更新())
        return CapsLockX_Update_AlreadyLatest
    版本文件地址:="https://github.com/snomiao/CapsLockX/raw/master/Tools/version.txt,"
    归档文件前缀:="https://github.com/snomiao/CapsLockX/archive"
    ; CapsLockX_更新提示("正在检查更新： github")
    if(CapsLockX_Update_Updated & CapsLockX_通过git仓库HTTP_更新(版本文件地址, 归档文件前缀))
        return
    版本文件地址:="https://gitee.com/snomiao/CapslockX/raw/master/Tools/version.txt"
    归档文件前缀:="https://gitee.com/snomiao/CapslockX/repository/archive"
    ; CapsLockX_更新提示("正在检查更新： gitee")
    if(CapsLockX_Update_Updated & CapsLockX_通过git仓库HTTP_更新(版本文件地址, 归档文件前缀))
        return
    版本文件地址:="https://cdn.jsdelivr.net/gh/snomiao/CapsLockX@master/Tools/version.txt"
    归档文件前缀:="https://gitee.com/snomiao/CapslockX/repository/archive"
    ; CapsLockX_更新提示("正在检查更新： cdn")
    if(CapsLockX_Update_Updated & CapsLockX_通过git仓库HTTP_更新(版本文件地址, 归档文件前缀))
        return
}
