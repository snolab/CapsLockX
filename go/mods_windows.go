package main

import "golang.design/x/hotkey"

func modsDecode(k int) []string {
	mods := []string{}
	mods = iif(k&int(hotkey.ModWin) > 0, append(mods, "command"), mods)
	mods = iif(k&int(hotkey.ModCtrl) > 0, append(mods, "control"), mods)
	mods = iif(k&int(hotkey.ModAlt) > 0, append(mods, "alt"), mods)
	mods = iif(k&int(hotkey.ModShift) > 0, append(mods, "shift"), mods)
	return mods
}

func modsreg(key hotkey.Key, onPress func(int, int), onRelease func()) func() {
	k := int(key)
	unmyreg := myreg([]hotkey.Modifier{}, key, func() { onPress(k, k|int(0)) }, onRelease)
	//
	unmyrega := myreg([]hotkey.Modifier{hotkey.ModAlt}, key, func() { onPress(k, k|int(hotkey.ModAlt)) }, onRelease)
	unmyregc := myreg([]hotkey.Modifier{hotkey.ModCtrl}, key, func() { onPress(k, k|int(hotkey.ModCtrl)) }, onRelease)
	unmyregs := myreg([]hotkey.Modifier{hotkey.ModShift}, key, func() { onPress(k, k|int(hotkey.ModShift)) }, onRelease)
	unmyregm := myreg([]hotkey.Modifier{hotkey.ModWin}, key, func() { onPress(k, k|int(hotkey.ModWin)) }, onRelease)
	//
	unmyregca := myreg([]hotkey.Modifier{hotkey.ModCtrl, hotkey.ModAlt}, key, func() { onPress(k, k|int(hotkey.ModCtrl)|int(hotkey.ModAlt)) }, onRelease)
	unmyregsa := myreg([]hotkey.Modifier{hotkey.ModShift, hotkey.ModAlt}, key, func() { onPress(k, k|int(hotkey.ModShift)|int(hotkey.ModAlt)) }, onRelease)
	unmyregma := myreg([]hotkey.Modifier{hotkey.ModWin, hotkey.ModAlt}, key, func() { onPress(k, k|int(hotkey.ModWin)|int(hotkey.ModAlt)) }, onRelease)
	unmyregac := myreg([]hotkey.Modifier{hotkey.ModAlt, hotkey.ModCtrl}, key, func() { onPress(k, k|int(hotkey.ModAlt)|int(hotkey.ModCtrl)) }, onRelease)
	unmyregsc := myreg([]hotkey.Modifier{hotkey.ModShift, hotkey.ModCtrl}, key, func() { onPress(k, k|int(hotkey.ModShift)|int(hotkey.ModCtrl)) }, onRelease)
	unmyregmc := myreg([]hotkey.Modifier{hotkey.ModWin, hotkey.ModCtrl}, key, func() { onPress(k, k|int(hotkey.ModWin)|int(hotkey.ModCtrl)) }, onRelease)
	unmyregas := myreg([]hotkey.Modifier{hotkey.ModAlt, hotkey.ModShift}, key, func() { onPress(k, k|int(hotkey.ModAlt)|int(hotkey.ModShift)) }, onRelease)
	unmyregcs := myreg([]hotkey.Modifier{hotkey.ModCtrl, hotkey.ModShift}, key, func() { onPress(k, k|int(hotkey.ModCtrl)|int(hotkey.ModShift)) }, onRelease)
	unmyregms := myreg([]hotkey.Modifier{hotkey.ModWin, hotkey.ModShift}, key, func() { onPress(k, k|int(hotkey.ModWin)|int(hotkey.ModShift)) }, onRelease)
	unmyregam := myreg([]hotkey.Modifier{hotkey.ModAlt, hotkey.ModWin}, key, func() { onPress(k, k|int(hotkey.ModAlt)|int(hotkey.ModWin)) }, onRelease)
	unmyregcm := myreg([]hotkey.Modifier{hotkey.ModCtrl, hotkey.ModWin}, key, func() { onPress(k, k|int(hotkey.ModCtrl)|int(hotkey.ModWin)) }, onRelease)
	unmyregsm := myreg([]hotkey.Modifier{hotkey.ModShift, hotkey.ModWin}, key, func() { onPress(k, k|int(hotkey.ModShift)|int(hotkey.ModWin)) }, onRelease)
	//
	return func() {
		unmyreg()
		//
		unmyrega()
		unmyregc()
		unmyregs()
		unmyregm()
		//
		unmyregca()
		unmyregsa()
		unmyregma()
		unmyregac()
		unmyregsc()
		unmyregmc()
		unmyregas()
		unmyregcs()
		unmyregms()
		unmyregam()
		unmyregcm()
		unmyregsm()
	}
}

func myreg(mod []hotkey.Modifier, key hotkey.Key, i func(), o func()) func() {
	hk := hotkey.New(mod, key)
	hk.Register()
	go func() {
		for range hk.Keydown() {
			i()
			<-hk.Keyup()
			o()
		}
	}()
	return func() {
		hk.Unregister()
	}
}
func iif(condition bool, a []string, b []string) []string {
	if condition {
		return a
	}
	return b
}
