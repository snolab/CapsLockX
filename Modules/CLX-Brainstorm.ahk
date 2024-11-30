#SingleInstance, Force
; #Requires AutoHotkey v1.1.33
#Include Lib/AHK-GDIp-Library-Compilation/ahk-v1-1/Gdip_All.ahk ; https://github.com/marius-sucan/AHK-GDIp-Library-Compilation

global brainstorming := false
global brainstorm_origin := CLX_Config("BrainStorm", "Website", "https://brainstorm.snomiao.com")
global brainstormApiKey := CLX_Config("BrainStorm", "Key", "FREE", t("CLX BrainStorm 的功能激活碼，填FREE使用免費版本"))
global brainstormLastQuestion := CLX_Config("BrainStorm", "LastQuestion", "", t("Brainstorm 上次提问"))
global brainstormStagedAnswer := ""
global brainstormClipType

global brainstormFilePath := A_Temp "\capslockx-clipboard-image.jpg" ; Adjust as needed
SplitPath brainstormFilePath,, dir, ext, fnBare
old := dir "\" fnBare "-OLD" "." ext
FileRecycle % old
FileMove % brainstormFilePath, % old

OnClipboardChange("clipChanged")

return

#if CapsLockXMode

    ; Brainstorm

    b:: brainstorm_prompt()
    +b:: brainstorm_quick_capture()
    !b:: brainstorm_prompt("no-prompt")
    +!b:: brainstorm_quick_capture("no-prompt")
    ^b:: brainstorm_show()
    ; ^b:: brainstorm_set_key()

#if brainstorming

    esc:: stop_brainstorm()

#if

brainstorm_show()
{
    static BrainStormSite
    if (!BrainStormSite) {
        Gui BrainStorm:Destroy
        Gui BrainStorm:Add, ActiveX, xm w980 h640 vBrainStormSite, Shell.Explorer
        BrainStormSite.Silent := True
    }
    content := brainstorm_copy()
    BrainStormSite.Navigate(brainstorm_origin . "/?q=" . brainstorm_EncodeDecodeURI(content))
    Gui, BrainStorm:Show, , CapsLockX BrainStorm
}

