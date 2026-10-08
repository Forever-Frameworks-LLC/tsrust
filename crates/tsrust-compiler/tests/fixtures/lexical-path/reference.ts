// Independent reference program. Run with the pinned Node runtime.
const pathModule = require("node:path");
const family = process.argv[2];
const paths = JSON.parse(process.argv[3]);
for (const windows of [false, true]) {
    const convention = windows ? pathModule.win32 : pathModule.posix;
    for (const path of paths) {
        if (family === "parent-extension") {
            for (const method of ["dirname", "extname"]) {
                console.log(Buffer.from(convention[method](path)).toString("hex"));
            }
        } else if (family === "absolute-normalize") {
            const normalized = convention.normalize(path);
            console.log(JSON.stringify([normalized, convention.isAbsolute(path), convention.normalize(normalized)]));
        } else {
            throw new Error("Unknown path reference family");
        }
    }
}
