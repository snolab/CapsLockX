package main

import (
	"fmt"
	"time"

	"github.com/go-vgo/robotgo"
	"golang.design/x/hotkey"
	"golang.design/x/hotkey/mainthread"
)


func main() { mainthread.Init(mainThread) }
func mainThread() {
	space()
	// hk := hotkey.New([]hotkey.Modifier{}, hotkey.key)
	// hk.Register()
}
func space() {
	space := reg(hotkey.KeySpace)
	// space.Unregister()
	for range space.Keydown() {
		fmt.Println("space pressed")

		func() {
			clx := make(chan int)
			act := false
			go func() {
				<-clx
				act = true
			}()
			go func() {
				time.Sleep(200 * time.Millisecond)
				for !act {
					robotgo.KeyTap("space")
					time.Sleep(120 * time.Millisecond)
				}
			}()
			//
			go keyToKey(clx, hotkey.KeyT, "delete")
			go keyToKey(clx, hotkey.KeyG, "enter")
			//
			go keyToKey(clx, hotkey.KeyH, "left")
			go keyToKey(clx, hotkey.KeyJ, "down")
			go keyToKey(clx, hotkey.KeyK, "up")
			go keyToKey(clx, hotkey.KeyL, "right")
			//
			go keyToKey(clx, hotkey.KeyY, "home")
			go keyToKey(clx, hotkey.KeyO, "end")
			go keyToKey(clx, hotkey.KeyU, "pagedown")
			go keyToKey(clx, hotkey.KeyI, "pageup")
			//              hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh
			<-space.Keyup()

			if !act {
				robotgo.KeyTap("space")
			}
			close(clx)
			// fmt.Println("space unreg")
		}()
	}
}
func keyToKey(clx chan int, i hotkey.Key, o string) {
	hk := reg(i)
	fmt.Println("  ", o, " is waiting")
	go func() {
		for range hk.Keydown() {
			clx <- int(i)
			tap := make(chan int)
			stopTap := false
			go func() {
				for range tap {
					robotgo.KeyTap(o)
				}
			}()
			go func() {
				for !stopTap {
					tap <- 1
					time.Sleep(100 * time.Millisecond)
				}
			}()
			<-hk.Keyup()
			stopTap = true
			close(tap)
			fmt.Println("    ", o, " released")
		}
	}()
	for range clx {
		time.Sleep(0 * time.Millisecond)
	}
	fmt.Println("  ", o, " is not waiting")
	hk.Unregister()
}

func keyToMouse(clx chan int, i hotkey.Key, o string) {
	keydown, keyup, down := reg(i)
	fmt.Println("  ", o, " is waiting")
	go func() {
		for range hk.Keydown() {
			tap := make(chan int)
			stopTap := false
			go func() {
				for range tap {
					robotgo.KeyTap(o)
				}
			}()
			go func() {
				for !stopTap {
					tap <- 1
					time.Sleep(100 * time.Millisecond)
				}
			}()
			<-hk.Keyup()
			stopTap = true
			close(tap)
			fmt.Println("    ", o, " released")
		}
	}()
	for range clx {
		time.Sleep(0 * time.Millisecond)
	}
	fmt.Println("  ", o, " is not waiting")
	hk.Unregister()
}
func reg(key chan int, i hotkey.Key) (*hotkey.Hotkey, *hotkey.Hotkey) {
	keydown := make(chan int)
	keyup := make(chan int)
	hk := hotkey.New([]hotkey.Modifier{}, i)
	hka := hotkey.New([]hotkey.Modifier{hotkey.ModOption}, i)
	hkc := hotkey.New([]hotkey.Modifier{hotkey.ModCtrl}, i)
	hks := hotkey.New([]hotkey.Modifier{hotkey.ModShift}, i)
	hkm := hotkey.New([]hotkey.Modifier{hotkey.ModCmd}, i)
	hk.Register()
	hka.Register()
	hkc.Register()
	hks.Register()
	hkm.Register()

	select {
	case <-hk.Keydown():
		keydown <- int(i)
		<-hk.Keyup()
		keyup <- int(i)
	case <-hka.Keydown():
		keydown <- int(i)
		<-hka.Keyup()
		keyup <- int(i)
	case <-hkc.Keydown():
		keydown <- int(i)
		<-hkc.Keyup()
		keyup <- int(i)
	case <-hks.Keydown():
		keydown <- int(i)
		<-hks.Keyup()
		keyup <- int(i)
	case <-hkm.Keydown():
		keydown <- int(i)
		<-hkm.Keyup()
		keyup <- int(i)
	}
	key <- hk.Keyup()
	for range clx {
	}
	return keydown, keyup
}

// func ticker() {
// 	for {
// 		time.Sleep(1000 * time.Millisecond)
// 		fmt.Println("tick")
// 	}
// }
