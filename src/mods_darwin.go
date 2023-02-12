package main

import "golang.design/x/hotkey"

func modsDecode(k int) []string {
	mods := []string{}
	mods = iif(k&int(hotkey.ModCmd) > 0, append(mods, "command"), mods)
	mods = iif(k&int(hotkey.ModCtrl) > 0, append(mods, "control"), mods)
	mods = iif(k&int(hotkey.ModOption) > 0, append(mods, "alt"), mods)
	mods = iif(k&int(hotkey.ModShift) > 0, append(mods, "shift"), mods)
	return mods
}

func modsreg(key hotkey.Key, i func(int, int), o func()) func() {
	unmyreg := myreg([]hotkey.Modifier{}, key, func() { i(int(key), int(key)|int(0)) }, o)
	//
	unmyrega := myreg([]hotkey.Modifier{hotkey.ModOption}, key, func() { i(int(key), int(key)|int(hotkey.ModOption)) }, o)
	unmyregc := myreg([]hotkey.Modifier{hotkey.ModCtrl}, key, func() { i(int(key), int(key)|int(hotkey.ModCtrl)) }, o)
	unmyregs := myreg([]hotkey.Modifier{hotkey.ModShift}, key, func() { i(int(key), int(key)|int(hotkey.ModShift)) }, o)
	unmyregm := myreg([]hotkey.Modifier{hotkey.ModCmd}, key, func() { i(int(key), int(key)|int(hotkey.ModCmd)) }, o)
	//
	unmyregca := myreg([]hotkey.Modifier{hotkey.ModCtrl, hotkey.ModOption}, key, func() { i(int(key), int(key)|int(hotkey.ModCtrl)|int(hotkey.ModOption)) }, o)
	unmyregsa := myreg([]hotkey.Modifier{hotkey.ModShift, hotkey.ModOption}, key, func() { i(int(key), int(key)|int(hotkey.ModShift)|int(hotkey.ModOption)) }, o)
	unmyregma := myreg([]hotkey.Modifier{hotkey.ModCmd, hotkey.ModOption}, key, func() { i(int(key), int(key)|int(hotkey.ModCmd)|int(hotkey.ModOption)) }, o)
	//
	unmyregac := myreg([]hotkey.Modifier{hotkey.ModOption, hotkey.ModCtrl}, key, func() { i(int(key), int(key)|int(hotkey.ModOption)|int(hotkey.ModCtrl)) }, o)
	unmyregsc := myreg([]hotkey.Modifier{hotkey.ModShift, hotkey.ModCtrl}, key, func() { i(int(key), int(key)|int(hotkey.ModShift)|int(hotkey.ModCtrl)) }, o)
	unmyregmc := myreg([]hotkey.Modifier{hotkey.ModCmd, hotkey.ModCtrl}, key, func() { i(int(key), int(key)|int(hotkey.ModCmd)|int(hotkey.ModCtrl)) }, o)
	//
	unmyregas := myreg([]hotkey.Modifier{hotkey.ModOption, hotkey.ModShift}, key, func() { i(int(key), int(key)|int(hotkey.ModOption)|int(hotkey.ModShift)) }, o)
	unmyregcs := myreg([]hotkey.Modifier{hotkey.ModCtrl, hotkey.ModShift}, key, func() { i(int(key), int(key)|int(hotkey.ModCtrl)|int(hotkey.ModShift)) }, o)
	unmyregms := myreg([]hotkey.Modifier{hotkey.ModCmd, hotkey.ModShift}, key, func() { i(int(key), int(key)|int(hotkey.ModCmd)|int(hotkey.ModShift)) }, o)
	//
	unmyregam := myreg([]hotkey.Modifier{hotkey.ModOption, hotkey.ModCmd}, key, func() { i(int(key), int(key)|int(hotkey.ModOption)|int(hotkey.ModCmd)) }, o)
	unmyregcm := myreg([]hotkey.Modifier{hotkey.ModCtrl, hotkey.ModCmd}, key, func() { i(int(key), int(key)|int(hotkey.ModCtrl)|int(hotkey.ModCmd)) }, o)
	unmyregsm := myreg([]hotkey.Modifier{hotkey.ModShift, hotkey.ModCmd}, key, func() { i(int(key), int(key)|int(hotkey.ModShift)|int(hotkey.ModCmd)) }, o)
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
