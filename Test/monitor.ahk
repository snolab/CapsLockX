
TrayTip ,, % 0 || "asdf"
; SysGet, MonitorCount, MonitorCount
; SysGet, MonitorPrimary, MonitorPrimary
; js := ""
; MsgBox, Monitor Count:`t%MonitorCount%`nPrimary Monitor:`t%MonitorPrimary%
; js := "var data = { monitors: ["
; Loop, %MonitorCount%
; {
;     SysGet, MonitorName, MonitorName, %A_Index%
;     SysGet, Monitor, Monitor, %A_Index%
;     SysGet, MonitorWorkArea, MonitorWorkArea, %A_Index%
;     js .= "{"
;     js .= ", Left: " MonitorLeft
;     js .= ", Top: " MonitorTop
;     js .= ", Right: " MonitorRight
;     js .= ", Bottom: " MonitorBottom
;     js .= ", WorkLeft: " MonitorWorkAreaLeft
;     js .= ", WorkTop: " MonitorWorkAreaTop
;     js .= ", WorkRight: " MonitorWorkAreaRight
;     js .= ", WorkBottom: " MonitorWorkAreaBottom
;     js .= "}, "
;     ; MsgBox, Monitor:`t#%A_Index%`nName:`t%MonitorName%`nLeft:`t%MonitorLeft% (%MonitorWorkAreaLeft% work)`nTop:`t%MonitorTop% (%MonitorWorkAreaTop% work)`nRight:`t%MonitorRight% (%MonitorWorkAreaRight% work)`nBottom:`t%MonitorBottom% (%MonitorWorkAreaBottom% work)
; }
; js .= "], "
; js .= "windows: ["
; WinGet, id, List,,,
; Loop, %id%
; {
;     hWnd := id%A_Index%
;     WinGet, this_pid, PID, ahk_id %hWnd%
;     WinGet, this_style, style, ahk_id %hWnd%
;     WinGet, this_minmax, minmax, ahk_id %hWnd%
;     WinGetTitle, this_title, ahk_id %hWnd%
;     WinGetClass, this_class, ahk_id %hWnd%

;     js .= "{"
;     js .= ", hWnd: " hWnd 
;     js .= ", pid: " this_pid 
;     js .= ", style: " this_style 
;     js .= ", minmax: " this_minmax 
;     ; js .= ", title: " EscapeAsJavascriptQuoted(this_title)
;     ; js .= ", class: " EscapeAsJavascriptQuoted(this_class)
;     js .= "}, "
; }
; js .= "], "
; js .= "}"
; js .= "JSON.stringify([])"

; MsgBox % EvalJScript(js)

; GetObjJScript()
; {
;    if !FileExist(ComObjFile := A_Temp "\JS.wsc")
;       FileAppend,
;          (LTrim
;             <component>
;             <public><method name='eval'/></public>
;             <script language='JScript'></script>
;             </component>
;          ), % ComObjFile
;    Return ComObjGet("script:" . ComObjFile)
; }
; EscapeAsJavascriptQuoted(code){
;     escapeCode := code
;     escapeCode := RegExReplace(escapeCode, "\\", "\\")
;     escapeCode := RegExReplace(escapeCode, "'", "\'")
;     escapeCode := RegExReplace(escapeCode, "\n", "\n")
;     escapeCode := RegExReplace(escapeCode, "\r", "\r")
;     Return "'" escapeCode "'"
; }
; EvalJScript(code)
; {
;     ; 生成代码
;     quoted_code := EscapeAsJavascriptQuoted(code)
;     TrayTip, (quoted_code)
;     ; realcode := "(function(){try{Return eval(" . quoted_code .  ")}catch(e){Return [...Object.keys(e).val]}})()"
;     realcode := "typeof ([].map)"
;     ; 执行代码
;     JS := GetObjJScript()
;     re := JS.Eval(realcode)
;     ; ToolTip % code "`n" encoded_code "`n" realcode "`n" re
;     Return re
; }
