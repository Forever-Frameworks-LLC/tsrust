type int = number;
function receiver(trace: int[], empty: boolean): int[] { trace.push(1); if (empty) { return []; } return [2, 3]; }
function main() { const trace: int[] = []; console.log(receiver(trace, false).toString()); console.log(trace[0]); const other: int[] = []; console.log(receiver(other, true).toString()); console.log(other[0]); }
