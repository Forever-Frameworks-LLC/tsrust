function workload(n: number, seed: number, input: string): number {
    const values: string[] = [input, "é".toUpperCase(), "OMEGA".toLowerCase(), "😀".toUpperCase(), "alpha".toUpperCase()];
    let sum = seed; let i = 0;
    while (i < n) {
        values.reverse();
        let j = 0;
        while (j < values.length) {
            let code = 0;
            if (values[j] === input) { code = 17; }
            else {
                if (values[j] === "É") { code = 23; }
                else {
                    if (values[j] === "omega") { code = 31; }
                    else {
                        if (values[j] === "😀") { code = 41; }
                        else { if (values[j] === "ALPHA") { code = 53; } }
                    }
                }
            }
            sum = (sum * 131 + code) % 1000000007; j = j + 1;
        }
        i = i + 1;
    } return sum;
}
function main() { console.log(workload(2, 7, " AbCé😀 ")); }
