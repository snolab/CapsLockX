#!/usr/bin/env node
import aswitcher from "aswitcher";
import snorun from "snorun";
import workPackageDir from "work-package-dir";
aswitcher(process.platform, {
  aix: () => console.log("not supported"),
  android: () => console.log("not supported"),
  darwin: () => console.log("not supported"),
  freebsd: () => console.log("not supported"),
  haiku: () => console.log("not supported"),
  linux: () => console.log("not supported"),
  openbsd: () => console.log("not supported"),
  sunos: () => console.log("not supported"),
  win32: async () => {
    await workPackageDir();
    // quit after clx launched
    return snorun("CapsLockX.exe");
  },
  cygwin: () => console.log("not supported"),
  netbsd: () => console.log("not supported"),
});
