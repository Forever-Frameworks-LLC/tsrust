type int = number;
function workload(n: int, seed: int): int[] {
    const initialValues: int[] = [257, 129, 64, 16, 8, 3, 2, 1];
    const rhsValues: int[] = [0, 1, 2, 3, 7, 31, 129, 1024];
    const output: int[] = []; let i = 0;
    while (i < n) {
        let target: int = initialValues[(i * 17 + seed) % 8];
        const rhsAt = (i * 13 + seed) % 8;
        const observed: int = (target = rhsValues[rhsAt]);
        output.push(observed); output.push(target);
        i = i + 1;
    }
    return output;
}
