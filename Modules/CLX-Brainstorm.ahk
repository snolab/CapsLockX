; #Requires AutoHotkey v1.1.33
#Include ./Modules/Lib/AHK-GDIp-Library-Compilation/ahk-v1-1/Gdip_All.ahk ; https://github.com/marius-sucan/AHK-GDIp-Library-Compilation

; Note: This module uses Msxml2.XMLHTTP for HTTP requests
; which automatically respects Windows Internet Explorer proxy settings.
; To configure proxy: Control Panel → Internet Options → Connections → LAN settings

global brainstorming := false
global brainstormed := false
global brainstorm_voice_recording := false
global brainstorm_origin := CLX_Config("BrainStorm", "Website", "https://brainstorm.snomiao.com")
global brainstormApiKey := CLX_Config("BrainStorm", "Key", "FREE", t("CLX BrainStorm 的功能激活碼，填FREE使用免費版本"))
global brainstormLastQuestion := CLX_Config("BrainStorm", "LastQuestion", "", t("Brainstorm 上次提问"))
global brainstormStagedAnswer := ""
global brainstormClipType
global brainstormAudioPath := A_Temp "\capslockx-voice-recording.wav"
global brainstormFilePath := A_Temp "\capslockx-clipboard-image.jpg"

SplitPath brainstormFilePath,, dir, ext, fnBare
old := dir "\" fnBare "-OLD" "." ext
FileRecycle % old
FileMove % brainstormFilePath, % old

OnClipboardChange("clipChanged")

return

; ── Hotkeys ──────────────────────────────────────────────────────────

#if CapsLockXMode

    b:: brainstorm_ask("copy")
    +b:: brainstorm_ask("window")
    !b:: brainstorm_ask("copy", true)
    +!b:: brainstorm_ask("window", true)
    ^b:: brainstorm_show()

    m:: brainstorm_ask("window", true)

#if CapsLockXMode && !brainstorm_voice_recording

    v:: brainstorm_voice()

#if brainstorm_voice_recording

    v:: brainstorm_voice_send()
    esc:: brainstorm_voice_stop()
    `:: brainstorm_voice_stop()

#if brainstorming

    esc:: stop_brainstorm()
    `:: stop_brainstorm()

#if !brainstorming && brainstormed

    esc:: stop_brainstormed()
    `:: stop_brainstormed()

#if

; ── Tooltip / Text Wrapping ──────────────────────────────────────────

brainstorm_Tooltip(Text) {
    global brainstormed
    ToolTip, % WrapTextWithWidth(Text, 80), , , 20
    brainstormed := true
}

GetDisplayWidth(str) {
    width := 0
    Loop, Parse, str
    {
        code := Ord(A_LoopField)
        if ((code >= 0x4E00 && code <= 0x9FFF)
            || (code >= 0x3040 && code <= 0x30FF)
            || (code >= 0xAC00 && code <= 0xD7AF)
            || (code >= 0xFF00 && code <= 0xFFEF))
            width += 2
        else
            width += 1
    }
    return width
}

WrapTextWithWidth(text, maxWidth := 80) {
    result := ""
    currentLine := ""
    for index, word in StrSplit(text, " ") {
        testLine := currentLine ? currentLine . " " . word : word
        if (GetDisplayWidth(testLine) <= maxWidth) {
            currentLine := testLine
        } else {
            if (currentLine != "")
                result .= currentLine . "`n"
            if (GetDisplayWidth(word) > maxWidth) {
                chars := ""
                Loop, Parse, word
                {
                    testChars := chars . A_LoopField
                    if (GetDisplayWidth(testChars) > maxWidth) {
                        result .= chars . "`n"
                        chars := A_LoopField
                    } else {
                        chars := testChars
                    }
                }
                currentLine := chars
            } else {
                currentLine := word
            }
        }
    }
    if (currentLine != "")
        result .= currentLine
    return result
}

; ── State Control ────────────────────────────────────────────────────

stop_brainstorm() {
    global brainstorming
    brainstorming := false
}

stop_brainstormed() {
    global brainstormed
    brainstorm_Tooltip("")
    brainstormed := false
}

; ── Content Capture ──────────────────────────────────────────────────

