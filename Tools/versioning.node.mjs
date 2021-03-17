import { readFile, writeFile } from 'fs/promises'
import { exec } from 'child_process';
import { promisify } from 'util'
const { version } = JSON.parse(await readFile('package.json', 'utf-8'))

// 更新choco包文件
const choco包文件 = 'Tools/choco/Package.nuspec'
const choco包文 = await readFile(choco包文件, 'utf-8')
const 新版本包文 = choco包文.replace(/(<version>)(.*?)(<\/version>)/, (_, $1, $2, $3) => $1 + version + $3)
console.assert(choco包文 !== 新版本包文, `警告：Choco包版本没有变化，当前版本为${version}`)
await writeFile(choco包文件, 新版本包文)
await promisify(exec)('git commit -a -m "versioning"')

// await promisify(exec)('start "" tools/pack.bat')