// Explicit counterparts for the Rust-native UTF8/scalar contract.
function scalarCharAt(text: string, index: number): string {
 if (index < 0) return "";
 let position = 0;
 for (const scalar of text) { if (position === index) return scalar; position++; }
 return "";
}
function workload(n: number, seed: number, input: string, coordinates: number): number {
 const width = coordinates; const values: string[] = []; let j = 0;
 while (j < 129) { values.push(scalarCharAt(input, (j * 37 + seed) % width)); j = j + 1; }
 let sum = seed; let i = 0;
 while (i < n) {
  const from = (i * 13 + seed) % 387 - 193; const choice = (i + seed) % 4;
  if (choice === 0) { if (values.includes("A", from)) { sum = sum + 1; } }
  else { if (choice === 1) { if (values.includes("😀", from)) { sum = sum + 1; } }
  else { if (choice === 2) { if (values.includes(scalarCharAt(input, (i * 17 + seed) % (width + 2)), from)) { sum = sum + 1; } }
  else { if (values.includes("missing", from)) { sum = sum + 1; } } } }
  i = i + 1;
 }
 return sum;
}
function main() { console.log(workload(2, 7, "A😀e\u0301Z", 5)); }
