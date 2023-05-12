// https://stackoverflow.com/questions/58793857/robotgo-for-windows-10-fatal-error-zlib-h-no-such-file-or-directory
// https://sourceforge.net/projects/mingw-w64/files/
package main

import (
	"math"
	"time"

	"github.com/go-vgo/robotgo"
	"golang.design/x/hotkey"
	"golang.design/x/hotkey/mainthread"
)

func main() { mainthread.Init(mainThread) }
func mainThread() {
	unspacex := func() {}
	for {
		escaped := false
		unspacex = spacex(func() {
			unspacex()
			robotgo.KeyTap("space")
			escaped = true
		})
		for !escaped {
			time.Sleep(time.Millisecond * time.Duration(100))
		}
	}
	// return unspacex()
}
func spacex(tap func()) func() {
	unclxedit := func() {}
	unclxdesktop := func() {}
	unclxmouse := func() {}
	acted := false
	act := func() { acted = true }
	unreg := myreg([]hotkey.Modifier{}, hotkey.KeySpace,
		func() {
			acted = false
			unclxedit = clxedit(act)
			unclxdesktop = clxdesktop(act)
			unclxmouse = clxmouse(act)
		},
		func() {
			unclxedit()
			unclxdesktop()
			unclxmouse()
			if !acted {
				tap()
			}
		})
	return unreg
}

func clxmouse(act func()) func() {
	unA := turboMove(hotkey.KeyA,
		func(k int, d int) {
			robotgo.MoveRelative(-d, 0)
		},
		act)
	unD := turboMove(hotkey.KeyD,
		func(k int, d int) {
			robotgo.MoveRelative(d, 0)
		},
		act)
	unW := turboMove(hotkey.KeyW,
		func(k int, d int) {
			robotgo.MoveRelative(0, -d)
		},
		act)
	unS := turboMove(hotkey.KeyS,
		func(k int, d int) {
			robotgo.MoveRelative(0, d)
		},
		act)
	//
	unR := turboMove(hotkey.KeyR,
		func(k int, d int) { robotgo.Scroll(0, d) },
		act)
	unF := turboMove(hotkey.KeyF,
		func(k int, d int) { robotgo.Scroll(0, -d) },
		act)
	// TODO HOLD
	unE := modsreg(hotkey.KeyE,
		func(k int, taps int) { act(); robotgo.Toggle("left") },
		func() { act(); robotgo.Toggle("left", "up") })
	unQ := modsreg(hotkey.KeyQ,
		func(k int, taps int) { act(); robotgo.Toggle("right") },
		func() { act(); robotgo.Toggle("right", "up") })
	return func() {
		unA()
		unD()
		unW()
		unS()
		unE()
		unQ()
		unR()
		unF()
	}
}

func clxdesktop(act func()) func() {
	undesktop1 := turboTap(0x12, func(k int, taps int) {
		robotgo.KeyTap("left", "ctrl")
	}, act)
	undesktop2 := turboTap(0x13, func(k int, taps int) {
		robotgo.KeyTap("right", "ctrl")
	}, act)
	return func() {
		undesktop1()
		undesktop2()
	}
}
func clxedit(act func()) func() {
	unturboT := turboKey(hotkey.KeyT, "delete", act)
	unturboG := turboKey(hotkey.KeyG, "enter", act)
	//
	unturboH := turboKey(hotkey.KeyH, "left", act)
	unturboJ := turboKey(hotkey.KeyJ, "down", act)
	unturboK := turboKey(hotkey.KeyK, "up", act)
	unturboL := turboKey(hotkey.KeyL, "right", act)
	//
	unturboY := turboKey(hotkey.KeyY, "home", act)
	unturboO := turboKey(hotkey.KeyO, "end", act)
	unturboU := turboKey(hotkey.KeyU, "pagedown", act)
	unturboI := turboKey(hotkey.KeyI, "pageup", act)
	//
	// kVK_ANSI_RightBracket         = 0x1E,
	// kVK_ANSI_LeftBracket          = 0x21,
	// unturboLB := turboTap(hotkey.Key(0x21), func(k int, taps int) {
	// 	robotgo.KeyTap("tab", "shift")
	// }, act)
	// unturboRB := turboTap(hotkey.Key(0x1E), func(k int, taps int) {
	// 	robotgo.KeyTap("tab")
	// }, act)
	unturboP := turboTap(hotkey.KeyP, func(k int, taps int) {
		robotgo.KeyTap("tab", "shift")
	}, act)
	unturboN := turboTap(hotkey.KeyN, func(k int, taps int) {
		robotgo.KeyTap("tab")
	}, act)
	return func() {
		unturboT()
		unturboG()
		unturboH()
		unturboJ()
		unturboK()
		unturboL()
		unturboY()
		unturboO()
		unturboU()
		unturboI()
		unturboP()
		unturboN()
	}
}
func turboKey(i hotkey.Key, o string, act func()) func() {
	return turboTap(i, func(k int, taps int) {
		mods := modsDecode(k)
		if len(mods) == 0 {
			robotgo.KeyTap(o)
		} else {
			robotgo.KeyTap(o, mods)
		}
	}, act)
}

