#Persistent
#Include WinClip\WinClip.ahk
#Include WinClip\WinClipAPI.ahk

CreateUUID()
{
  VarSetCapacity(puuid, 16, 0)
  if !(DllCall("rpcrt4.dll\UuidCreate", "ptr", &puuid))
    if !(DllCall("rpcrt4.dll\UuidToString", "ptr", &puuid, "uint*", suuid))
      return StrGet(suuid), DllCall("rpcrt4.dll\RpcStringFree", "uint*", suuid)
  return ""
}

global d := 0
global i := 0
global UUID := CreateUUID()

global mediaPATH := "z_media_" + UUID
FileCreateDir, %mediaPATH%
Return

OnClipboardChange:
	If ( A_EventInfo == 2 ) {
		i := i + 1
		filename := CAPTURE-" UUID "-" i ".png"
		WinClip.SaveBitmap(mediaPATH "/" filename, "png")

		ankiContext := "<img src='" filename "' />`n"
		
		FileAppend, %ankiContext%, anki_%UUID%.txt
		ToolTip, 图片-%filename% 已保存, 0, 0
	}
	Return


1::
	Send ^!a
	;Send #+s
	Sleep, 128
	Send {LButton Down}
	Return

2::
	Send {LButton Up}
	Sleep 64
	Send {Enter}
	Return

3::
	i := i - 1
	ToolTip, i 被设置为 %i%, 0, 0
	Return

!`::
	Send ^{Enter}
	Return

F12:: ExitApp