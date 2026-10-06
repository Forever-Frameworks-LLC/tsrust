declare function require(name: string): { posix: { basename(path: string, suffix?: string): string } };
const basename = require("node:path").posix.basename;
export function value(): number { return 7; }
