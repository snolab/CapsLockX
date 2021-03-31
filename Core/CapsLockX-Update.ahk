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

if(T_CheckUpdate)
    CapsLockX_检查更新()
Sleep, 5000
return

CapsLockX_更新提示(msg){
    ; TrayTip CapsLockX 更新, %msg%
    ToolTip, CapsLockX 更新：%msg%
}

CapsLockX更新通过gitpull(){
    gitUpdateResult := CapsLockX_RunSilent("cmd /c git fetch && git pull")
    if(Trim(gitUpdateResult, "`t`r`n ") == "Already up to date."){
        CapsLockX_更新提示("CapsLockX 已是最新")
        return 1
    }
    if(gitUpdateResult){
        MsgBox, %gitUpdateResult%
        CapsLockX_更新提示(gitUpdateResult)
        return 0
    }
}
CapsLockX更新_Util_VersionCompare(other,local) {
    ; from [(1) Simple version comparison - AutoHotkey Community](https://www.autohotkey.com/boards/viewtopic.php?f=6&t=5959)
    ver_other:=StrSplit(other,".")
    ver_local:=StrSplit(local,".")
    for _index, _num in ver_local
        if ( (ver_other[_index]+0) > (_num+0) )
        return 1
    else if ( (ver_other[_index]+0) < (_num+0) )
        return 0
    return 0
}
CapsLockX更新通过github(){
    UrlDownloadToFile, https://github.com/snomiao/CapsLockX/raw/master/Tools/version.txt, Tools/new-version.txt
    FileRead, remoteVersion, Tools/new-version.txt
    url := "https://github.com/snomiao/CapsLockX/archive/refs/tags/v" remoteVersion ".zip" ; release
    ; url := "https://github.com/snomiao/CapsLockX/archive/refs/heads/master.zip" ; newest.. CapsLockX-master
    file := A_Temp "\CapsLockX-Update-" remoteVersion ".zip"
    folder := A_Temp "\CapsLockX-Update-" remoteVersion
    programFolder := A_Temp "\CapsLockX-Update-" remoteVersion "\CapsLockX-" remoteVersion
    FileCreateDir %folder%
    CapsLockX_更新提示("正在从github下载新版本...")
    UrlDownloadToFile %url%, %file%
    CapsLockX_更新提示("正在解压...")
    RunWait PowerShell.exe -Command Expand-Archive -LiteralPath '%file%' -DestinationPath '%folder%' -Force,, Hide
    CapsLockX_更新提示("解压完成...")
    FileCopy, ./CapsLockX-Config.ini, %programFolder%/CapsLockX-Config.ini, 1
    Run explorer /select`,%programFolder%
    CapsLockX_更新提示("已自动打开新版本文件夹，请把它手动复制到你需要的目录。")
    Return 1
}
CapsLockX_检查更新(){
    CapsLockX_更新提示("正在检查更新")
    ; 
    UrlDownloadToFile, https://github.com/snomiao/CapsLockX/raw/master/Tools/version.txt, Tools/new-version.txt
    FileRead, remoteVersion, Tools/new-version.txt
    if(!remoteVersion){
        UrlDownloadToFile, https://cdn.jsdelivr.net/gh/snomiao/CapsLockX@master/Tools/version.txt, Tools/new-version-cdn.txt
        FileRead, remoteVersion, Tools/new-version-cdn.txt
    }
    if(!remoteVersion){
        CapsLockX_更新提示("更新检查失败，请检查网络")
        Return
    }
    FileRead, version, Tools/version.txt
    if(CapsLockX更新_Util_VersionCompare(remoteVersion, version))
        CapsLockX_更新提示("发现新版本：" remoteVersion "`n当前版本：" version "`n准备更新")
    else
        CapsLockX_更新提示("已经是最新版本")
    return 1
    if(!T_DownloadUpdate)
        Return 1
    if(CapsLockX更新通过gitpull())
        Return 1
    if(CapsLockX更新通过github())
        Return 1
}

; CapsLockX_检查更新:
;     CapsLockX_检查更新()
;     SetTimer, CapsLockX_检查更新, Off
; return
