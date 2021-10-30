import fs from "fs";
const lsModules = await fs.promises.readdir("Modules");
const ModuleAHKS = lsModules.filter((e) => e.match(/\.ahk/));
const ModuleMDS = lsModules.filter((e) => e.match(/\.md/));
console.log(ModuleAHKS);
console.log(ModuleMDS);
ModuleAHKS.map((e) => `Modules/${e}`);
