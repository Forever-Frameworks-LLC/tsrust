declare function require(name: string): { posix: { basename(path: string, suffix?: string): string } };
const pathModule = require("node:path").posix;
export function report(): string { return pathModule.basename("/tmp/report.txt", ".txt"); }
