function rustTrim(text: string): string { return text.replace(/^\p{White_Space}+|\p{White_Space}+$/gu, ""); }
function workload(n: number, seed: number): string[] {
 const values: string[] = [];
 for (let i = 0; i < n; i++) { const name = (i + seed) % 2 === 0 ? " SAM " : "  Pat  "; values.push(rustTrim(name).toLowerCase()); }
 return values;
}
