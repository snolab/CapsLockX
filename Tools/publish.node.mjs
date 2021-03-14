import {readFile} from 'fs/promises'
const {version} = JSON.parse(await readFile('package.json', 'utf-8'))

console.log(version);