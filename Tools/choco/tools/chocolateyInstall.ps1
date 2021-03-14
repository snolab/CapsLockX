$packageName    = 'CapsLockX'
$toolsDir       = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$url            = 'https://github.com/snomiao/CapsLockX/archive/master.zip'
$checksum       = '...'
$checksumType   = 'md5'

Install-ChocolateyZipPackage -PackageName "$packageName" `
                             -UnzipLocation "$toolsDir"
                             -Url "$url" `
                             -Checksum "$checksum" `
                             -ChecksumType "$checksumType" `
                            #  -Url64 "$url64" `
                            #  -Checksum64 "$checksum64" `
                            #  -ChecksumType64 "$checksumType64"
