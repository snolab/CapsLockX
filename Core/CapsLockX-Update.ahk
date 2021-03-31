; ========== CapsLockX ==========
; Note：Save as UTF-8 with BOM please
; 名称：CapsLockX 更新模块
; 作者：snomiao (snomiao@gmail.com)
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2021.03.24
; 版权：Copyright © 2017-2021 Snowstar Laboratory. All Rights Reserved.
; LICENCE: GNU GPLv3
; ========== CapsLockX ==========
#Include Core/CapsLockX-RunSilent.ahk
; 载入设定
global CapsLockXConfigPath := "./CapsLockX-Config.ini"
#Include Core/CapsLockX-Config.ahk

global T_CheckUpdate := CapsLockX_Config("Core", "T_CheckUpdate", 1, "自动检查更新")
global T_DownloadUpdate := CapsLockX_Config("Core", "T_DownloadUpdate", 1, "自动下载更新包")
global CLXU_Updated :=1
global CLXU_Fail := 2
global CLXU_AlreadyLatest := 4
if(T_CheckUpdate)
    CapsLockX_检查更新()
Sleep, 5000
return

CapsLockX_更新提示(msg){
    ; TrayTip CapsLockX 更新, %msg%
    ToolTip, CapsLockX 更新：%msg%
}

CapsLockX更新通过gitpull(tryAgainFlag:=0){
    gitUpdateResult := CapsLockX_RunSilent("cmd /c git fetch && git pull")
    if(Trim(gitUpdateResult, "`t`r`n ") == "Already up to date."){
        CapsLockX_更新提示("CapsLockX 已是最新")
        return CLXU_AlreadyLatest
    }
    if(gitUpdateResult){
        ; MsgBox, %gitUpdateResult%
        ; CapsLockX_更新提示(gitUpdateResult)
        if(!tryAgainFlag)
            return CapsLockX更新通过gitpull("tryAgainFlag")
    }
    return CLXU_Fail
}
CapsLockX更新_Util_VersionCompare(remote, local) {
    ; from [(1) Simple version comparison - AutoHotkey Community](https://www.autohotkey.com/boards/viewtopic.php?f=6&t=5959)
    ver_other:=StrSplit(remote,".")
    ver_local:=StrSplit(local,".")
    for _index, _num in ver_local{
        if ( (ver_other[_index]+0) > (_num+0) ){
            CapsLockX_更新提示("发现新版本！准备更新：" "`n仓库版本：" remote "`n我的版本：" local)
            return 1
        }else if ( (ver_other[_index]+0) < (_num+0) ){
            ; CapsLockX_更新提示("当前已经是最新版本" "`n仓库版本：" remote "`n我的版本：" local)
            return -1
        }
        CapsLockX_更新提示("当前已经是最新版本" "`n仓库版本：" remote "`n我的版本：" local)
        return 0
    }
}
CapsLockX更新通过git仓库HTTP(版本文件地址, 归档文件前缀){
    ; get latest version
    UrlDownloadToFile, %版本文件地址%, Tools/new-version.txt
    FileRead, version, Tools/version.txt
    FileRead, remoteVersion, Tools/new-version.txt
    if(!remoteVersion)
        return CLXU_Fail
    if(!version)
        return CLXU_Fail
    ; version compare
    ver_cmp := CapsLockX更新_Util_VersionCompare(remoteVersion, version)
    if(ver_cmp<0)
        return CLXU_AlreadyLatest
    if(ver_cmp==0)
        return CLXU_AlreadyLatest
    ; url := 归档文件前缀 "/master.zip" ; latest
    url := 归档文件前缀 "/v" remoteVersion ".zip" ; release
    ; download and unzip
    file := A_Temp "\CapsLockX-Update-" remoteVersion ".zip"
    unzipFolder := A_Temp "\CapsLockX-Update-" remoteVersion
    programFolder := A_Temp "\CapsLockX-Update-" remoteVersion "\CapsLockX-" remoteVersion
    FileCreateDir %unzipFolder%
    CapsLockX_更新提示("正在从github下载新版本...")
    UrlDownloadToFile %url%, %file%
    CapsLockX_更新提示("正在解压...")
    RunWait PowerShell.exe -Command Expand-Archive -LiteralPath '%file%' -DestinationPath '%unzipFolder%' -Force,, Hide
    CapsLockX_更新提示("解压完成...")
    ; migrate configs
    FileCopy, ./CapsLockX-Config.ini, %programFolder%/CapsLockX-Config.ini, 1
    Run explorer /select`,%programFolder%
    Run explorer /select`,.
    CapsLockX_更新提示("已自动打开新版本文件夹，请把它手动复制到当前软件目录。")
    ; TODO REPLACE CURRENT FOLDER
    Return CLXU_Updated
}

CapsLockX_检查更新(){
    CapsLockX_更新提示("正在检查更新： gitpull")
    if(CLXU_AlreadyLatest & CapsLockX更新通过gitpull())
        return CLXU_AlreadyLatest
    版本文件地址:="https://github.com/snomiao/CapsLockX/raw/master/Tools/version.txt,"
    归档文件前缀:="https://github.com/snomiao/CapsLockX/archive"
    CapsLockX_更新提示("正在检查更新： github")
    if(CLXU_Updated & CapsLockX更新通过git仓库HTTP(版本文件地址, 归档文件前缀))
        return
    版本文件地址:="https://gitee.com/snomiao/CapslockX/raw/master/Tools/version.txt"
    归档文件前缀:="https://gitee.com/snomiao/CapslockX/repository/archive"
    CapsLockX_更新提示("正在检查更新： gitee")
    if(CLXU_Updated & CapsLockX更新通过git仓库HTTP(版本文件地址, 归档文件前缀))
        return
    版本文件地址:="https://cdn.jsdelivr.net/gh/snomiao/CapsLockX@master/Tools/version.txt"
    归档文件前缀:="https://gitee.com/snomiao/CapslockX/repository/archive"
    CapsLockX_更新提示("正在检查更新： cdn")
    if(CLXU_Updated & CapsLockX更新通过git仓库HTTP(版本文件地址, 归档文件前缀))
        return
}
