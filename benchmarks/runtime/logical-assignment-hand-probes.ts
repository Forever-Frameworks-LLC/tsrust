type int = number;
function andCell(old: boolean, right: boolean): int[] {
    let target = old; const observed = (target &&= right);
    let resultBit = 0; if (observed) { resultBit = 1; }
    let targetBit = 0; if (target) { targetBit = 1; }
    return [resultBit, targetBit];
}
function orCell(old: boolean, right: boolean): int[] {
    let target = old; const observed = (target ||= right);
    let resultBit = 0; if (observed) { resultBit = 1; }
    let targetBit = 0; if (target) { targetBit = 1; }
    return [resultBit, targetBit];
}