brainstorm_capture(mode) {
    if (mode == "window")
        return brainstorm_capture_window()
    if (mode == "both")
        return brainstorm_capture_both()
    return brainstorm_capture_copy()
}

brainstorm_capture_copy() {
    originalClipboard := ClipboardAll
    imagePathBefore := save_clipboard_image()

    Clipboard=
    SendEvent, ^c
    ClipWait, 1, 1

    imagePath := imagePathBefore != "" ? imagePathBefore : save_clipboard_image()
    content := imagePath != "" ? "![](" . "image.jpg" . ")`n" . Clipboard : Clipboard

    Clipboard := originalClipboard
    return { question: content, imagePath: imagePath }
}

brainstorm_capture_window() {
    imagePath := save_active_window_image()
    content := imagePath != "" ? "![](" . "image.jpg" . ")`n" : ""
    return { question: content, imagePath: imagePath }
}

brainstorm_capture_both() {
    ; grab selected text via clipboard
    originalClipboard := ClipboardAll
    Clipboard=
    SendEvent, ^c
    ClipWait, 1, 1
    selectedText := Clipboard
    Clipboard := originalClipboard

    ; grab window screenshot
    imagePath := save_active_window_image()
    content := (imagePath != "" ? "![](" . "image.jpg" . ")`n" : "") . selectedText
    return { question: content, imagePath: imagePath }
}

; ── Main Entry Points ────────────────────────────────────────────────

brainstorm_ask(captureMode := "copy", skip_prompt := false, defaultPrompt := "") {
    global brainstorm_origin, brainstormLastQuestion, brainstorming, brainstormed

    ; heat up
    brainstorm_heatup()

    captured := brainstorm_capture(captureMode)
    content := captured.question
    imagePath := captured.imagePath

    ; default prompt depends on capture mode
    if (brainstormLastQuestion == "")
        brainstormLastQuestion := captureMode == "window"
            ? ">>> this is screenshot, plz help user solve the problem (in the language of in screenshot)"
            : ">>>"

    if (skip_prompt) {
        cmd := defaultPrompt != "" ? defaultPrompt : brainstormLastQuestion
    } else {
        prompt := ""
        prompt .= t("回邮件：选中一段邮件，输入 >> Reply this email") . "`n"
        prompt .= t("翻译：选中一段文字，输入 >> Translate to English/翻译到繁体中文/简体中文") . "`n"
        prompt .= t("总结：选中一段文字，输入 >> Summary") . "`n"
        prompt .= t("提问：选中一段文字，输入 >> Question") . "`n"
        prompt .= "--- " . t("以下为提問内容") . " ---`n" . content
        InputBox, cmd, % t("请輸入文本指令"), %prompt%, , 500, 600,,,,,% brainstormLastQuestion

        if (ErrorLevel == 1)
            return
        brainstormLastQuestion := CLX_ConfigSet("BrainStorm", "LastQuestion", cmd)
    }

    msg := Trim(content . "`n`n" . cmd, OmitChars = " `t`n")

    brainstorm_Tooltip(t("Going to Ask AI"))
    brainstorming := true
    brainstormed := true
    brainstorm_questionPost(msg, imagePath)
    brainstorm_Tooltip(t("Asking AI"))
}

brainstorm_show() {
    static BrainStormSite
    if (!BrainStormSite) {
        Gui BrainStorm:Destroy
        Gui BrainStorm:Add, ActiveX, xm w980 h640 vBrainStormSite, Shell.Explorer
        BrainStormSite.Silent := True
    }
    content := brainstorm_capture("copy")
    BrainStormSite.Navigate(brainstorm_origin . "/?q=" . brainstorm_EncodeDecodeURI(content))
    Gui, BrainStorm:Show, , CapsLockX BrainStorm
}

brainstorm_set_key() {
    msg := t("訪問官方網站来取得激活碼，在此輸入，或者填 FREE 使用免費版，網址如下：")
    InputBox, key, % t("激活碼輸入"), % msg "`n" brainstorm_origin
    if (ErrorLevel == 1)
        return
    CLX_ConfigSet("BrainStorm", "Key", key)
}

; ── API Communication ────────────────────────────────────────────────

