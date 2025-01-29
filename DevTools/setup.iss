; -- setup.iss --
; SEE THE DOCUMENTATION FOR DETAILS ON CREATING .ISS SCRIPT FILES!

[Setup]
AppName=CapsLockX
AppVersion=1.34.11
WizardStyle=modern
; DefaultDirName={autopf}\CapsLockX
DefaultDirName={userappdata}\CapsLockX
DefaultGroupName=CapsLockX
UninstallDisplayIcon={app}\CapsLockX.exe
Compression=lzma2
SolidCompression=yes
OutputDir=userdocs:Inno Setup CapsLockX Output

[Files]
Source: "README.md"; DestDir: "{app}"; Flags: isreadme
Source: "CapsLockX.ahk"; DestDir: "{app}" 
Source: "CapsLockX.exe"; DestDir: "{app}"
Source: "Core\*"; DestDir: "{app}"
Source: "Data\*"; DestDir: "{app}"
Source: "docs\*"; DestDir: "{app}"
Source: "Modules\*"; DestDir: "{app}"
Source: "Tools\*"; DestDir: "{app}"

[Icons]
Name: "{group}\CapsLockX"; Filename: "{app}\CapsLockX.exe"
