import callerPath from "caller-path";
import fs from "fs";
import path from "path";
import pkg from "rambda";
const { filter, pipe, sortBy, dissoc } = pkg;
const caller = callerPath({ depth: 1 });
if (!caller) throw new Error(`Cannot find caller`);
process.chdir(
  path.dirname(path.resolve(caller.replace("file:///", ""), "../")),
);
const replaceMapper = (s = "", 表: Record<string, RegExp>) =>
  Object.entries(表).reduce((s, [k, v]) => s.replace(RegExp(v, v.flags), k), s);
const hkp = (s = "") =>
  replaceMapper(s, {
    // &
    "_ _": /&/g,
    // sort
    "^#": /#\^/g,
    "#+": /\+#/g,
    "^!": /!\^/g,
    "#!": /!#/g,
    "+!": /\+!/g,
    "+^": /\^\+/g,
    // convert
    _Win_: /#/g,
    _Alt_: /!/g,
    _Shift_: /\+/g,
    _Ctrl_: /\^/g,
    "": /[~\$]/g,
  })
    .replace(/_+/g, " ")
    .trim()
    .replace(/\s+/g, " + ");
// .replace(, "ctrl"+alt)
const 热键列提取 = (文件内容: string) => {
  // const 函数列 = 全部提取(文件内容, /^(\S+)\(\)\s*?{/);
  const conds = [
    ...文件内容.matchAll(
      /(?<=\n|^)(#if\w*)[ \t]*(.*)\s*([\s\S]*?)(?:$|(?=#if))/gi,
    ),
  ];
  const 热键列 = conds.flatMap(([_, head, rawCond, code]) => {
    let trimedCond = rawCond.trim().replace(/;.*/, "");
    const cond =
      (head.match(/^#IfWinActive/) && `WinActive("${trimedCond}")`) ||
      (head.match(/^#IfWinExist/) && `WinExist("${trimedCond}")`) ||
      trimedCond;
    // console.log("条件", 条件, "\n", code.slice(0, 10));
    const ent = pipe(
      () => [
        ...[...code.matchAll(/^(?!;)(.*?)::\s*?(\S+)\(\)/gm)]
          .filter(([, hk]) => !hk.match(/Up$/))
          .map(([, hk, fn]) => [fn, hkp(hk)]),
        ...[...code.matchAll(/^(?!;)(.*?)::.*?;+\s*?(\S+)$/gm)]
          .filter(([, hk]) => !hk.match(/Up$/))
          .map(([, hk, de]) => [de, hkp(hk)]),
      ],
      sortBy(([k, v]) => k),
    )();
    if (!Object.entries(Object.fromEntries(ent)).length) {
      return [];
    }
    return [[cond, Object.fromEntries(ent)]];
  });
  return 热键列;
};
const ModulesPath = "Modules";
const moduleFiles = await fs.promises.readdir(ModulesPath);
const 条件热键对表列 = (
  await Promise.all(
    moduleFiles
      .filter((e) => e.match(/.ahk$/))
      .map(async (文件名) => {
        return 热键列提取(
          await fs.promises.readFile(`${ModulesPath}/${文件名}`, "utf8"),
        );
      }),
  )
).flat();
const 条件热键表 = (function () {
  return 条件热键对表列.reduce((表, [条件, 热键补表]) => {
    表[条件] = { ...表[条件], ...热键补表 };
    return 表;
  }, {});
})();
const 函数条件热键表 = filter((v, k) => !!k.match(/^\S+\(\)$/), 条件热键表);
const 非函数条件热键表 = filter((v, k) => !k.match(/^\S+\(\)$/), 条件热键表);

console.error(JSON.stringify(dissoc("", 非函数条件热键表), null, 4));

const QuickTipsUpdate = async (条件热键表) => {
  const prefix = `    msg := ""`;
  const ahkFuncTry =
    (动作生成函数) =>
    ([条件, 热键表]) =>
      `
    try{
        if (Func("${条件.replace(/\(\)$/, "")}").Call()) {\n
            ${Object.entries(热键表).map(动作生成函数).join("\n            ")}
        }
    }`;
  const ahkTryIf =
    (动作生成函数) =>
    ([条件, 热键表]) =>
      `
    try{
        if (${条件 || "True"}) {
            ${Object.entries(热键表).map(动作生成函数).join("\n            ")}
        }
    }`;
  const msgAppend = ([描述, 热键]) =>
    `msg .= "|\t${热键 + "\t|\t" + 描述}\t|\`n"`;
  const content = [
    ...Object.entries(非函数条件热键表).map(ahkTryIf(msgAppend)),
    ...Object.entries(函数条件热键表).map(ahkFuncTry(msgAppend)),
  ].join("\n");
  const suffix = "    return msg";
  const QuickTips = [prefix, content, suffix].join("\n");
  const QuickTipsAHK = "Core/CapsLockX-QuickTips.ahk";
  const src = await fs.promises.readFile(QuickTipsAHK, "utf8");
  const dst = `\uFEFF${src
    .replace(/^\uFEFF/, "")
    .replace(
      /^QuickTips\(\)\s*?{[\s\S]*?^}/gim,
      `QuickTips(){\n${QuickTips}\n}`,
    )}`;
  await fs.promises.writeFile(QuickTipsAHK, dst);
};

if (import.meta.main) {
  await QuickTipsUpdate(条件热键表);
}
