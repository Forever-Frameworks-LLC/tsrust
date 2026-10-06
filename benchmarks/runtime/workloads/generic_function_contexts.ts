function workload(n: number, seed: number, input: string): number[][] {
    const output: number[][] = [];
    let i = 0;
    while (i < n) {
        const base = i % 5 + seed % 5 + 1;
        let calls = 0;
        function select<T>(value: T): T { calls += 1; return value; }
        let numberResult = 0;
        let stringLength = 0;
        let arrayFirst = 0;
        {
            const numbers: (value: number) => number = select;
            numberResult = numbers(base);
        }
        {
            const strings: (value: string) => string = select;
            stringLength = strings("owner").length;
        }
        {
            const arrays: (value: number[]) => number[] = select;
            const result = arrays([base + 1, base + 2]);
            arrayFirst = result[0];
        }
        function head<T>(values: T[]): T { calls += 1; return values.shift()!; }
        let headResult = 0;
        let remaining = 0;
        {
            const first: (values: number[]) => number = head;
            const values = [base + 3, base + 4];
            headResult = first(values);
            remaining = values.length;
        }
        output.push([numberResult, stringLength, arrayFirst, headResult, remaining, calls, input.length]);
        i += 1;
    }
    return output;
}
function main() {}