brainstorm_heatup() {
    global brainstorm_origin
    xhr := ComObjCreate("Msxml2.XMLHTTP")
    xhr.Open("PUT", brainstorm_origin . "/ai/chat?ret=polling")
    xhr.onreadystatechange := Func("BS_heatUp_onReadyStateChange").Bind(xhr)
    xhr.Send("")
}

BS_heatUp_onReadyStateChange(xhr) {
    xhr.Close()
    brainstorm_Tooltip(t("Ready to Ask AI"))
}

brainstorm_questionPost(question, imagePath, audioPath := "") {
    global brainstorm_origin, brainstormApiKey, brainstorming
    if (!brainstorming)
        return

    xhr := ComObjCreate("Msxml2.XMLHTTP")
    xhr.Open("POST", brainstorm_origin . "/ai/chat?ret=polling")
    xhr.setRequestHeader("Authorization", "Bearer " . brainstormApiKey)
    xhr.onreadystatechange := Func("BS_questionPost_onReadyStateChange").Bind(xhr)

    formObj := { question: question }
    if (imagePath != "")
        formObj.image := [imagePath]
    if (audioPath != "")
        formObj.audio := [audioPath]

    contentType := ""
    CreateFormData(PostData, contentType, formObj)
    xhr.setRequestHeader("Content-Type", contentType)
    xhr.setRequestHeader("Origin", brainstorm_origin)
    xhr.Send(PostData)
}

BS_questionPost_onReadyStateChange(xhr) {
    global brainstorming, brainstormed, brainstormStagedAnswer, questionId
    if (!brainstorming)
        return
    if (xhr.readyState != 4)
        return
    if (xhr.status != 200) {
        if (xhr.status == 403) {
            MsgBox, % xhr.responseText . " " . t("请检查激活码是否正确")
            brainstorm_set_key()
        } else if (xhr.status == 429) {
            MsgBox, % xhr.responseText . " " . t("请等待一段时间后再试")
        }
        return
    }
    questionId := xhr.responseText
    if (!questionId) {
        brainstorm_Tooltip(t("Fail to ask ai"))
        return
    }
    brainstorm_Tooltip(t("Waiting Answer..."))
    brainstormed := true
    brainstormStagedAnswer := ""
    tokenAppend(questionId)
}

tokenAppend(questionId) {
    global brainstorming, brainstorm_origin
    if (!brainstorming)
        return
    xhra := ComObjCreate("Msxml2.XMLHTTP")
    xhra.open("GET", brainstorm_origin "/ai/" questionId)
    xhra.onreadystatechange := Func("tokenAppend_onReadyStateChange").Bind(xhra)
    xhra.Send()
}

tokenAppend_onReadyStateChange(xhra) {
    global brainstormStagedAnswer, brainstorming, questionId
    if (!brainstorming) {
        Clipboard := brainstormStagedAnswer
        brainstorm_Tooltip(brainstormStagedAnswer . "`n`n" . t("Chat streaming stopped, Copied to Clipboard, Press [ESC] to hide."))
        return
    }
    if (xhra.readyState != 4)
        return
    if (xhra.status != 200) {
        brainstorming := false
        Clipboard := brainstormStagedAnswer
        brainstorm_Tooltip(brainstormStagedAnswer . "`n`n" . t("Copied to Clipboard, Press [ESC] to hide."))
        return
    }
    token := xhra.responseText
    if (!token) {
        tokenAppend(questionId)
        return
    }
    brainstormStagedAnswer .= token
    brainstorm_Tooltip(brainstormStagedAnswer . "`n`n" . t("Press [ESC] to stop."))
    Clipboard := brainstormStagedAnswer
    tokenAppend(questionId)
}

; ── Voice Recording ──────────────────────────────────────────────────

mciSendString(cmd) {
    return DllCall("winmm\mciSendStringW", "WStr", cmd, "Ptr", 0, "UInt", 0, "Ptr", 0)
}

brainstorm_get_default_mic_name() {
    ; WAVEINCAPSW: wMid(2) + wPid(2) + vDriverVersion(4) + szPname(32*2=64) + dwFormats(4) + wChannels(2) + wReserved1(2) = 80
    VarSetCapacity(caps, 80, 0)
    result := DllCall("winmm\waveInGetDevCapsW", "UInt", 0xFFFFFFFF, "Ptr", &caps, "UInt", 80)
    if (result != 0)
        return t("Unknown")
    return StrGet(&caps + 8, 32, "UTF-16")
}