func turboTap(i hotkey.Key, tap func(k int, taps int), act func()) func() {
	taps := 0
	unreg := modsreg(i,
		func(kk int, k int) {
			taps = 0
			act()
			go func() {
				for taps >= 0 {
					tap(k, taps)
					ms := math.Max(0, 120*(math.Pow(0.5, 0.5*float64(taps))))
					time.Sleep(time.Millisecond * time.Duration(ms))
					taps++
				}
			}()
		},
		func() { taps = -100 })
	return func() {
		taps = -100
		unreg()
	}
}

func turboMove(i hotkey.Key, move func(k int, distance int), act func()) func() {
	t := int64(0)
	unreg := modsreg(i,
		func(kk int, k int) {
			if t != 0 {
				return
			}
			t = time.Now().UnixNano() / int64(time.Millisecond)
			act()
			go func() {
				tracking := 0
				for t != 0 {
					ct := time.Now().UnixNano() / int64(time.Millisecond)
					dt := (ct - t)
					P := 0.2
					B := 1.1
					E := 0.1
					distance := math.Max(0.0, math.Min(
						P*(math.Pow(B, 1+E*float64(dt))),
						2147483647.0))
					diff := int(distance) - tracking
					tracking += diff
					move(k, diff)
					time.Sleep(time.Millisecond * time.Duration(1))
				}
			}()
		},
		func() { t = 0 })
	return func() {
		t = 0
		unreg()
	}
}

/*


enum {
	kVK_ANSI_A                    = 0x00,
	kVK_ANSI_S                    = 0x01,
	kVK_ANSI_D                    = 0x02,
	kVK_ANSI_F                    = 0x03,
	kVK_ANSI_H                    = 0x04,
	kVK_ANSI_G                    = 0x05,
	kVK_ANSI_Z                    = 0x06,
	kVK_ANSI_X                    = 0x07,
	kVK_ANSI_C                    = 0x08,
	kVK_ANSI_V                    = 0x09,
	kVK_ANSI_B                    = 0x0B,
	kVK_ANSI_Q                    = 0x0C,
	kVK_ANSI_W                    = 0x0D,
	kVK_ANSI_E                    = 0x0E,
	kVK_ANSI_R                    = 0x0F,
	kVK_ANSI_Y                    = 0x10,
	kVK_ANSI_T                    = 0x11,
	kVK_ANSI_1                    = 0x12,
	kVK_ANSI_2                    = 0x13,
	kVK_ANSI_3                    = 0x14,
	kVK_ANSI_4                    = 0x15,
	kVK_ANSI_6                    = 0x16,
	kVK_ANSI_5                    = 0x17,
	kVK_ANSI_Equal                = 0x18,
	kVK_ANSI_9                    = 0x19,
	kVK_ANSI_7                    = 0x1A,
	kVK_ANSI_Minus                = 0x1B,
	kVK_ANSI_8                    = 0x1C,
	kVK_ANSI_0                    = 0x1D,
	kVK_ANSI_RightBracket         = 0x1E,
	kVK_ANSI_O                    = 0x1F,
	kVK_ANSI_U                    = 0x20,
	kVK_ANSI_LeftBracket          = 0x21,
	kVK_ANSI_I                    = 0x22,
	kVK_ANSI_P                    = 0x23,
	kVK_ANSI_L                    = 0x25,
	kVK_ANSI_J                    = 0x26,
	kVK_ANSI_Quote                = 0x27,
	kVK_ANSI_K                    = 0x28,
	kVK_ANSI_Semicolon            = 0x29,
	kVK_ANSI_Backslash            = 0x2A,
	kVK_ANSI_Comma                = 0x2B,
	kVK_ANSI_Slash                = 0x2C,
	kVK_ANSI_N                    = 0x2D,
	kVK_ANSI_M                    = 0x2E,
	kVK_ANSI_Period               = 0x2F,
	kVK_ANSI_Grave                = 0x32,
	kVK_ANSI_KeypadDecimal        = 0x41,
	kVK_ANSI_KeypadMultiply       = 0x43,
	kVK_ANSI_KeypadPlus           = 0x45,
	kVK_ANSI_KeypadClear          = 0x47,
	kVK_ANSI_KeypadDivide         = 0x4B,
	kVK_ANSI_KeypadEnter          = 0x4C,
	kVK_ANSI_KeypadMinus          = 0x4E,
	kVK_ANSI_KeypadEquals         = 0x51,
	kVK_ANSI_Keypad0              = 0x52,
	kVK_ANSI_Keypad1              = 0x53,
	kVK_ANSI_Keypad2              = 0x54,
	kVK_ANSI_Keypad3              = 0x55,
	kVK_ANSI_Keypad4              = 0x56,
	kVK_ANSI_Keypad5              = 0x57,
	kVK_ANSI_Keypad6              = 0x58,
	kVK_ANSI_Keypad7              = 0x59,
	kVK_ANSI_Keypad8              = 0x5B,
	kVK_ANSI_Keypad9              = 0x5C
  };
*/
