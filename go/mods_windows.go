package main

import (
	"fmt"
	"time"

	"golang.design/x/hotkey"
)

func modsDecode(m int) []string {
	mods := []string{}
	mods = iif(m&int(hotkey.ModWin) > 0, append(mods, "cmd"), mods)
	mods = iif(m&int(hotkey.ModCtrl) > 0, append(mods, "control"), mods)
	mods = iif(m&int(hotkey.ModAlt) > 0, append(mods, "alt"), mods)
	mods = iif(m&int(hotkey.ModShift) > 0, append(mods, "shift"), mods)
	fmt.Print("mods ")
	fmt.Println(mods)
	return mods
}

func modsreg(key hotkey.Key, onPress func(k int, m int), onRelease func()) func() {
	k := int(key)
	unmyreg := myreg([]hotkey.Modifier{}, key, func() { onPress(k, int(0)) }, onRelease)
	unmyrega := myreg([]hotkey.Modifier{hotkey.ModAlt}, key, func() { onPress(k, int(hotkey.ModAlt)) }, onRelease)
	unmyregc := myreg([]hotkey.Modifier{hotkey.ModCtrl}, key, func() { onPress(k, int(hotkey.ModCtrl)) }, onRelease)
	unmyregs := myreg([]hotkey.Modifier{hotkey.ModShift}, key, func() { onPress(k, int(hotkey.ModShift)) }, onRelease)
	unmyregm := myreg([]hotkey.Modifier{hotkey.ModWin}, key, func() { onPress(k, int(hotkey.ModWin)) }, onRelease)
	unmyregac := myreg([]hotkey.Modifier{hotkey.ModAlt, hotkey.ModCtrl}, key, func() { onPress(k, int(hotkey.ModAlt)|int(hotkey.ModCtrl)) }, onRelease)
	unmyregam := myreg([]hotkey.Modifier{hotkey.ModAlt, hotkey.ModWin}, key, func() { onPress(k, int(hotkey.ModAlt)|int(hotkey.ModWin)) }, onRelease)
	unmyregas := myreg([]hotkey.Modifier{hotkey.ModAlt, hotkey.ModShift}, key, func() { onPress(k, int(hotkey.ModAlt)|int(hotkey.ModShift)) }, onRelease)
	// unmyregca := myreg([]hotkey.Modifier{hotkey.ModCtrl, hotkey.ModAlt}, key, func() { onPress(k, int(hotkey.ModCtrl)|int(hotkey.ModAlt)) }, onRelease)
	unmyregcm := myreg([]hotkey.Modifier{hotkey.ModCtrl, hotkey.ModWin}, key, func() { onPress(k, int(hotkey.ModCtrl)|int(hotkey.ModWin)) }, onRelease)
	unmyregcs := myreg([]hotkey.Modifier{hotkey.ModCtrl, hotkey.ModShift}, key, func() { onPress(k, int(hotkey.ModCtrl)|int(hotkey.ModShift)) }, onRelease)
	// unmyregma := myreg([]hotkey.Modifier{hotkey.ModWin, hotkey.ModAlt}, key, func() { onPress(k, int(hotkey.ModWin)|int(hotkey.ModAlt)) }, onRelease)
	// unmyregmc := myreg([]hotkey.Modifier{hotkey.ModWin, hotkey.ModCtrl}, key, func() { onPress(k, int(hotkey.ModWin)|int(hotkey.ModCtrl)) }, onRelease)
	unmyregms := myreg([]hotkey.Modifier{hotkey.ModWin, hotkey.ModShift}, key, func() { onPress(k, int(hotkey.ModWin)|int(hotkey.ModShift)) }, onRelease)
	// unmyregsa := myreg([]hotkey.Modifier{hotkey.ModShift, hotkey.ModAlt}, key, func() { onPress(k, int(hotkey.ModShift)|int(hotkey.ModAlt)) }, onRelease)
	// unmyregsc := myreg([]hotkey.Modifier{hotkey.ModShift, hotkey.ModCtrl}, key, func() { onPress(k, int(hotkey.ModShift)|int(hotkey.ModCtrl)) }, onRelease)
	// unmyregsm := myreg([]hotkey.Modifier{hotkey.ModShift, hotkey.ModWin}, key, func() { onPress(k, int(hotkey.ModShift)|int(hotkey.ModWin)) }, onRelease)
	//
	return func() {
		unmyreg()
		unmyrega()
		unmyregc()
		unmyregs()
		unmyregm()
		unmyregac()
		unmyregam()
		unmyregas()
		// unmyregca()
		unmyregcm()
		unmyregcs()
		// unmyregma()
		// unmyregmc()
		unmyregms()
		// unmyregsa()
		// unmyregsc()
		// unmyregsm()
	}
}

func myreg(mod []hotkey.Modifier, key hotkey.Key, i func(), o func()) func() {
	hk := hotkey.New(mod, key)
	registered := make(chan int)
	go func() {
		fmt.Println(0, time.Now().UnixMilli())
		// err :=
		hk.Register()
		// if err != nil {
		// 	return
		// }
		registered <- 1
		fmt.Println(1, time.Now().UnixMilli())
		fmt.Println(2, time.Now().UnixMilli())
		for range hk.Keydown() {
			fmt.Println(3, time.Now().UnixMilli())
			i()
			fmt.Println(4, time.Now().UnixMilli())
			<-hk.Keyup()

			fmt.Println(5, time.Now().UnixMilli())
			o()
			fmt.Println(6, time.Now().UnixMilli())
		}
	}()
	return func() {
		<-registered
		hk.Unregister()
	}
}
func iif(condition bool, a []string, b []string) []string {
	if condition {
		return a
	}
	return b
}
