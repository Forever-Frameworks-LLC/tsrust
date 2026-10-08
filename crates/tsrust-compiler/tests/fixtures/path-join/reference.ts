declare function require(name: string): any;
const {join: compose} = require("node:path"); const {strictEqual} = require("node:assert");
const first = "é😊".repeat(1); const stored = compose; strictEqual(compose(), "."); strictEqual(compose("", "", ""), "."); strictEqual(compose(first, "..", "文"), "文"); strictEqual(first, "é😊"); const parts = ["a", "..", "report"]; strictEqual(stored(...parts), "report"); strictEqual(parts[0], "a"); const annotated: (...paths:string[])=>string=compose; strictEqual(annotated("a", "..", "archive"), "archive");
