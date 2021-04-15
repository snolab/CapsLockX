#SingleInstance Force
SetBatchLines, -1

Text=
(
淡入淡出。

Fadein  and  Fadeout.
)

; fadein
for k, v in [15,35,55,75,95,115,135,155,175,195,215,235,255]
{
	btt(Text,,,,"Style4",{Transparent:v})
	Sleep, 30
}

Sleep, 2000

; fadeout
for k, v in [240,220,200,180,160,140,120,100,80,60,40,20,0]
{
	btt(Text,,,,"Style4",{Transparent:v})
	Sleep, 30
}

ExitApp