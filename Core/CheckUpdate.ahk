#SingleInstance Force ; 跳过对话框并自动替换旧实例

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