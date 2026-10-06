type int = number;
function workload(n: int, seed: int): int {
    let digest = seed; let i = 0;
    while (i < n) {
        const left = (i * 37 + seed) % 2001 - 1000;
        const result = -left;
        digest = (digest * 131 + result + 1100) % 1000000007;
        i = i + 1;
    }
    return digest;
}
