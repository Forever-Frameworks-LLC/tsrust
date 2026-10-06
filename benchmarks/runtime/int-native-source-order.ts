function left(): number { console.log(1); return 6; }
function right(): number { console.log(2); return 3; }
function index(): number { console.log(3); return 0; }
function rhs(values: number[]): number { console.log(4); values[0] = 100; return 3; }
function main() {
    console.log(+left()); console.log(~left());
    console.log(left() & right()); console.log(left() | right()); console.log(left() ^ right());
    console.log(left() << right()); console.log(left() >> right()); console.log(left() >>> right()); console.log(left() ** right());
    const values: number[] = [6];
    values[index()] &= rhs(values); console.log(values[0]); values[0] = 6;
    values[index()] |= rhs(values); console.log(values[0]); values[0] = 6;
    values[index()] ^= rhs(values); console.log(values[0]); values[0] = 6;
    values[index()] <<= rhs(values); console.log(values[0]); values[0] = 6;
    values[index()] >>= rhs(values); console.log(values[0]); values[0] = 6;
    values[index()] >>>= rhs(values); console.log(values[0]); values[0] = 6;
    values[index()] **= rhs(values); console.log(values[0]);
}
