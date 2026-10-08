type int = number;
function workload(n: int, seed: int, input: string): int[][] {
    const output: int[][] = [];
    let i = 0;
    while (i < n) {
        const value = i % 5 + seed % 5 + 1;
        const base = [value, value + 1];
        const factorial = function fact(count: int): int {
            return count <= 1 ? 1 : count * fact(count - 1);
        };
        let total = 0;
        {
            const walk = function down(depth: int): void {
                if (depth === 0) {
                    return;
                }

                total += depth + base[0];
                down(depth - 1);
            };
            walk(3);
        }

        const mapped = [1, 2].map(function shift(item: int): int {
            return item + base[1];
        });
        const make = (offset: int) => function add(count: int): int {
            return count <= 0 ? offset : 1 + add(count - 1);
        };
        const pick = function choose<T>(left: T, right: T, first: boolean): T {
            return first ? left : right;
        };
        output.push([factorial(value), total, mapped[0], mapped[1], make(value)(2), pick(value, 0, true), pick("ab", "c", false).length]);
        i += 1;
    }

    return output;
}

function main() {
    const result = workload(3, 0, "named");
}
