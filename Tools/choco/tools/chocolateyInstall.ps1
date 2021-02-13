$packageName    = 'CapsLockX'
$url            = 'https://github.com/snomiao/CapsLockX/archive/master.zip'
$toolsDir       = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"

Install-ChocolateyZipPackage -PackageName "$packageName" `
                             -Url "$url" `
                             -UnzipLocation "$toolsDir"
#  -Url64 "$url64" `
#  -Checksum "$checksum" `
#  -ChecksumType "$checksumType" `
#  -Checksum64 "$checksum64" `
#  -ChecksumType64 "$checksumType64"
