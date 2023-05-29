import { readFile } from "fs/promises";
import { glob } from "glob";
async function ahksf() {
  return await glob("**/*.ahk");
  // return ahks;
}
ahksf()
const read = async (f: string) => ({ f, s: await readFile(f, "utf8") });
let ahkitems;
ahkitems = await Promise.all(ahks.map(read));

// %%
const warns = {
  if未加括号: /^\s*if[ ]*[^( ]/gim,
};
function warn(f: string) {
  return mapValues(warns, (reg, key) =>
    [...s.matchAll(reg)].map((m) => ({
      peek: s.slice(m.index, m[0].length),
      m: String(m),
    })),
  );
}
JSON.stringify(await Promise.all(ahkitems.map(warn)), null, 2);

// %%

// %%

// %%
