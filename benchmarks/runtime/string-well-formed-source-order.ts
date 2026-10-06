/// <reference lib="es2024.string" />
type int = number;
type Trace = { code: int; };
function receiver(trace: Trace): string { trace.code = trace.code * 10 + 1; return "A\u0000é😀é\uFEFF􏿿"; }
function main() {
    const trace: Trace = { code: 0 };
    console.log(receiver(trace).isWellFormed()); console.log(trace.code);
    trace.code = 0;
    console.log(receiver(trace).toWellFormed()); console.log(trace.code);
    const empty: string = "";
    console.log(empty.isWellFormed()); console.log(empty.toWellFormed()); console.log(empty);
    const owned: string = "é".repeat(2);
    console.log(owned.isWellFormed()); console.log(owned.toWellFormed()); console.log(owned);
    const literal: string = "\uFEFFé😀";
    console.log(literal.isWellFormed()); console.log(literal.toWellFormed()); console.log(literal);
}