stop_brainstorm()
{
    global brainstorming
    brainstorming := false
    ; traytip brainstorming stopped
}
brainstorm_set_key()
{
    msg := t("訪問官方網站来取得激活碼，在此輸入，或者填 FREE 使用免費版，網址如下：")
    InputBox, key, % t("激活碼輸入"), % msg "`n" brainstorm_origin
    if (ErrorLevel == 1) {
        Return
    }
    CLX_ConfigSet("BrainStorm", "Key", key)
}
brainstorm_copy()
{
    originalClipboard := ClipboardAll
    imagePathBeforeCopy := save_clipboard_image()

    Clipboard=
    SendEvent, ^c
    ; ClipWait, 3 ; wait for 3 seconds
    ; content := Clipboard

    ClipWait, 1, 1 ; wait for 1 seconds, wait for anytype, use with ClipboardAll
    imagePath := save_clipboard_image()
    ; prefix with image path if imagePathBeforeCopy or imagePath
    if (imagePathBeforeCopy != "" ) {
        content := "![](" . "image.jpg" . ")" . "`n" . Clipboard
    } else if (imagePath != "") {
        content := "![](" . "image.jpg" . ")" . "`n" . Clipboard
    } else {
        content := Clipboard
    }

    Clipboard := originalClipboard
    return { question: content, imagePath: imagePathBeforeCopy != "" ? imagePathBeforeCopy : imagePath != "" ? imagePath : "" }
}
brainstorm_capture_window()
{
    ;     originalClipboard := ClipboardAll

    ; Clipboard=
    ; SendEvent !{PrintScreen}
    ; ClipWait, 3 ; wait for 3 seconds
    ; content := Clipboard

    ; ClipWait, 3, 1 ; wait for 1 seconds, wait for anytype, use with ClipboardAll
    imagePath := save_active_window_image()
    if (imagePath != "") {
        content := "![](" . "image.jpg" . ")" . "`n" . ""
    } else {
        content := ""
    }

    ; Clipboard := originalClipboard
    return { question: content, imagePath: imagePath != "" ? imagePath : "" }
}
brainstorm_quick_capture(skip_prompt:=false)
{
    ; heat up
    global brainstorm_origin
    endpoint := brainstorm_origin . "/ai/chat?ret=polling"
    xhrHeatUp := ComObjCreate("Msxml2.XMLHTTP")
    xhrHeatUp.Open("PUT", endpoint)
    xhrHeatUp.onreadystatechange := Func("BS_heatUp_onReadyStateChange").Bind(xhr)
    xhrHeatUp.Send("")

    clipboardContent := brainstorm_capture_window()
    content := clipboardContent.question
    imagePath := clipboardContent.imagePath

    ; use ">>>" as default prompt if brainstormLastQuestion is empty
    global brainstormLastQuestion
    if (brainstormLastQuestion == "") {
        brainstormLastQuestion := ">>> this is screenshot, plz help user solve the problem (in the language of in screenshot)"
    }
    if(skip_prompt) {
        cmd := brainstormLastQuestion
    }else{
        prompt := ""
        prompt .= t("回邮件：选中一段邮件，输入 >> Reply this email") . "`n"
        prompt .= t("翻译：选中一段文字，输入 >> Translate to English/翻译到繁体中文/简体中文") . "`n"
        prompt .= t("总结：选中一段文字，输入 >> Summary") . "`n"
        prompt .= t("提问：选中一段文字，输入 >> Question") . "`n"
        prompt .= "--- " . t("以下为提問内容") . " ---`n" . content
        InputBox, cmd, % t("请輸入文本指令"), %prompt%, , 500, 600,,,,,% brainstormLastQuestion
    }

    ; if escape
    if (ErrorLevel == 1) {
        Return
    }
    brainstormLastQuestion := CLX_ConfigSet("BrainStorm", "LastQuestion", cmd)

    msg := Trim(content . "`n`n" . cmd, OmitChars = " `t`n")

    ToolTip, % t("Going to Ask AI")

    global brainstorming
    brainstorming := true
    brainstorm_questionPost(msg, imagePath)
    ToolTip, % t("Asking AI")
}
brainstorm_prompt(skip_prompt=false)
{
    ; heat up
    global brainstorm_origin
    endpoint := brainstorm_origin . "/ai/chat?ret=polling"
    xhrHeatUp := ComObjCreate("Msxml2.XMLHTTP")
    xhrHeatUp.Open("PUT", endpoint)
    xhrHeatUp.onreadystatechange := Func("BS_heatUp_onReadyStateChange").Bind(xhr)
    xhrHeatUp.Send("")

    clipboardContent := brainstorm_copy()
    content := clipboardContent.question
    imagePath := clipboardContent.imagePath

    prompt := ""
    prompt .= t("回邮件：选中一段邮件，输入 >> Reply this email") . "`n"
    prompt .= t("翻译：选中一段文字，输入 >> Translate to English/翻译到繁体中文/简体中文") . "`n"
    prompt .= t("总结：选中一段文字，输入 >> Summary") . "`n"
    prompt .= t("提问：选中一段文字，输入 >> Question") . "`n"
    prompt .= "--- " . t("以下为提問内容") . " ---`n" . content

    ; use ">>>" as default prompt if brainstormLastQuestion is empty
    if (brainstormLastQuestion == "") {
        brainstormLastQuestion := ">>>"
    }
    if(skip_prompt){
        cmd := brainstormLastQuestion
    }else{
        InputBox, cmd, % t("请輸入文本指令"), %prompt%, , 500, 600,,,,,% brainstormLastQuestion
    }

    ; if escape
    if (ErrorLevel == 1) {
        Return
    }

    global brainstormLastQuestion := CLX_ConfigSet("BrainStorm", "LastQuestion", cmd)

    msg := Trim(content . "`n`n" . cmd, OmitChars = " `t`n")

    ToolTip, % t("Going to Ask AI")

    global brainstorming
    brainstorming := true
    brainstorm_questionPost(msg, imagePath)
    ToolTip, % t("Asking AI")
}
BS_heatUp_onReadyStateChange(xhr){
    xhr.Close()
    ToolTip, % t("Ready to Ask AI")

    ; xhr heatup done
}

brainstorm_questionPost(question, imagePath)
{
    global brainstorm_origin
    endpoint := brainstorm_origin . "/ai/chat?ret=polling"
    xhr := ComObjCreate("Msxml2.XMLHTTP")
    xhr.Open("POST", endpoint)
    xhr.setRequestHeader("Authorization", "Bearer " . brainstormApiKey)
    global brainstorming
    if (!brainstorming) {
        xhr.Close()
        return
    }
    xhr.onreadystatechange := Func("BS_questionPost_onReadyStateChange").Bind(xhr)
    if (imagePath != "") {
        formObj := { question: question, image: [imagePath] }
    } else {
        formObj := { question: question }
    }
    contentType := ""
    CreateFormData(PostData, contentType, formObj) ; convert imagePath to formData dkj
    xhr.setRequestHeader("Content-Type", contentType)
    xhr.setRequestHeader("Origin", brainstorm_origin)
    xhr.Send(PostData)
}
BS_questionPost_onReadyStateChange(xhr)
{
    global brainstorming
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
        } else if (xhr.status == 500) {
            ; ignore 500 error
            return
        }
        ; ignore unknown error
        ; MsgBox, % xhr.status . " " xhr.responseText . " " . t("Unknown Error")
        return
    }
    global questionId := xhr.responseText
    if (!questionId) {
        ; ignore error
        ; MsgBox, % t("Fail to ask ai")
        return
    }

    ToolTip, % t("Waiting Answer...")
    ; tooltip askAiSucc with question %questionId%

    global brainstormStagedAnswer
    brainstormStagedAnswer := ""
    tokenAppend(questionId)
}
tokenAppend(questionId)
{
    global brainstorming
    if(!brainstorming)
        return
    global brainstorm_origin
    endpoint := brainstorm_origin "/ai/" questionId
    xhra := ComObjCreate("Msxml2.XMLHTTP")
    xhra.open("GET", endpoint)
    xhra.onreadystatechange := Func("tokenAppend_onReadyStateChange").Bind(xhra)
    xhra.Send()
}
tokenAppend_onReadyStateChange(xhra)
{
    global brainstorming
    if (!brainstorming)
        return
    global questionId
    ; global brainstorm_response
    if (xhra.readyState != 4) {
        return
    }
    ; drain
    if (xhra.status != 200) {
        ; ToolTip
        ; TrayTip AI Response copied, %brainstorm_response%
        ; Clipboard:=brainstorm_response

        return
    }

    token := xhra.responseText
    if (!token){
        ; HEART BEAT，continue
        tokenAppend(questionId)

        return
    }
    ; brainstorm_response .= token
    ; ToolTip response %brainstorm_response%%brainstorm_response%
    global brainstormStagedAnswer
    brainstormStagedAnswer .= token

    ToolTip, % brainstormStagedAnswer . "`n`n" . t("Copied, Press [CLX+Space] to hide.")

    Clipboard := brainstormStagedAnswer

    ; SetKeyDelay, 0, 0
    ; SendEvent {text}%token%
    tokenAppend(questionId)
}

