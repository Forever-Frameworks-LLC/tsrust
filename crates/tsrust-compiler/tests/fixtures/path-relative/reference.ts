const path = require("node:path");
if (process.version !== "v24.11.1") throw Error("Pinned Node reference required");
const paths = JSON.parse(process.argv[1]);
const contexts = JSON.parse(process.argv[2]);
const original = process.env;
for (const context of contexts) {
    process.cwd = () => context.cwd;
    process.env = { ...original, "=D:": "D:\\saved" };
    const selected = context.windows ? path.win32 : path.posix;
    for (const from of paths) for (const to of paths) {
        console.log(Buffer.from(selected.relative(from, to)).toString("hex"));
    }
}
