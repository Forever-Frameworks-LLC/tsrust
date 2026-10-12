// Distinct zero-sized values; void remains the procedure result ().
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TsrNull;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TsrUndefined;
impl std::fmt::Display for TsrNull {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output.write_str("null")
    }
}
impl std::fmt::Display for TsrUndefined {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output.write_str("undefined")
    }
}
