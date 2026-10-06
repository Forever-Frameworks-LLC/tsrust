function int_native_unary_plus(a: number, b: number = 0): number { return +a; }
function int_native_bit_not(a: number, b: number = 0): number { return ~a; }
function int_native_bit_and(a: number, b: number = 0): number { return a & b; }
function int_native_bit_or(a: number, b: number = 0): number { return a | b; }
function int_native_bit_xor(a: number, b: number = 0): number { return a ^ b; }
function int_native_shift_left(a: number, b: number = 0): number { return a << b; }
function int_native_shift_right(a: number, b: number = 0): number { return a >> b; }
function int_native_shift_right_unsigned(a: number, b: number = 0): number { return a >>> b; }
function int_native_power(a: number, b: number = 0): number { return a ** b; }
function int_native_compound_bit_and(a: number, b: number = 0): number { let value = a; value &= b; return value; }
function int_native_compound_bit_or(a: number, b: number = 0): number { let value = a; value |= b; return value; }
function int_native_compound_bit_xor(a: number, b: number = 0): number { let value = a; value ^= b; return value; }
function int_native_compound_shift_left(a: number, b: number = 0): number { let value = a; value <<= b; return value; }
function int_native_compound_shift_right(a: number, b: number = 0): number { let value = a; value >>= b; return value; }
function int_native_compound_shift_right_unsigned(a: number, b: number = 0): number { let value = a; value >>>= b; return value; }
function int_native_compound_power(a: number, b: number = 0): number { let value = a; value **= b; return value; }
