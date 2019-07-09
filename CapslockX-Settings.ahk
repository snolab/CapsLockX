
; 基本设定

	; 以管理员身份运行
	global T_AskRunAsAdmin      := 1

	; 鼠标模块设定
		; 禁用模块
		global TMouse_Disabled         := 0
		; 提高鼠标点击、移动性能
		global TMouse_SendInput        := 1
		; 强势提升鼠标移动性能
		global TMouse_SendInputAPI     := 1
		; 自动粘附各种按钮，编辑框
		global TMouse_StickyCursor     := 1
		; 撞上屏幕边界后停止加速
		global TMouse_StopAtScreenEdge := 1
		; 屏幕 DPI 比率，自动计算得出，如果数值不对，才需要纠正
		global TMouse_DPIRatio         := A_ScreenDPI / 96
		; 鼠标加速度比率, 一般就改那个1，你想慢点就改成 0.8
		global TMouse_MouseSpeedRatio  := TMouse_DPIRatio * 0.5
		; 滚轮加速度比率, 一般就改那个1，你想慢点就改成 0.8
		global TMouse_WheelSpeedRatio  := TMouse_DPIRatio * 0.8

	; 其它模块是否禁用
		global TWinTab_Disabled := 0
		global TClip_Disabled   := 0
		global TEdit_Disabled   := 0
		global TMedia_Disabled  := 0
		global TSearch_Disabled := 0

; 进阶设定
	; 还没有

; 智能设定
	; 还没有

; 实验性功能（改了不知道会出啥事哦）

	; 修改CapslockX触发键
	global T_CapslockXKey           := "CapsLock"

	; 是否使用 ScrollLock 灯来显示 CapslockX 状态
	global T_UseScrollLockLight := 0
	global T_UseScrollLockAsCapslock := 0

	; 是否开启声音提示
	global T_SwitchSound := 0
	global T_SwitchSoundOn := "./数据/NoteG.mp3"
	global T_SwitchSoundOff := "./数据/NoteC.mp3"

