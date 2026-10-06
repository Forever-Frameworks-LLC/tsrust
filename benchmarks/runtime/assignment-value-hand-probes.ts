type int = number;
function intCell(old: int, right: int): int[] { let target = old; const observed = (target = right); return [observed, target]; }
function numberCell(old: number, right: number): number[] { let target = old; const observed = (target = right); return [observed, target]; }
function booleanCell(old: boolean, right: boolean): int[] { let target = old; const observed = (target = right); let resultBit = 0; if (observed) { resultBit = 1; } let targetBit = 0; if (target) { targetBit = 1; } return [resultBit, targetBit]; }