brainstorm_EncodeDecodeURI(str, encode := true, component := true)
{
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
  ; Create a random Boundary
		Local Boundary := this.RandomBoundary()
		Local BoundaryLine := "------------------------------" . Boundary

    this.Len := 0 ; GMEM_ZEROINIT|GMEM_FIXED = 0x40
    this.Ptr := DllCall( "GlobalAlloc", "UInt",0x40, "UInt",1, "Ptr"  )          ; allocate global memory

  ; Loop input paramters
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

    ; Create a bytearray and copy data in to it.
    retData := ComObjArray( 0x11, this.Len ) ; Create SAFEARRAY = VT_ARRAY|VT_UI1
    pvData  := NumGet( ComObjValue( retData ) + 8 + A_PtrSize )
    DllCall( "RtlMoveMemory", "Ptr",pvData, "Ptr",this.Ptr, "Ptr",this.Len )

    this.Ptr := DllCall( "GlobalFree", "Ptr",this.Ptr, "Ptr" )                   ; free global memory

    retHeader := "multipart/form-data; boundary=----------------------------" . Boundary
	}

  StrPutUTF8( str ) {
    Local ReqSz := StrPut( str, "utf-8" ) - 1
    this.Len += ReqSz                                  ; GMEM_ZEROINIT|GMEM_MOVEABLE = 0x42
    this.Ptr := DllCall( "GlobalReAlloc", "Ptr",this.Ptr, "UInt",this.len + 1, "UInt", 0x42 )
    StrPut( str, this.Ptr + this.len - ReqSz, ReqSz, "utf-8" )
  }

  LoadFromFile( Filename ) {
    Local objFile := FileOpen( FileName, "r" )
    this.Len += objFile.Length                     ; GMEM_ZEROINIT|GMEM_MOVEABLE = 0x42
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

; Save clipboard image to file
save_clipboard_image() {
    global brainstormFilePath
    If (brainstormClipType = "" || brainstormClipType = NONTEXT := 2) {
        FileRecycle % brainstormFilePath
        clipboardToImageFile(brainstormFilePath)
        If FileExist(brainstormFilePath){
            ; preview image
            ; Run % brainstormFilePath
            return brainstormFilePath
        }
        ; silent fail
    }
    ; silent fail
    Return ""
}

save_active_window_image() {
    global brainstormFilePath
    FileRecycle % brainstormFilePath
    activeWindowToImageFile(brainstormFilePath)
    If FileExist(brainstormFilePath){
        ; preview image
        ; Run % brainstormFilePath
        return brainstormFilePath
    }
    ; silent fail
    Return ""
}

clipboardToImageFile(filePath) {
    pToken := Gdip_Startup()
    pBitmap := Gdip_CreateBitmapFromClipboard() ; Clipboard -> bitmap
    Gdip_SaveBitmapToFile(pBitmap, filePath) ; Bitmap    -> file
    Gdip_DisposeImage(pBitmap), Gdip_Shutdown(pToken)
}
activeWindowToImageFile(filePath) {
    pToken := Gdip_Startup()
    WinGetPos, x, y, w, h, A
    pBitmap := Gdip_BitmapFromScreen(x "|" y "|" w "|" h) ; Screen -> bitmap
    Gdip_SaveBitmapToFile(pBitmap, filePath) ; Bitmap    -> file
    Gdip_DisposeImage(pBitmap), Gdip_Shutdown(pToken)
}

clipChanged(type) {
    brainstormClipType := type
}
