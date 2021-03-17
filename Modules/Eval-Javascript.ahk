; ========== CapsLockX ==========
; 名称：Javascript 表达式计算
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.06.27
; ========== CapsLockX ==========
; 

if (!CapsLockX){
    MsgBox, % "本模块只在 CapsLockX 下工作"
    ExitApp
}

; 
; 注释：在这里，你可以使用 AppendHelp 添加帮助信息
; 
AppendHelp("
(
JavaScript 计算 (建议安装NodeJS)
| Win + Alt + C| 计算当前选区 JavaScript 表达式，并替换
)")

Return

GetObjJScript()
{
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
EscapeQuoted(code)
{
    encodedCode := code
    encodedCode := RegExReplace(encodedCode, "\\", "\\")
    encodedCode := RegExReplace(encodedCode, "'", "\'")
    encodedCode := RegExReplace(encodedCode, "\n", "\n")
    encodedCode := RegExReplace(encodedCode, "\r", "\r")
    Return "'" encodedCode "'"
}
EscapeDoubleQuotedForBatch(code)
{
    encodedCode := code
    encodedCode := RegExReplace(encodedCode, "\\", "\\")
    encodedCode := RegExReplace(encodedCode, """", "^""")
    encodedCode := RegExReplace(encodedCode, "\n", "\n")
    encodedCode := RegExReplace(encodedCode, "\r", "\r")
    Return """" encodedCode """"
}
EvalJScript(code)
{
    ; 生成代码
    realcode := "(function(){try{Return eval(" . EscapeQuoted(code) . ")}catch(e){Return e.toString()}})()"
    ; 执行代码
    JS := GetObjJScript()
    re := JS.Eval(realcode)
    Return re
}

EvalNodejs(code)
{
    ; 检查 Node.js 是否安装
    nodejsPath := "C:\Program Files\nodejs\node.exe"
    if (!FileExist(nodejsPath))
        Return ""
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
        ToolTip % inputScriptPath
        MsgBox 执行失败，未能写入脚本文件
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
        shell := comobjcreate("wscript.shell")
        exec := (shell.exec(nodejsCommand))
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
    Return out ? out : "" 
    }

    SafeEval(code)
    {
        nodejsPath := "C:\Program Files\nodejs\node.exe"
        if (FileExist(nodejsPath)){
            Return EvalNodejs(code)
        }else{
            Return EvalJScript(code)
        }
    }

    ; 使用 JS 计算并替换所选内容
    #!c::
        Clipboard =
        SendEvent ^c
        ClipWait, 1, 1
        code := Clipboard
        codeWithoutEqualEnding := RegExReplace(code, "\s+$", "")
        Clipboard := SafeEval(codeWithoutEqualEnding)
        ; 如果输入代码最后是空的就把结果添加到后面
        if (code != codeWithoutEqualEnding){
            SendEvent {Right}
        }
        SendEvent ^v
    Return
