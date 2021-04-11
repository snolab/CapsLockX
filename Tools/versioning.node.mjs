import { readFile, writeFile } from 'fs/promises'
const { version } = JSON.parse(await readFile('package.json', 'utf-8'))

// 更新版本号txt
const txt版本号路径 = 'Tools/version.txt'
const txt版本号 = await readFile(txt版本号路径, 'utf-8')
console.assert(txt版本号 !== version, `警告：版本号txt文件版本没有变化，当前版本为${version}`)
await writeFile(txt版本号路径, version)

// 更新choco包文件
const choco包文件路径 = 'Tools/choco/CapsLockX.nuspec'
const choco包文 = await readFile(choco包文件路径, 'utf-8')
const 新版本包文 = choco包文.replace(/(<version>)(.*?)(<\/version>)/, (_, $1, $2, $3) => $1 + version + $3)
console.assert(choco包文 !== 新版本包文, `警告：Choco包文版本没有变化，当前版本为${version}`)
await writeFile(choco包文件路径, 新版本包文)
