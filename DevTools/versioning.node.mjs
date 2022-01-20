import { readFile, writeFile } from "fs/promises";
const 文本读 = (path) => readFile(path, "utf-8") || "";
const { version } = JSON.parse(await 文本读("package.json"));
// console.log('get version', version)
// 更新版本号txt
const txt版本号路径 = "./Core/version.txt";
const txt版本号 = await 文本读(txt版本号路径);
console.assert(
    txt版本号 !== version,
    `警告：版本号txt文件版本没有变化，当前版本为${version}`
);
await writeFile(txt版本号路径, version);

// 更新choco包文件
const choco包文件路径 = "./DevTools/choco/CapsLockX.nuspec";
const choco包文 = await 文本读(choco包文件路径);
const CDATA包装 = (文本) =>
    `<![CDATA[${文本
        .slice(0, 3888)
        .split(/\r?\n/g)
        .slice(0, -1)
        .join("\n")}]]>`;
const README = CDATA包装(await 文本读("README.MD")).replace(/[一-龥]+/g, "");
const CHANGELOG = CDATA包装(await 文本读("CHANGELOG.MD")).replace(
    /[一-龥]+/g,
    ""
);
const 新版本包文 = choco包文
    .replace(
        /(<version>)(.*?)(<\/version>)/,
        (_, $1, $2, $3) => $1 + version + $3
    )
    // .replace(/(<description>)([\s\S]*?)(<\/description>)/, (_, $1, $2, $3) => $1 + README + $3)
    .replace(
        /(<releaseNotes>)([\s\S]*?)(<\/releaseNotes>)/,
        (_, $1, $2, $3) => $1 + CHANGELOG + $3
    );
console.assert(
    choco包文 !== 新版本包文,
    `警告：Choco包文版本没有变化，当前版本为${version}`
);
await writeFile(choco包文件路径, 新版本包文);

console.log("chore(release): " + version);
