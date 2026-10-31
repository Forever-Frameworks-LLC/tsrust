// Independent standard Rust reference; retain pattern/default/control operations.
type String = std::borrow::Cow<'static, str>;
type Text = std::string::String;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn owned(value: impl Into<Text>) -> String {
    String::Owned(value.into())
}
#[derive(Clone, Copy)]
enum Actual {
    Null,
    Undefined,
    Number(f64),
}
impl Actual {
    fn truthy(self) -> bool {
        matches!(self,Self::Number(v)if v!=0.0&&!v.is_nan())
    }
    fn kind(self) -> &'static str {
        match self {
            Self::Null => "object",
            Self::Undefined => "undefined",
            Self::Number(_) => "number",
        }
    }
    fn nan(self) -> bool {
        matches!(self,Self::Number(v)if v.is_nan())
    }
}
fn same(a: Actual, b: Actual) -> bool {
    match (a, b) {
        (Actual::Null, Actual::Null) | (Actual::Undefined, Actual::Undefined) => true,
        (Actual::Number(a), Actual::Number(b)) => {
            a.to_bits() == b.to_bits() || a.is_nan() && b.is_nan()
        }
        _ => false,
    }
}
fn close(a: f64, b: f64, digits: f64) -> bool {
    if a.is_infinite() && b.is_infinite() {
        a == b
    } else {
        (a - b).abs() < 10_f64.powf(-digits) / 2.0
    }
}

struct Measured {
    length: Option<f64>,
}
fn function0(value0: i64, value1: i64) -> i64 {
    let mut checksum = 0;
    for i in 0..value0 {
        let kind = add(mul(i, 13), value1) % 6;
        let actual = match kind {
            0 => Actual::Null,
            1 => Actual::Undefined,
            2 => Actual::Number(0.0),
            3 => Actual::Number(f64::NAN),
            4 => Actual::Number(-0.0),
            _ => Actual::Number(7.0),
        };
        if kind == 0 {
            assert!(matches!(actual, Actual::Null));
        } else if kind == 1 {
            assert!(matches!(actual, Actual::Undefined));
        } else {
            assert!(!matches!(actual, Actual::Undefined));
        }
        if kind == 5 {
            assert!(actual.truthy());
        } else {
            assert!(!actual.truthy());
        }
        if kind == 3 {
            assert!(actual.nan());
        } else {
            assert!(same(actual, actual));
        }
        if kind == 0 {
            assert!(!matches!(actual, Actual::Undefined));
        } else if kind == 1 {
            assert!(matches!(actual, Actual::Undefined));
        } else {
            assert!(!matches!(actual, Actual::Null));
        }
        if kind == 5 {
            assert!(actual.truthy());
        } else {
            assert!(!actual.truthy());
        }
        if kind == 3 {
            assert!(!same(actual, Actual::Number(0.0)));
        } else {
            assert!(!actual.nan());
        }
        let expected_type = if kind == 0 {
            "object"
        } else if kind == 1 {
            "undefined"
        } else {
            "number"
        };
        assert_eq!(actual.kind(), expected_type);
        assert_ne!(actual.kind(), "string");
        let numeric = kind as f64;
        assert!(numeric > -1.0);
        assert!(numeric >= numeric);
        assert!(numeric < 6.0);
        assert!(numeric <= numeric);
        assert!(!(numeric > 6.0));
        assert!(!(numeric >= 6.0));
        assert!(!(numeric < -1.0));
        assert!(!(numeric <= -1.0));
        assert!(close(numeric, numeric, 2.0));
        assert!(!close(numeric, numeric + 0.125, 5.0));
        let text = if kind == 0 {
            "😀".to_owned()
        } else {
            "x".repeat(usize::try_from(kind).unwrap())
        };
        let expected_length = if kind == 0 { 2.0 } else { kind as f64 };
        let length = text.encode_utf16().count() as f64;
        assert_eq!(length, expected_length);
        assert_ne!(length, expected_length + 1.0);
        let measured = Some(numeric);
        let record = Measured { length: measured };
        assert_eq!(record.length.unwrap(), numeric);
        assert_ne!(record.length.unwrap(), numeric + 1.0);
        let code = if matches!(actual, Actual::Null) {
            1
        } else if matches!(actual, Actual::Undefined) {
            2
        } else if actual.truthy() {
            4
        } else {
            8
        };
        checksum = add(mul(checksum, 131), code) % 1000000007;
    }
    checksum
}
fn main() {}
