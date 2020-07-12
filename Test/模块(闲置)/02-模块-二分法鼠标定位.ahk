Return
CoordMode, Mouse, Screen
global rctl, rctt, rctr, rctb, o_mode
o_mode := 0
rctl := 0
rctt := 0
rctr := A_ScreenWidth
rctb := A_ScreenHeight


move_to_center(){
	global rctl, rctt, rctr, rctb
	posx := (rctl + rctr) / 2
	posy := (rctt + rctb) / 2
	MouseMove, %posx%, %posy%, 0
}

start_precision_mouse_locate(){
	global rctl, rctt, rctr, rctb
	rctl := 0
	rctt := 0
	rctr := A_ScreenWidth
	rctb := A_ScreenHeight
	move_to_center()
}



#If CapsLockXMode == CM_CapsLockX || CapsLockXMode == CM_FN
	o::
		global o_mode
		o_mode := 1
		start_precision_mouse_locate()
		Return

#If (CapsLockXMode == CM_CapsLockX || CapsLockXMode == CM_FN) && o_mode
	o::
		global o_mode
		o_mode := 0
		return
	h::
		rctr := (rctl + rctr) / 2
		move_to_center()
		return
	l::
		rctl := (rctl + rctr) / 2
		move_to_center()
		return
	j::
		rctt := (rctt + rctb) / 2
		move_to_center()
		return
	k::
		rctb := (rctt + rctb) / 2
		move_to_center()
		return