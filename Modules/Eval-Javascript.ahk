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
EvalJScript(code)
{
    ; 生成代码
    encoded_code := code
    encoded_code := RegExReplace(encoded_code, "\\", "\\")
    encoded_code := RegExReplace(encoded_code, "'", "\'")
    realcode := "(function(){try{return eval('" . encoded_code .  "')}catch(e){return e.toString()}})()"
    ; 执行代码
    JS := GetObjJScript()
    re := JS.Eval(realcode)
    ; ToolTip % code "`n" encoded_code "`n" realcode "`n" re
    return re
}

EvalNodejs(code)
{
    ; 检查 Node.js 是否安装
    nodejsPath := "C:\Program Files\nodejs\node.exe"
    if !FileExist(nodejsPath)
        return ""
    ; 生成代码
    encoded_code := code
    encoded_code := RegExReplace(encoded_code, "\\", "\\")
    encoded_code := RegExReplace(encoded_code, "'", "\'")
    realcode := "process.stdout.write((function(){try{return eval('" . encoded_code .  "')}catch(e){return e}})().toString())"
    ; 写入临时文件
    scriptPath := A_Temp "\eval-javascript.js"
    if FileExist(scriptPath)
        FileDelete %scriptPath%
    FileAppend %realcode%, %scriptPath%
    ; 执行 node 的指令
    command := """" nodejsPath """" " " """" scriptPath """"
    ; 使用 Node.js 运行并获取输出 // exec方法，有小黑框
    ; shell := comobjcreate("wscript.shell")
    ; exec := (shell.exec(command))
    ; stdout := exec.stdout.readall()
    
    ; 使用 Node.js 运行并获取输出 // tmp文件方法，无小黑框
    ; [How to read output of a command in Git Bash through Autohotkey - Stack Overflow]( https://stackoverflow.com/questions/53189150/how-to-read-output-of-a-command-in-git-bash-through-autohotkey )
    tmpOutputPath := A_Temp . "\" . A_ScriptName . "_eval-javascript.output.tmp"
    RunWait, % ComSpec . " /c """ . command . " > """ . tmpOutputPath . """""",, Hide
    FileRead, stdout, %tmpOutputPath%
    FileDelete, %tmpOutputPath%
    ; 清掉垃圾
    FileDelete %scriptPath%
    ; 调试
    ; ToolTip % stdout
    return stdout
}

SafeEval(code)
{
    nodejsPath := "C:\Program Files\nodejs\node.exe"
    if !FileExist(nodejsPath)
        return EvalJScript(code)
    return EvalNodejs(code)
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
