import { build } from "esbuild";
import { mkdir, writeFile, readdir, readFile } from "node:fs/promises";

const result = await build({
  entryPoints: ["web/app.tsx"],
  bundle: true,
  minify: true,
  write: false,
  metafile: true,
  outfile: "app.js",
  target: ["es2020"],
  jsx: "automatic",
  legalComments: "eof",
});
const js = result.outputFiles.find((file) => file.path.endsWith(".js")).text;
const css = result.outputFiles.find((file) => file.path.endsWith(".css")).text;
await mkdir("resources", { recursive: true });
await writeFile(
  "resources/workflow.html",
  `<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>工作流</title><style>${css}</style></head><body><div id="root"></div><script>${js.replaceAll("</script", "<\\/script")}</script></body></html>`,
);
const packages = new Set(
  Object.keys(result.metafile.inputs)
    .map((path) => /^node_modules\/((?:@[^/]+\/)?[^/]+)/.exec(path)?.[1])
    .filter(Boolean),
);
const notices = [];
for (const name of [...packages].sort()) {
  const directory = "node_modules/" + name;
  const metadata = JSON.parse(
    await readFile(directory + "/package.json", "utf8"),
  );
  const licenses = (await readdir(directory)).filter((file) =>
    /^licen[sc]e(?:\.|$)/i.test(file),
  );
  notices.push(`${name} ${metadata.version} (${metadata.license})`);
  for (const license of licenses)
    notices.push(await readFile(directory + "/" + license, "utf8"));
}
await writeFile("resources/THIRD_PARTY_NOTICES.txt", notices.join("\n\n"));
