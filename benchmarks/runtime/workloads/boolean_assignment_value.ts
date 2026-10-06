type int = number;
function workload(n: int, seed: int): int[] {
    const initialValues: boolean[] = [false, false, true, true];
    const rhsValues: boolean[] = [false, true, false, true];
    const output: int[] = []; let i = 0;
    while (i < n) {
        let target: boolean = initialValues[(i * 17 + seed) % 4];
        const rhsAt = (i * 13 + seed) % 4;
        const observed: boolean = (target = rhsValues[rhsAt]);
        let resultBit = 0; if (observed) { resultBit = 1; }
        let targetBit = 0; if (target) { targetBit = 1; }
        output.push(resultBit); output.push(targetBit);
        i = i + 1;
    }
    return output;
}
