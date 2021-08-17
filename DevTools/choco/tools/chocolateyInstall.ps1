$ErrorActionPreference = 'Stop';
$packageName    = 'CapsLockX'
$toolsDir       = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$zipFilePath    = "$toolsDir\\CapsLockX.zip"
$packageArgs = @{
  packageName   = $env:ChocolateyPackageName
}
Get-ChocolateyUnzip -FileFullPath $zipFilePath -Destination $toolsDir
