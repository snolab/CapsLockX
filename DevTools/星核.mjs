import "sno-utils";
import fs from "fs";
const ahk核表 = `
值 值 值 判断      为  a?b:c
值 对象吗          为  IsObject(a)
值 函数吗          为  IsFunc(a)
值 非              为  !a
值 值 化           为  0?a:b
数 数 化           为  0?a:b
数 化假            为  0?a:False
数 化真            为  0?a:True
数 化零            为  0?a:0
数 化一            为  0?a:1
数 反              为  -a
数 倒              为  1/a
数 Log             为  Log(a)
数 Exp             为  Exp(a)
数 符号            为  a==0?0:a>0?1:-1
数 绝对            为  a<0?-a:a
数 取整            为  a|0
数 取零            为  a-a|0
列 长度            为  a.Length()
列 克隆            为  a.Clone()
串 取函            为  Func(a)
串 长度            为  StrLen(a)
数 数 加           为  a+b
数 数 减           为  a-b
数 数 乘           为  a*b
数 数 除           为  a/b
数 数 与           为  a&&b
数 数 或           为  a||b
数 数 大于         为  a>b
数 数 小于         为  a<b
数 数 等于         为  a==b
数 数 严等         为  a===b
数 数 幂           为  a**b
数 数 位与         为  a&b
数 数 位或         为  a|0
列 列 直#<数数>     为 列列函映(a,b,Func("##"))
阵 阵 直#<数数>     为 阵阵函映(a,b,Func("##"))
列 数 直#<数数>     为 列数函映(a,b,Func("##"))
阵 数 直#<数数>     为 阵数函映(a,b,Func("##"))
列 之#<数数>        为 Func("#").Call(a[1],a[2])
列 #<数>映          为 列函映(a,Func("#"))
列 #<数>筛          为 列函筛(a,Func("#"))
列 #<数数>归        为 列函值归(a,Func("##"),b)
阵 #<数>映          为 阵函映(a,Func("#"))
`
    .split(/\r?\n/)
    .filter((e) => e)
    .map((e) => e.split(/\s+/));

const ahk核表1 = ahk核表.map((行) => {
    // let [我串, 为串] = 行.split(/\w+?为/);
    // let psr = 我串.split(/\s+/).slice(0, -1);
    // let fn = 我串.split(/\s+/).pop();

    // const exp = 为串.replace(/\\n/g, "\n");
    const exp = JSON.parse("'" + 行.slice(-1)[0] + "'");
    const psr = 行.slice(0, -3);
    const psrs = psr.join("");
    const fn = 行.slice(-3)[0];
    const ps = [...new Set(exp.match(/\b[a-z]\b/g) || [])].sort();
    return { exp, psr, psrs, fn, ps };
    // console.log(fn, exp, ps);
});

const re = ahk核表1
    .map(({ ps, exp, psr, psrs, fn }) => {
        // console.log(fn);
        const m = fn.match(/#<(.*?)>/);
        if (m) {
            return (
                ahk核表1
                    .filter(({ exp }) => !exp.match("#"))
                    // .filter((e) => psr.length === e.psr.length)
                    .filter(({ psrs }) => psrs === m[1])
                    .map(
                        ({ ps: ps2, exp: exps, psr, psrs: psrs2, fn: fn2 }) => {
                            const fn1 =
                                // psrs2 +
                                psrs +
                                fn.replace(/#<.*?>/, fn2) +
                                "(" +
                                ps +
                                "){\n    return " +
                                exp.replace(/#+/, psrs2 + fn2) +
                                "\n}";
                            // console.log(fn1);
                            return fn1;
                        }
                    )
                    .join("\n")
            );
        } else {
            const fn1 = psrs + fn + "(" + ps + "){\n    return " + exp + "\n}";
            // console.log(fn1);
            return fn1;
        }
    })
    .join("\n");

console.log(re);
await fs.promises.writeFile(
    "Modules/01-向量加速模型.ahk",
    (
        await fs.promises.readFile("Modules/01-向量加速模型.ahk", "UTF8")
    ).replace(
        /;;; 注：以下代码由 星核.mjs 自动生成[\s\S]*?;;; 注：以上代码由 星核.mjs 自动生成/gim,
        `;;; 注：以下代码由 星核.mjs 自动生成\n${re}\n;;; 注：以上代码由 星核.mjs 自动生成`
    )
);
