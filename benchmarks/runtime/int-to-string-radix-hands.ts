type int = number;
function main() {
    const values: int[] = [0, 1, -1, 35, -35, 9007199254740991, -9007199254740991];
    const radices: int[] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36];
    let i = 0;
    while (i < radices.length) {
        let j = 0;
        while (j < values.length) {
            const selected: string = values[j].toString(radices[i]);
            console.log(selected); j = j + 1;
        }
        i = i + 1;
    }
}
