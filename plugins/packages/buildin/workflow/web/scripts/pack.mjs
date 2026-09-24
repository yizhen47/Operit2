import { readFile, readdir, writeFile } from "node:fs/promises";
import { zipSync } from "fflate";
const files = {
  "manifest.json": new Uint8Array(await readFile("manifest.json")),
  "resources/workflow.html": new Uint8Array(
    await readFile("resources/workflow.html"),
  ),
};
files["resources/THIRD_PARTY_NOTICES.txt"] = new Uint8Array(
  await readFile("resources/THIRD_PARTY_NOTICES.txt"),
);
/** Adds executable package modules while excluding previous archives. */
async function collect(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = directory + "/" + entry.name;
    if (entry.isDirectory()) await collect(path);
    else if (entry.name.endsWith(".js"))
      files[path] = new Uint8Array(await readFile(path));
  }
}
await collect("dist");
await writeFile("dist/workflow.toolpkg", zipSync(files));
