#SingleInstance Force ; 跳过对话框并自动替换旧实例

; 计划用这个作更新库
; [jsdelivr/jsdelivr: A free, fast, and reliable Open Source CDN for npm and GitHub]( https://github.com/jsdelivr/jsdelivr )


url2 := "https://ws.codeku.me/snomiao/CapsLockX/zip/master"
; UnZip("C:\Test.zip", "C:\Temp\Test")

urlPackageJson := "https://github.com/snomiao/CapslockX/raw/master/package.json"
urlPackage := "https://github.com/snomiao/CapsLockX/archive/master.zip"

; urlPackageJson := "https://github.com/snomiao/CapslockX/raw/master/package.json"
; urlPackage := "https://github.com/snomiao/CapsLockX/archive/master.zip"

MsgBox, start
; UrlDownloadToFile, %urlPackage%, A_Temp . "\package-check.json"
UrlDownloadToFile, %urlPackage%, ".\package-check.json"
MsgBox, done