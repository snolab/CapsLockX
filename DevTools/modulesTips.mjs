import fs from "fs";
import "sno-utils";
import _ from "lodash";
import pinyin from "pinyin";
import { 表键筛 } from "sno-utils";
const 全部提取 = (s, p) => {
  return (s.match(RegExp(p, p.flags)) || []).map((e) => {
    console.log(e);
    return e.match(RegExp(p, p.flags.replace("g", ""))).slice(1);
  });
};
const 按表替换 = (s, 表) =>
  Object.entries(表).reduce((s, [k, v]) => s.replace(RegExp(v, v.flags), k), s);
const 表按键排序 = (表) =>
  Object.fromEntries(
    Object.entries(表).sort(([k0, v0], [k1, v1]) => k0.localeCompare(k1))
  );

// const 默认辅助键序列 = "#^+!".split('')

const hkp = (s) =>
  按表替换(s, {
    "_->_": /&/g,
    _CLX_: /\*/g,
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
const 热键列提取 = (文件内容) => {
  // const 函数列 = 全部提取(文件内容, /^(\S+)\(\)\s*?{/);
  const 条件列 = 全部提取(
    文件内容,
    /(?<=\n|^)#if[ \t]*(.*)\s*([\s\S]*?)(?:$|(?=#if))/gi
  );
  const 热键列 = 条件列.map(([条件, code]) => {
    条件 = 条件.trim();
    // console.log("条件", 条件, "\n", code.slice(0, 10));
    const hkfn = 全部提取(code, /^(?!;)(.*?)::\s*?(\S+)\(\)/gm);
    const hkde = 全部提取(code, /^(?!;)(.*?)::.*?;+\s*?(\S+)?$/gm);
    const 指令热键表 = 表按键排序(
      Object.fromEntries([
        ...hkfn.map(([hk, fn]) => [fn, hkp(hk)]),
        ...hkde.map(([hk, de]) => [de, hkp(hk)]),
      ])
    );
    return [条件, 指令热键表];
  });
  return 热键列;
};
const ModulesPath = "Modules";
const 热键合并表 = (列) => {
  return 列.reduce((表, [条件, 热键补表]) => {
    表[条件] = { ...表[条件], ...热键补表 };
    return 表;
  }, {});
};
const 模块文件列表 = await fs.promises.readdir(ModulesPath);
const 条件热键对表列 = (
  await Promise.all(
    模块文件列表
      .filter((e) => e.match(/.ahk$/))
      .map(async (文件名) => {
        const 文件内容 = await fs.promises.readFile(
          `${ModulesPath}/${文件名}`,
          "utf8"
        );
        return 热键列提取(文件内容);
      })
  )
).flat();
const 条件热键表 = 热键合并表(条件热键对表列);
const 函数条件热键表 = 表键筛((键) => 键.match(/^\S+\(\)$/))(条件热键表);
console.log(JSON.stringify(函数条件热键表, null, 4));

const QuickTipsUpdate = async (函数条件热键表) => {
  const prefix = `    msg := ""`;
  const content = Object.entries(函数条件热键表)
    .map(
      ([条件, 热键表]) => `
    if (${条件}) {
${Object.entries(热键表)
  .map(([描述, 热键]) => `        msg .= "|\t${热键 + "\t|\t" + 描述}\t|"`)
  .join("\n")}
    }`
    )
    .join("\n");
  const suffix = "    ToolTip %msg%";
  const QuickTips = [prefix, content, suffix].join("\n");
  // console.log(QuickTips);
  const QuickTipsAHK = "Core/CapsLockX-QuickTips.ahk";
  const src = await fs.promises.readFile(QuickTipsAHK, "utf8");
  const dst =
    "\uFEFF" +
    src
      .replace(/^\uFEFF/, "")
      .replace(
        /^QuickTips\(\)\s*?{[\s\S]*?^}/gim,
        `QuickTips(){\n${QuickTips}\n}`
      );
  console.log(dst);
  await fs.promises.writeFile(QuickTipsAHK, dst);
};

await QuickTipsUpdate(函数条件热键表);
