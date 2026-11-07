import { readFile } from "node:fs/promises";

const wasm = await WebAssembly.compile(await readFile(process.argv[2]));
const imports = Object.create(null);
let initializesExternrefs = false;
for (const { module, name, kind } of WebAssembly.Module.imports(wasm)) {
  if (kind !== "function") {
    throw new Error(`Unexpected import ${module}.${name} (${kind})`);
  }
  imports[module] ??= Object.create(null);
  imports[module][name] = () => {
    throw new Error(`Unexpected call to ${module}.${name} before initialization`);
  };
  initializesExternrefs ||= name === "__wbindgen_init_externref_table";
}
if (initializesExternrefs) {
  const { __wbindgen_externrefs } = new WebAssembly.Instance(wasm, imports).exports;
  __wbindgen_externrefs.set(0, undefined);
}
