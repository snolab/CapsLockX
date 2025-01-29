import { readFile, writeFile } from "fs/promises";

if (import.meta.main) {
  await versioning();
}

//
async function versioning() {
  // get current version
  const { version } = JSON.parse(await readFileUtf8("package.json"));

  // console.log('get version', version)

  //
  (await fileContentReplace(
    "./DevTools/setup.iss",
    /AppVersion=.*/,
    "AppVersion=" + version,
  )) || console.warn("setup.iss version not changed");

  // update version.txt
  (await fileContentReplace("./Core/version.txt", /.*/, version)) ||
    console.warn("txt版本号路径版本没有变化");

  // update choco package version
  const choco包文件路径 = "./DevTools/choco/CapsLockX.nuspec";
  const choco包文 = await readFileUtf8(choco包文件路径);
  const CDATA包装 = (文本) =>
    `<![CDATA[${文本
      .slice(0, 3888)
      .split(/\r?\n/g)
      .slice(0, -1)
      .join("\n")}]]>`;
  const CHANGELOG = CDATA包装(await readFileUtf8("CHANGELOG.MD")).replace(
    /[一-龥]+/g,
    "",
  );
  //
  const 新版本包文 = choco包文
    .replace(
      /(<version>)(.*?)(<\/version>)/,
      (_, $1, $2, $3) => $1 + version + $3,
    )
    // .replace(/(<description>)([\s\S]*?)(<\/description>)/, (_, $1, $2, $3) => $1 + README + $3)
    .replace(
      /(<releaseNotes>)([\s\S]*?)(<\/releaseNotes>)/,
      (_, $1, $2, $3) => $1 + CHANGELOG + $3,
    );
  console.assert(
    choco包文 !== 新版本包文,
    `警告：Choco包文版本没有变化，当前版本为${version}`,
  );
  await writeFile(choco包文件路径, 新版本包文);

  console.log("chore(release): " + version);
}

async function readFileUtf8(f: string) {
  return await readFile(f, { encoding: "utf8" });
}
async function fileContentReplace(
  filePath: string,
  regexp: RegExp,
  replacement: string,
) {
  const content = await readFileUtf8(filePath);
  const newContent = content.replace(regexp, replacement);
  const replaced = newContent !== content;
  if (replaced) await writeFile(filePath, newContent);
  return replaced;
}
