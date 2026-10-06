type State = { value: number; };
function workload(n: number, seed: number): number {
    let checksum = seed;
    let i = 0;
    while (i < n) {
        const left: State = { value: i + seed };
        const right: State = { value: i + seed };
        const actions: (() => number)[] = [
            (): number => { left.value = left.value + 1; return left.value; },
            (): number => { right.value = right.value + 2; return right.value; }
        ];
        const forward = actions.reduce((sum: number, action: () => number): number => { return sum + action(); }, 0);
        const reverse = actions.reduceRight((sum: number, action: () => number): number => { return sum * 10 + action(); }, 0);
        checksum = checksum + forward + reverse + actions[0]() + actions[1]();
        i = i + 1;
    }
    return checksum;
}

function main() { console.log(workload(2, 7)); }
