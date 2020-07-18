; ========== CapsLockX ==========
; 名称：Javascript 表达式计算
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.06.27
; ========== CapsLockX ==========
; 

If(!CapsLockX){
    MsgBox, % "本模块只在 CapsLockX 下工作"
    ExitApp
}

; 
; 注释：在这里，你可以使用 AppendHelp 添加帮助信息
; 
AppendHelp("
(
Javascript 计算
| CapsLockX + Tab   `t| 计算当前选区 Javascript 表达式，并替换
)")



; CreateScriptObj() {
;     static doc := ComObjCreate("htmlfile")
;     doc.write("<meta http-equiv='X-UA-Compatible' content='IE=9'>")
;     return ObjBindMethod(doc.parentWindow, "eval")
; }

; EvalJavascript(){
;     jsObj := CreateScriptObj()
;     return %jsObj%("1+1")
; }
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
    return "'" encodedCode "'"
}
EscapeDoubleQuotedForBatch(code)
{
    encodedCode := code
    encodedCode := RegExReplace(encodedCode, "\\", "\\")
    encodedCode := RegExReplace(encodedCode, """", "^""")
    encodedCode := RegExReplace(encodedCode, "\n", "\n")
    encodedCode := RegExReplace(encodedCode, "\r", "\r")
    return """" encodedCode """"
}
EvalJScript(code)
{
    ; 生成代码
    realcode := "(function(){try{return eval(" . EscapeQuoted(code) .  ")}catch(e){return e.toString()}})()"
    ; 执行代码
    JS := GetObjJScript()
    re := JS.Eval(realcode)
    return re
}

EvalNodejs(code)
{
    ; 检查 Node.js 是否安装
    nodejsPath := "C:\Program Files\nodejs\node.exe"
    if (!FileExist(nodejsPath))
        return ""

    ; 定义工作临时文件
    inputScriptPath := A_Temp . "\eval-javascript.b1fd357f-67fe-4e2f-b9ac-e123f10b8c54.js"    
    FileDelete %inputScriptPath%
    jsonoutPath := A_Temp . "\eval-javascript.json.b1fd357f-67fe-4e2f-b9ac-e123f10b8c54.tmp"
    FileDelete %jsonoutPath%

    ; 生成代码
    encodedCode := 
    realcode := ""
    realcode .= "const code = " EscapeQuoted(code) "; `n"
    realcode .= "const jsonoutPath = " EscapeQuoted(jsonoutPath) "; `n"
    realcode .= "const ret = (()=>{try{return JSON.stringify(eval(code))}catch(err){return err}})(); `n"
    realcode .= "const fs = require('fs'); `n"
    realcode .= "fs.writeFileSync(jsonoutPath, ret)"

    ; 写入纯 UTF8 脚本文件
    FileAppend %realcode%, %inputScriptPath%, UTF-8-RAW
    if(!FileExist(inputScriptPath)){
        ToolTip % inputScriptPatherr
        MsgBox 执行失败，未能写入脚本文件
        return "err"
    }
    ; 执行 node 的指令
    nodejsCommand := """" nodejsPath """" " " """" inputScriptPath """"
    RunWait, % nodejsCommand, , Hide

    ; 读取纯 UTF8 输出
    FileRead, out, *P65001 %jsonoutPath%

    ; `清掉垃圾文件`
    FileDelete %inputScriptPath%
    FileDelete %jsonoutPath%
    return out ? out : ""   
}

SafeEval(code)
{
    nodejsPath := "C:\Program Files\nodejs\node.exe"
    if (FileExist(nodejsPath)){
        return EvalNodejs(code)
    }else{
        return EvalJScript(code)
    }
}
#If CapsLockXMode
; 使用 JS 计算并替换所选内容
Tab::
    Clipboard =
    Send ^c
    ClipWait, 1, 1
    code := Clipboard
    ; ToolTip, % code
    Clipboard := SafeEval(code)
    Send ^v
Return

; 只计算不替换，先从剪贴板取内容，如果没有则自动复制选区
^Tab::
    code := Clipboard
    if ("" == code){
        Send ^c
        ClipWait, 1
        code := Clipboard
    }
    ; ToolTip, % code
    Clipboard := SafeEval(code)
Return
