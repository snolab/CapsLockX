; #SingleInstance, Force
; SendMode Input
; SetWorkingDir, %A_ScriptDir%
; #Persistent
; #SingleInstance, force
; SetBatchLines, -1

paths := {}
paths["/"] := Func("HelloWorld")
paths["404"] := Func("NotFound")

server := new HttpServer()
server.LoadMimes(A_ScriptDir . "/mime.types")
server.SetPaths(paths)
server.Serve(8000)
return

NotFound(ByRef req, ByRef res)
{
    res.SetBodyText("Page not found")
}

HelloWorld(ByRef req, ByRef res)
{
    res.SetBodyText("Hello World")
    res.status := 200
}

#include, Modules\AHKhttp\AHKhttp.ahk
#include, Modules\AHKhttp\AHKsock.ahk