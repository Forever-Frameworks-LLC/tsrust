// Text conversion writes directly into one owned UTF-8 buffer. Inputs remain
// borrowed, including inline Option-backed arrays and nested owning containers.
trait TsrJoinText {
    fn append_join_text(&self, output: &mut std::string::String);
    fn append_locale_text(&self, output: &mut std::string::String) { self.append_join_text(output); }
}
fn tsr_append_join<'a, T: TsrJoinText + ?Sized + 'a>(
    output: &mut std::string::String,
    values: impl Iterator<Item = &'a T>,
    separator: &str,
) {
    for (index, value) in values.enumerate() {
        if index != 0 {
            output.push_str(separator);
        }
        value.append_join_text(output);
    }
}
fn tsr_array_join<'a, T: TsrJoinText + ?Sized + 'a>(
    values: impl Iterator<Item = &'a T>,
    separator: &str,
) -> String {
    let mut output = std::string::String::new();
    tsr_append_join(&mut output, values, separator);
    String::Owned(output)
}
impl TsrJoinText for str {
    fn append_join_text(&self, output: &mut std::string::String) {
        output.push_str(self);
    }
}
impl<T: TsrJoinText + ?Sized> TsrJoinText for &T {
    fn append_locale_text(&self, output: &mut std::string::String) { T::append_locale_text(*self, output); }
    fn append_join_text(&self, output: &mut std::string::String) {
        T::append_join_text(*self, output);
    }
}
impl TsrJoinText for String {
    fn append_join_text(&self, output: &mut std::string::String) {
        output.push_str(self.as_ref());
    }
}
impl TsrJoinText for i64 {
    fn append_locale_text(&self, output: &mut std::string::String) { output.push_str(tsr_int_locale_string(*self).as_ref()); }
    fn append_join_text(&self, output: &mut std::string::String) {
        use std::fmt::Write as _;
        write!(output, "{self}").expect("standard String formatting");
    }
}
impl TsrJoinText for bool {
    fn append_join_text(&self, output: &mut std::string::String) {
        output.push_str(if *self { "true" } else { "false" });
    }
}
impl TsrJoinText for f64 {
    fn append_locale_text(&self, output: &mut std::string::String) { output.push_str(tsr_number_locale_string(*self).as_ref()); }
    fn append_join_text(&self, output: &mut std::string::String) {
        use std::fmt::Write as _;
        if self.is_nan() {
            output.push_str("NaN");
        } else if *self == f64::INFINITY {
            output.push_str("Infinity");
        } else if *self == f64::NEG_INFINITY {
            output.push_str("-Infinity");
        } else if *self == 0.0 {
            output.push('0');
        } else if !(1e-6..1e21).contains(&self.abs()) {
            // Standard scientific digits, with the existing String() exponent
            // spelling. Common finite values need no intermediate allocation.
            let mut scientific = format!("{self:e}");
            tsr_number_tie(*self, &mut scientific, 0);
            let (digits, exponent) = scientific
                .split_once('e')
                .expect("standard scientific format");
            let exponent: i32 = exponent.parse().expect("standard scientific exponent");
            write!(output, "{digits}e{exponent:+}").expect("standard String formatting");
        } else {
            let start = output.len();
            write!(output, "{self}").expect("standard String formatting");
            tsr_number_tie(*self, output, start);
        }
    }
}
impl<T: TsrJoinText> TsrJoinText for Vec<T> {
    fn append_locale_text(&self, output: &mut std::string::String) { tsr_append_locale(output, self.iter()); }
    fn append_join_text(&self, output: &mut std::string::String) {
        tsr_append_join(output, self.iter(), ",");
    }
}
impl<T: TsrJoinText> TsrJoinText for [T] {
    fn append_locale_text(&self, output: &mut std::string::String) { tsr_append_locale(output, self.iter()); }
    fn append_join_text(&self, output: &mut std::string::String) {
        tsr_append_join(output, self.iter(), ",");
    }
}
impl<T: TsrJoinText, const N: usize> TsrJoinText for [T; N] {
    fn append_locale_text(&self, output: &mut std::string::String) { tsr_append_locale(output, self.iter()); }
    fn append_join_text(&self, output: &mut std::string::String) {
        tsr_append_join(output, self.iter(), ",");
    }
}

fn tsr_append_locale<'a, T: TsrJoinText + ?Sized + 'a>(output: &mut std::string::String, values: impl Iterator<Item=&'a T>) {
    for (index,value) in values.enumerate() {
        if index != 0 { output.push(','); }
        value.append_locale_text(output);
    }
}
fn tsr_array_locale<'a, T: TsrJoinText + ?Sized + 'a>(values: impl Iterator<Item=&'a T>) -> String {
    let mut output=std::string::String::new();
    tsr_append_locale(&mut output,values);
    String::Owned(output)
}
