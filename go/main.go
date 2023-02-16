// https://stackoverflow.com/questions/58793857/robotgo-for-windows-10-fatal-error-zlib-h-no-such-file-or-directory
// https://sourceforge.net/projects/mingw-w64/files/
package main

import (
	"math"
	"time"
	"fmt"

	"github.com/go-vgo/robotgo"
	"golang.design/x/hotkey"
	"golang.design/x/hotkey/mainthread"
)
// https://stackoverflow.com/questions/58793857/robotgo-for-windows-10-fatal-error-zlib-h-no-such-file-or-directory

func main() { mainthread.Init(mainThread) }

func mainThread() {
	fmt.Println("clx-next buggy version")
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
	fmt.Println("spacex")
	unclxedit := func() {}
	unclxmouse := func() {}
	acted := false
	act := func() { acted = true }
	unreg := myreg([]hotkey.Modifier{}, hotkey.KeySpace,
		func() {
			acted = false
			unclxedit = clxedit(act)
			unclxmouse = clxmouse(act)
		},
		func() {
			unclxedit()
			unclxmouse()
			if !acted {
				tap()
			}
		})
	return unreg
}

func clxmouse(act func()) func() {
	unA := turboTap(hotkey.KeyA,
		func(k int, taps int) { r := int(math.Log2(float64(taps))); robotgo.MoveA rgs(-r, 0) },
		act)
	unD := turboTap(hotkey.KeyD,
		func(k int, taps int) { r := int(math.Log2(float64(taps))); robotgo.MoveArgs(r, 0) },
		act)
	unW := turboTap(hotkey.KeyW,
		func(k int, taps int) { r := int(math.Log2(float64(taps))); robotgo.MoveArgs(0, -r) },
		act)
	unS := turboTap(hotkey.KeyS,
		func(k int, taps int) { r := int(math.Log2(float64(taps))); robotgo.MoveArgs(0, r) },
		act)
	// TODO HOLD
	unE := modsreg(hotkey.KeyE,
		func(k int, taps int) { act(); robotgo.Toggle("left") },
		func() { act(); robotgo.Toggle("left", "up") })
	unQ := modsreg(hotkey.KeyQ,
		func(k int, taps int) { act(); robotgo.Toggle("right") },
		func() { act(); robotgo.Toggle("right", "up") })
	//
	unR := turboTap(hotkey.KeyR,
		func(k int, taps int) { r := int((float64(taps))); robotgo.Scroll(0, r) },
		act)
	unF := turboTap(hotkey.KeyF,
		func(k int, taps int) { r := int((float64(taps))); robotgo.Scroll(0, -r) },
		act)
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
	unturboN := turboTap(hotkey.KeyN, func(k int, taps int) {
		robotgo.KeyTap("tab", "shift")
	}, act)
	unturboM := turboTap(hotkey.KeyM, func(k int, taps int) {
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
		unturboN()
		unturboM()
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