brainstorm_voice() {
    global brainstorm_voice_recording, brainstormAudioPath
    if (brainstorm_voice_recording)
        return

    FileRecycle % brainstormAudioPath
    FileDelete % brainstormAudioPath

    mciSendString("close clxmic")
    mciSendString("open new type waveaudio alias clxmic")
    mciSendString("record clxmic")

    brainstorm_voice_recording := true
    micName := brainstorm_get_default_mic_name()
    brainstorm_Tooltip(t("Recording voice... Press [V] to send, [ESC] to cancel.") . "`n" . t("Mic: ") . micName)
    SetTimer, brainstorm_voice_auto_send, -60000
}

brainstorm_voice_auto_send:
    if (brainstorm_voice_recording)
        brainstorm_voice_send()
return

brainstorm_voice_stop() {
    global brainstorm_voice_recording, brainstormAudioPath
    if (!brainstorm_voice_recording)
        return

    SetTimer, brainstorm_voice_auto_send, Off
    mciSendString("stop clxmic")
    mciSendString("close clxmic")
    brainstorm_voice_recording := false

    FileDelete % brainstormAudioPath
    brainstorm_Tooltip(t("Voice recording cancelled."))
    SetTimer, brainstorm_clear_cancel_tooltip, -2000
}

brainstorm_clear_cancel_tooltip:
    brainstorm_Tooltip("")
    brainstormed := false
return

brainstorm_voice_send() {
    global brainstorm_voice_recording, brainstormAudioPath
    global brainstorm_origin, brainstormLastQuestion, brainstorming, brainstormed
    if (!brainstorm_voice_recording)
        return

    SetTimer, brainstorm_voice_auto_send, Off
    mciSendString("stop clxmic")
    mciSendString("save clxmic " . brainstormAudioPath)
    mciSendString("close clxmic")
    brainstorm_voice_recording := false

    if (!FileExist(brainstormAudioPath)) {
        brainstorm_Tooltip(t("Voice recording failed."))
        return
    }

    brainstorm_Tooltip(t("Sending voice..."))
    brainstorm_heatup()

    captured := brainstorm_capture_window()
    if (brainstormLastQuestion == "")
        brainstormLastQuestion := ">>> this is screenshot with voice message, plz help user (in the language of in screenshot)"

    msg := Trim(captured.question . "`n`n" . brainstormLastQuestion, OmitChars = " `t`n")

    brainstorming := true
    brainstormed := true
    brainstorm_questionPost(msg, captured.imagePath, brainstormAudioPath)
    brainstorm_Tooltip(t("Asking AI"))
}

; ── Utilities ────────────────────────────────────────────────────────

brainstorm_EncodeDecodeURI(str, encode := true, component := true) {
    ; https://www.autohotkey.com/boards/viewtopic.php?t=84825
    static Doc, JS
    if !Doc {
        Doc := ComObjCreate("htmlfile")
        Doc.write("<meta http-equiv=""X-UA-Compatible"" content=""IE=9"">")
        JS := Doc.parentWindow
        ( Doc.documentMode < 9 && JS.execScript() )
    }
    Return JS[ (encode ? "en" : "de") . "codeURI" . (component ? "Component" : "") ](str)
}

; https://www.autohotkey.com/boards/viewtopic.php?t=67426 dl
CreateFormData(ByRef retData, ByRef retHeader, objParam) {
	New CreateFormData(retData, retHeader, objParam)
}

