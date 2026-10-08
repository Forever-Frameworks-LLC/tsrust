declare function require(name: string): { join(...paths: string[]): string };
const join = require("node:path").join;
function workload(n: number, seed: number): string[] {
    const paths: string[] = []; paths.push("é😊"); let j = 0;
    while (j < 64) { paths.push("文"); j = j + 1; }
    paths.push(".."); paths.push("résumé/");
    const values: string[] = []; let i = 0;
    while (i < n) { values.push(join(...paths)); i = i + 1; }
    return values;
}
