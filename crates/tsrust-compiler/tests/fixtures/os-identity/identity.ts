declare function require(name: string): {platform(): string; arch(): string};
const os = require("node:os");
console.log(os.platform(), os.arch());
