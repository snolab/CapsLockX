$ErrorActionPreference = 'Stop'; # stop on all errors
$packageName    = 'CapsLockX'
$toolsDir       = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$zipFilePath    = "$toolsDir\\CapsLockX.zip"
# $url            = 'https://github.com/snomiao/CapsLockX/archive/master.zip'
# $checksum       = '%checksum%'
# $checksumType   = 'md5'
$packageArgs = @{
  packageName   = $env:ChocolateyPackageName
  # unzipLocation = $toolsDir
  # url           = $url
  # softwareName  = 'CapsLockX' #part or all of the Display Name as you see it in Programs and Features. It should be enough to be unique
  # checksum      = '%checksum%'
  # checksumType  = 'sha256' #default is md5, can also be sha1, sha256 or sha512
}
Get-ChocolateyUnzip -FileFullPath $zipFilePath -Destination $toolsDir
# Install-ChocolateyZipPackage -PackageName "$packageName" `
#                              -UnzipLocation "$toolsDir"
#                              -Url "$url" `
#                              -Checksum "$checksum" `
#                              -ChecksumType "$checksumType"
