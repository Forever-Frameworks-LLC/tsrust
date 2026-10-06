function rustTrim(text: string): string { return text.replace(/^\p{White_Space}+|\p{White_Space}+$/gu, ""); }
function workload(n: number, seed: number, input: string): string[] { const normalized: string[] = []; for (let index = 0; index < n; index++) normalized.push(rustTrim(input).toLowerCase()); return normalized; }