Class CreateFormData {

	__New(ByRef retData, ByRef retHeader, objParam) {
		Local CRLF := "`r`n", i, k, v, str, pvData
		Local Boundary := this.RandomBoundary()
		Local BoundaryLine := "------------------------------" . Boundary

    this.Len := 0
    this.Ptr := DllCall( "GlobalAlloc", "UInt",0x40, "UInt",1, "Ptr"  )

		For k, v in objParam
		{
			If IsObject(v) {
				For i, FileName in v
				{
					str := BoundaryLine . CRLF
					     . "Content-Disposition: form-data; name=""" . k . """; filename=""" . FileName . """" . CRLF
					     . "Content-Type: " . this.MimeType(FileName) . CRLF . CRLF
          this.StrPutUTF8( str )
          this.LoadFromFile( Filename )
          this.StrPutUTF8( CRLF )
				}
			} Else {
				str := BoundaryLine . CRLF
				     . "Content-Disposition: form-data; name=""" . k """" . CRLF . CRLF
				     . v . CRLF
        this.StrPutUTF8( str )
			}
		}

		this.StrPutUTF8( BoundaryLine . "--" . CRLF )

    retData := ComObjArray( 0x11, this.Len )
    pvData  := NumGet( ComObjValue( retData ) + 8 + A_PtrSize )
    DllCall( "RtlMoveMemory", "Ptr",pvData, "Ptr",this.Ptr, "Ptr",this.Len )
    this.Ptr := DllCall( "GlobalFree", "Ptr",this.Ptr, "Ptr" )

    retHeader := "multipart/form-data; boundary=----------------------------" . Boundary
	}

  StrPutUTF8( str ) {
    Local ReqSz := StrPut( str, "utf-8" ) - 1
    this.Len += ReqSz
    this.Ptr := DllCall( "GlobalReAlloc", "Ptr",this.Ptr, "UInt",this.len + 1, "UInt", 0x42 )
    StrPut( str, this.Ptr + this.len - ReqSz, ReqSz, "utf-8" )
  }

  LoadFromFile( Filename ) {
    Local objFile := FileOpen( FileName, "r" )
    this.Len += objFile.Length
    this.Ptr := DllCall( "GlobalReAlloc", "Ptr",this.Ptr, "UInt",this.len, "UInt", 0x42 )
    objFile.RawRead( this.Ptr + this.Len - objFile.length, objFile.length )
    objFile.Close()
  }

	RandomBoundary() {
		str := "0|1|2|3|4|5|6|7|8|9|a|b|c|d|e|f|g|h|i|j|k|l|m|n|o|p|q|r|s|t|u|v|w|x|y|z"
		Sort, str, D| Random
		str := StrReplace(str, "|")
		Return SubStr(str, 1, 12)
	}

	MimeType(FileName) {
		SplitPath, FileName,,, ext
		if (ext = "wav")
			return "audio/wav"
		if (ext = "mp3")
			return "audio/mpeg"
		n := FileOpen(FileName, "r").ReadUInt()
		Return (n        = 0x474E5089) ? "image/png"
		     : (n        = 0x38464947) ? "image/gif"
		     : (n&0xFFFF = 0x4D42    ) ? "image/bmp"
		     : (n&0xFFFF = 0xD8FF    ) ? "image/jpeg"
		     : (n&0xFFFF = 0x4949    ) ? "image/tiff"
		     : (n&0xFFFF = 0x4D4D    ) ? "image/tiff"
		     : "application/octet-stream"
	}
}

; ── Image Capture ────────────────────────────────────────────────────

save_clipboard_image() {
    global brainstormFilePath
    If (brainstormClipType = "" || brainstormClipType = NONTEXT := 2) {
        FileRecycle % brainstormFilePath
        clipboardToImageFile(brainstormFilePath)
        If FileExist(brainstormFilePath)
            return brainstormFilePath
    }
    Return ""
}

save_active_window_image() {
    global brainstormFilePath
    FileRecycle % brainstormFilePath
    activeWindowToImageFile(brainstormFilePath)
    If FileExist(brainstormFilePath)
        return brainstormFilePath
    Return ""
}

clipboardToImageFile(filePath) {
    pToken := Gdip_Startup()
    pBitmap := Gdip_CreateBitmapFromClipboard()
    Gdip_SaveBitmapToFile(pBitmap, filePath)
    Gdip_DisposeImage(pBitmap), Gdip_Shutdown(pToken)
}

activeWindowToImageFile(filePath) {
    pToken := Gdip_Startup()
    WinGetPos, x, y, w, h, A
    pBitmap := Gdip_BitmapFromScreen(x "|" y "|" w "|" h)
    Gdip_SaveBitmapToFile(pBitmap, filePath)
    Gdip_DisposeImage(pBitmap), Gdip_Shutdown(pToken)
}

clipChanged(type) {
    brainstormClipType := type
}
