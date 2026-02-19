; ========== CapsLockX ==========
; 名称：连击 / Lianki
; 描述：Media key shortcuts and repeat play/pause N times
;   CapsLockX + [  →  Previous track
;   CapsLockX + ]  →  Next track
;   CapsLockX + \  →  Play/Pause
; ========== CapsLockX ==========

if (!CapsLockX) {
    MsgBox, % "本模块只在 CapsLockX 下工作 / This module is only for CapsLockX"
    ExitApp
}
CLX_AppendHelp( CLX_LoadHelpFrom(CLX_THIS_MODULE_HELP_FILE_PATH))
Return

#if CapsLockXMode

[:: Send {Media_Prev}   ; previous track
]:: Send {Media_Next}   ; next track
\:: Send {Media_Play_Pause}   ; next track
