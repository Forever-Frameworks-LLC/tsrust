<!-- tsrust-docs
{"id":"reference-standard-library-numeric","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:numeric","name":"toInt"},{"receiver":"std:numeric","name":"toNumber"},{"receiver":"std:numeric","name":"toFloat32"},{"receiver":"std:numeric","name":"checkSigned"},{"receiver":"std:numeric","name":"checkUnsigned"},{"receiver":"std:numeric","name":"parseInt"},{"receiver":"std:numeric","name":"NumericError"},{"receiver":"std:numeric","name":"parseNumber"},{"receiver":"std:numeric","name":"checkedAdd"},{"receiver":"std:numeric","name":"checkedSubtract"},{"receiver":"std:numeric","name":"checkedMultiply"},{"receiver":"std:numeric","name":"checkedDivide"},{"receiver":"std:numeric","name":"checkedRemainder"}]}
-->
# Numeric

Convert numbers at explicit boundaries. std:numeric.parseInt validates a complete signed decimal string and reports syntax or range failures. The global parseInt has different prefix-parsing behavior.

## Strict parsing and checked integers

`parseNumber(text)` parses all floating-point text into `Result<number, NumericError>`, including signed zero, decimal exponents, NaN and infinity. Whitespace and partial text return Syntax; exponent overflow/underflow follows binary64 infinity/signed-zero behavior. `parseInt(text, radix?)` accepts bases 2–36, default 10; invalid base returns Radix, malformed text Syntax and signed 64-bit overflow Range. No prefixes or separators are accepted. `checkedAdd`, `checkedSubtract`, `checkedMultiply`, `checkedDivide` and `checkedRemainder` return exact int Results. Overflow returns Range; divide/remainder by zero return Zero; MIN with divisor -1 returns Range. Division truncates toward zero and remainder follows the dividend sign. Existing operators are unchanged.
