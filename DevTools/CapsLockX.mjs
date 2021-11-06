import { exec } from 'child_process'
import { promisify } from 'util'
import path from 'path'
import fs from 'fs'
const clxPath = path.dirname(path.dirname(import.meta.url)) + '/CapsLockX.exe'
await promisify(exec)('cmd /c explorer ' + clxPath)
// console.log(clxPath);

// await fs.promises.readdir('Modules')
// await fs.promises.('Modules')
// modu 