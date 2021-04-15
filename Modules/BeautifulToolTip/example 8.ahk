#SingleInstance Force
SetBatchLines, -1

Text=
(
btt() 的返回值。
btt() return value.
)

ret:=btt(Text,200,200,,"Style5")

Text2:=""
for k, v in ret
	Text2.=k " : " v "`n`n"

btt(Text2,200,300,2,"Style6")

Sleep, 10000

ExitApp