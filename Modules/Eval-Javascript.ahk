; ========== CapsLockX ==========
; 名称：JavaScript 表达式计算
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.06.27
; ========== CapsLockX ==========

if (!CapsLockX){
    MsgBox, % "本模块只在 CapsLockX 下工作"
    ExitApp
}

CapsLockX_AppendHelp( CapsLockX_LoadHelpFrom(CapsLockX_THIS_MODULE_HELP_FILE_PATH))

Return

EscapeQuoted(code){
    encodedCode := code
    encodedCode := RegExReplace(encodedCode, "\\", "\\")
    encodedCode := RegExReplace(encodedCode, "'", "\'")
    encodedCode := RegExReplace(encodedCode, "\n", "\n")
    encodedCode := RegExReplace(encodedCode, "\r", "\r")
    Return "'" encodedCode "'"
}
EscapeDoubleQuotedForBatch(code){
    encodedCode := code
    encodedCode := RegExReplace(encodedCode, "\\", "\\")
    encodedCode := RegExReplace(encodedCode, """", "^""")
    encodedCode := RegExReplace(encodedCode, "\n", "\n")
    encodedCode := RegExReplace(encodedCode, "\r", "\r")
    Return """" encodedCode """"
}

EvalJScript_DEPRECATED(code){
    ; 生成代码
    realcode := "(function(){try{Return eval(" . EscapeQuoted(code) . ")}catch(e){Return e.toString()}})()"
    ; 执行代码
    JS := GetObjJScript_DEPRECATED()
    re := JS.Eval(realcode)
    Return re
}
GetObjJScript_DEPRECATED(){
    if !FileExist(ComObjFile := A_Temp "\JS.wsc")
        FileAppend,
    (LTrim
    <component>
    <public><method name='eval'/></public>
    <script language='JScript'></script>
    </component>
    ), % ComObjFile
    Return ComObjGet("script:" . ComObjFile)
}
SafetyEvalJavascript(code){
    ; return EvalJavaScriptFromBrowser_TODO(code)
    static nodejsExists := !!FileExist("C:\Program Files\nodejs\node.exe")
    if(nodejsExists)
        Return EvalJavaScriptByNodeServer(code)
    else{
        Run explorer "https://nodejs.org/zh-cn/download/current/"
        MsgBox, 运行此功能需要安装 NodeJS，已为你打开下载页面。
    }
    ; Return EvalJScript_DEPRECATED(code)
}
EvalJavaScriptByBrowser_TODO(code){
    static window:=""
    if(!window){
        ; inspired with [Exo/Exo.ahk at master · Aurelain/Exo]( https://github.com/Aurelain/Exo/blob/master/Exo.ahk )
        WIDTH := 800
        HEIGHT := 600
        Gui, Add, ActiveX, w%WIDTH% h%HEIGHT% x0 y0 vwb, Shell.Explorer
        wb.Navigate("about:<!DOCTYPE html><meta http-equiv='X-UA-Compatible' content='IE=edge'>")
        while wb.readyState < 4
            Sleep 10
        wb.document.open() ; important
        document := wb.document ; shortcut
        window := document.parentWindow ; shortcut
    }
    return window.execScript(code)
}

EvalJavaScriptByNodeServer(code){
    static PassTutorial := ""
    if(!PassTutorial)
        PassTutorial := CapsLockX_Config("EvalJS", "PassTutorial", 0, "忽略使用提示")
    /* 
    sno.md5("asdf")=
    */
    ; if(!PassTutorial){
    ;     Run, notepad
    ;     MsgBox,
    ;     (
    ;     你似乎是第一次使用表达式计算功能……
    ;     让我们来试试输入：
    ;     sno.md5("asdf")
    ;     )
    ;     CapsLockX_ConfigSet("EvalJS", "PassTutorial", 1)
    ; }
    static port := ""
    if(!port)
        port := CapsLockX_Config("EvalJS", "Port", 29503, "EvalJS 服务端口")
    static nodePID := 0
    static EvalNodeJS_PIDFile := A_Temp "/EvalNodeJS.pid"
    ; pid 文件读取尝试
    if(!nodePID)
        FileRead, nodePID, %EvalNodeJS_PIDFile%, *P65001
    ; 进程存在检查
    Process Exist, %nodePID%
    if (ErrorLevel != nodePID)
        nodePID := 0
    ; 不存在则尝试启动
    if(!nodePID){
        TrayTip, EvalJS 模块, 正在启动 NodeJS...
        EnvGet, USERPROFILE, USERPROFILE
        escaped_USERPROFILE := RegExReplace(USERPROFILE, "\\", "\\")
        ; 生成服务端代码
        serverCode =
        (
        const _require = require;{
        const require = (m) => {
            try{
                _require.resolve(m)
            }catch(e){
                _require('child_process').execSync('cd "%escaped_USERPROFILE%" && npm i -S '+m)
            }
            return _require(m)
        }
        const 雪 = new Proxy({}, {get: (t, p)=>require(p)}), sno=雪;
        const _eval = async (code) => await eval(code)
        require('http').createServer(async (req,res)=>{
            let body = [];
            req.on('data', (chunk) => {
                body.push(chunk);
            }).on('end', () => {
                let code = Buffer.concat(body).toString() || decodeURI(req.url.split('?').slice(1).join('?'))
                res.writeHead(200, {"Content-Type": "text/plain; charset=utf-8"});
                    _eval(code)
                    .then(res => res?.toString !== ({}).toString && res?.toString() || JSON.stringify(res))
                    .catch(e => (console.error('错误', e), e.toString()))
                        .then(s => (res.end(s), console.log('入：', code, '\n出：', s)))
                });
            }).listen(%port%)
        }
        )
        serverScriptPath := A_Temp . "\eval-javascript-server.b1fd357f-67fe-4e2f-b9ac-e123f10b8c54.js"
        FileDelete %serverScriptPath%
        FileAppend, %serverCode%, %serverScriptPath%, UTF-8-Raw
        ; 启动 nodejs 并启用调试，注意工作目录在用户文件夹，需要调试请打开 chrome -> f12 -> node
        Run, node --inspect "%serverScriptPath%", %USERPROFILE%, Hide, nodePID
        FileDelete, %EvalNodeJS_PIDFile%
        FileAppend, %nodePID%, %EvalNodeJS_PIDFile%, UTF-8-Raw
        TrayTip, EvalJS 模块, NodeJS 服务已启动。
    }
    ; 若没有代码需要执行则将进程退出
    if(!code){
        Process Exist, %nodePID%
        if (ErrorLevel == nodePID)
            Process Close, %nodePID%
        return
    }
    ; 发送 eval 请求
    whr := ComObjCreate("WinHttp.WinHttpRequest.5.1")
    whr.Open("POST", "http://localhost:" port "/", true)
    whr.Send(code)
    whr.WaitForResponse()
    result := whr.ResponseText
    return result
}
EvalJavaScriptByNodeStdIO(code){
    ; 定义工作临时文件
    inputScriptPath := A_Temp . "\eval-javascript.b1fd357f-67fe-4e2f-b9ac-e123f10b8c54.js"
    FileDelete %inputScriptPath%
    jsonoutPath := A_Temp . "eval-javascript.b1fd357f-67fe-4e2f-b9ac-e123f10b8c54.json"
    FileDelete %jsonoutPath%

    ; 生成代码
    realcode := ""
    realcode .= "const _require = require;{" "`n"
    realcode .= "const require=(m)=>{try{_require.resolve(m)}catch(e){_require('child_process').execSync('cd %USERPROFILE% && npm i -S '+m)};return _require(m)};" "`n"
        realcode .= "const 雪 = new Proxy({}, {get: (t, p)=>require(p)}), sno=雪;" "`n"
    realcode .= "const code = " EscapeQuoted(code) ";" "`n"
    realcode .= "(async () => await eval(code))() `n"
    realcode .= " .then(res => res?.toString !== ({}).toString && res?.toString() || JSON.stringify(res)) `n"
    realcode .= " .then(s=>process.stdout.write(s)) `n"
    realcode .= " .catch(e=>process.stderr.write(e.toString())) `n"
    realcode .= "}" "`n"
    ; 写入纯 UTF8 脚本文件
    FileAppend %realcode%, %inputScriptPath%, UTF-8-RAW
    if (!FileExist(inputScriptPath)){
        TrayTip 错误, % inputScriptPath "执行失败，未能写入脚本文件"
        Return "err"
    }
    ; 执行 node 的指令
    nodejsCommand := """" nodejsPath """" " " """" inputScriptPath """"

    if (0){
        RunWait, % nodejsCommand, , Hide
        ; 读取纯 UTF8 输出
        FileRead, out, *P65001 %jsonoutPath%
        FileDelete %jsonoutPath%
    }else{
        shell := ComObjCreate("wscript.shell")
        exec := shell.exec(nodejsCommand)
        stderr := exec.stderr.readall()
        stdout := exec.stdout.readall()
        out := stdout
        if(stderr){
            TrayTip Error, % stderr
        }
        ; msgbox % out
    }
    ; `清掉垃圾文件`
    ; run "notepad " %inputScriptPath%
    FileDelete %inputScriptPath%
    re := out ? out : ""
    Return re
}

#If CapsLockXMode

; 使用 JS 计算并替换所选内容
-::
    Clipboard =
    SendEvent ^c
    ClipWait, 1, 1
    code := Clipboard
    codeWithoutEqualEnding := RegExReplace(code, "\s+$", "")
    Clipboard := SafetyEvalJavascript(codeWithoutEqualEnding)
    SendEvent ^v
return

; 使用 JS 计算并试图追加或替换所选内容
=::
    Clipboard =
    SendEvent ^c
    ClipWait, 1, 1
    code := Clipboard
    codeWithoutEqualEnding := RegExReplace(code, "=?\r?\n?(?:\*+\/)?\s*$", "")
    Clipboard := SafetyEvalJavascript(codeWithoutEqualEnding)
    ; 如果输入代码最后是空的就把结果添加到后面
    if (code != codeWithoutEqualEnding){
        SendEvent {Right}
    }
    SendEvent ^v
Return
