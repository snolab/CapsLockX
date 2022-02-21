; ========== CapsLockX ==========
; 名称：CLX开机运行
; 描述：用于把clx添加到用户startup文件夹。
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v0.0.1
; ========== CapsLockX ==========

return

CapsLockX_MakeStartup()
{
    content = cd "%A_WorkingDir%" && start "" CapsLockX.exe
    startCMDPath := APPDATA "\Microsoft\Windows\Start Menu\Programs\Startup\capslockx-startup.cmd"
    FileDelete, %startCMDPath%
    FileAppend, %content%, %startCMDPath%
    cmdView := "explorer /select, " """" startCMDPath """"
    run % cmdView
    TrayTip 已在Startup文件夹添加CLX的开机自启动，请确认。
}