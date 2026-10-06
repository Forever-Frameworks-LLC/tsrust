type int = number;
function clone(value: string): string { return value; }
function left(trace: int[], tag: int): int | null | undefined {
    trace.push(1); if (tag === 0) { return null; } if (tag === 1) { return undefined; } return 0;
}
function right(trace: int[]): int { trace.push(2); return 7; }
function residual(trace: int[], value: null | undefined): null | undefined { trace.push(3); return value; }
function stringRight(trace: int[]): string { trace.push(4); return "é😀\u0000\uFEFF"; }
function main() {
    const first: int[] = []; let a: int | null | undefined = left(first, 2);
    const ar = (a ??= right(first)); console.log(ar); console.log(a); console.log(first.join(","));
    const second: int[] = []; let b: int | null | undefined = left(second, 0);
    const br = (b ??= right(second)); console.log(br); console.log(b); console.log(second.join(","));
    const third: int[] = []; let c: int | null | undefined = left(third, 1);
    const cr = (c ??= right(third)); console.log(cr); console.log(c); console.log(third.join(","));
    const nilTrace: int[] = []; let nil: null | undefined = null;
    const nr = (nil ??= residual(nilTrace, undefined)); console.log(String(nr)); console.log(String(nil)); console.log(nilTrace.join(","));
    let nil2: null | undefined = undefined;
    const nr2 = (nil2 ??= residual(nilTrace, null)); console.log(String(nr2)); console.log(String(nil2)); console.log(nilTrace.join(","));
    const ownedTrace: int[] = []; let text: string | null | undefined = null;
    const observed = (text ??= stringRight(ownedTrace));
    console.log(observed); console.log(text); console.log(ownedTrace.join(","));
    const saved: string = clone(observed); console.log(saved);
}
