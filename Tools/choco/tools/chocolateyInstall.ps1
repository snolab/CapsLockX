$ErrorActionPreference = 'Stop'; # stop on all errors
$packageName    = 'CapsLockX'
$toolsDir       = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$url            = 'https://github.com/snomiao/CapsLockX/archive/master.zip'
$checksum       = '%checksum%'
$checksumType   = 'md5'

$packageArgs = @{
  packageName   = $env:ChocolateyPackageName
  unzipLocation = $toolsDir
  url           = $url
  softwareName  = 'CapsLockX' #part or all of the Display Name as you see it in Programs and Features. It should be enough to be unique
  checksum      = '%checksum%'
  checksumType  = 'sha256' #default is md5, can also be sha1, sha256 or sha512
}

Install-ChocolateyZipPackage $packageName $url $toolsDir -checksum $checksum -checksumType $checksumType

Install-ChocolateyZipPackage -PackageName "$packageName" `
                             -UnzipLocation "$toolsDir"
                             -Url "$url" `
                             -Checksum "$checksum" `
                             -ChecksumType "$checksumType" `
                            #  -Url64 "$url64" `
                            #  -Checksum64 "$checksum64" `
                            #  -ChecksumType64 "$checksumType64"

choco apikey --key 4559c944-675e-4d3a-8e83-d5ffe05c6842 --source https://push.chocolatey.org/choco
push MyPackage.1.0.nupkg --source https://push.chocolatey.org/
