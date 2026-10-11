#[path="named_function_syntax.rs"]
mod named_function_syntax;
#[path="numeric_literals.rs"]
mod numeric_literals;
#[path="match_syntax.rs"]
pub mod match_syntax;
use crate::diagnostic::{Diagnostic, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Name,
    String,
    TemplateHead,
    TemplateMiddle,
    TemplateTail,
    Integer,
    Number,
    Symbol,
    End,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub text: String,
    pub span: Span,
    pub kind: TokenKind,
}

pub fn lex(source: &str) -> Result<Vec<Token>, Diagnostic> {
    let mut tokens = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    // Only interpolating templates need a frame; nested braces belong to its expression.
    let mut templates: Vec<(usize, usize)> = Vec::new();
    while i < bytes.len() {
        let start = i;
        let c = source[i..]
            .chars()
            .next()
            .expect("valid character boundary");
        if c.is_whitespace() {
            i += c.len_utf8();
            continue;
        }
        if source[i..].starts_with("//") {
            i = source[i..]
                .find(['\n', '\r', '\u{2028}', '\u{2029}'])
                .map_or(source.len(), |n| i + n);
            continue;
        }
        if source[i..].starts_with("/*") {
            let end = source[i + 2..].find("*/").ok_or_else(|| {
                Diagnostic::new(
                    "syntax",
                    "Unterminated comment.",
                    Span::new(i, source.len()),
                )
            })?;
            i += end + 4;
            continue;
        }
        if c == '`' {
            let (value, end, interpolation) = template_chunk(source, i)?;
            i = end;
            if interpolation {
                if templates.len() >= 128 {
                    return Err(Diagnostic::new(
                        "unsupported",
                        "Template nesting exceeds the prototype limit of 128.",
                        Span::new(start, i),
                    ));
                }
                templates.push((start, 0));
            }
            tokens.push(Token {
                text: value,
                span: Span::new(start, i),
                kind: if interpolation {
                    TokenKind::TemplateHead
                } else {
                    TokenKind::String
                },
            });
            continue;
        }
        if c == '}' && templates.last().is_some_and(|(_, braces)| *braces == 0) {
            let (value, end, interpolation) = template_chunk(source, i)?;
            i = end;
            if !interpolation {
                templates.pop();
            }
            tokens.push(Token {
                text: value,
                span: Span::new(start, i),
                kind: if interpolation {
                    TokenKind::TemplateMiddle
                } else {
                    TokenKind::TemplateTail
                },
            });
            continue;
        }
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            tokens.push(Token {
                text: source[start..i].to_owned(),
                span: Span::new(start, i),
                kind: TokenKind::Name,
            });
            continue;
        }
        if bytes[i].is_ascii_digit()
            || (bytes[i] == b'.' && bytes.get(i + 1).is_some_and(u8::is_ascii_digit))
        {
            let (token,end)=numeric_literals::scan(source,start)?;
            tokens.push(token);i=end;
            continue;
        }
        if matches!(c, '"' | '\'') {
            let (value, end) = string_literal(source, i)?;
            i = end;
            tokens.push(Token {
                text: value,
                span: Span::new(start, i),
                kind: TokenKind::String,
            });
            continue;
        }
        if let Some(symbol) = [
            "&&=", "||=", ">>>=", "<<=", ">>=", "**=", "+=", "-=", "*=", "/=", "%=", "&=", "|=",
            "^=", "++", "--",
        ]
        .into_iter()
        .find(|symbol| source[i..].starts_with(symbol))
        {
            i += symbol.len();
            tokens.push(Token {
                text: symbol.into(),
                span: Span::new(start, i),
                kind: TokenKind::Symbol,
            });
            continue;
        }
        if source[i..].starts_with("?.") && !bytes.get(i + 2).is_some_and(u8::is_ascii_digit) {
            i += 2;
            tokens.push(Token {
                text: "?.".into(),
                span: Span::new(start, i),
                kind: TokenKind::Symbol,
            });
            continue;
        }
        if let Some(symbol) = [
            "...", "??=", "===", "!==", ">>>", "<<", ">>", "**", "=>", "<=", ">=", "&&", "||", "??",
        ]
        .into_iter()
        .find(|symbol| source[i..].starts_with(symbol))
        {
            i += symbol.len();
            tokens.push(Token {
                text: symbol.into(),
                span: Span::new(start, i),
                kind: TokenKind::Symbol,
            });
            continue;
        }
        if ["==", "!="]
            .iter()
            .any(|symbol| source[i..].starts_with(symbol))
        {
            return Err(Diagnostic::new(
                "unsupported",
                "Use strict scalar equality (=== or !==); coercive equality is unsupported.",
                Span::new(start, start + 2),
            ));
        }
        if "{}[]():;,.=+-*/%!<>&|^~?".contains(c) {
            if let Some((_, braces)) = templates.last_mut() {
                if c == '{' {
                    *braces += 1;
                } else if c == '}' {
                    // A zero-depth closing brace was consumed by template_chunk above.
                    *braces -= 1;
                }
            }
            i += 1;
            tokens.push(Token {
                text: c.to_string(),
                span: Span::new(start, i),
                kind: TokenKind::Symbol,
            });
            continue;
        }
        return Err(Diagnostic::new(
            "unsupported",
            format!("Unsupported character {c:?} in this subset."),
            Span::new(i, i + c.len_utf8()),
        ));
    }
    if let Some((start, _)) = templates.last() {
        return Err(Diagnostic::new(
            "syntax",
            "Unterminated template interpolation.",
            Span::new(*start, source.len()),
        ));
    }
    tokens.push(Token {
        text: "<end>".into(),
        span: Span::new(source.len(), source.len()),
        kind: TokenKind::End,
    });
    Ok(tokens)
}

fn unpaired_surrogate(span: Span) -> Diagnostic {
    Diagnostic::new(
        "syntax",
        "Unpaired surrogate escape is not valid Unicode in a UTF-8 string.",
        span,
    )
}

fn push_literal_scalar(
    value: &mut String,
    pending: &mut Option<(u32, Span)>,
    scalar: u32,
    span: Span,
) -> Result<(), Diagnostic> {
    if let Some((high, high_span)) = pending.take() {
        if !(0xdc00..=0xdfff).contains(&scalar) {
            return Err(unpaired_surrogate(high_span));
        }
        value.push(char::from_u32(0x10000 + ((high - 0xd800) << 10) + scalar - 0xdc00).unwrap());
    } else if (0xd800..=0xdbff).contains(&scalar) {
        *pending = Some((scalar, span));
    } else if (0xdc00..=0xdfff).contains(&scalar) {
        return Err(unpaired_surrogate(span));
    } else {
        value.push(char::from_u32(scalar).expect("validated Unicode escape or source scalar"));
    }
    Ok(())
}

fn string_literal(source: &str, start: usize) -> Result<(String, usize), Diagnostic> {
    let quote = source.as_bytes()[start] as char;
    let mut i = start + 1;
    let mut value = String::new();
    let mut pending = None;
    let bad = || {
        Diagnostic::new(
            "syntax",
            "Invalid quoted string literal.",
            Span::new(start, source.len()),
        )
    };
    while i < source.len() {
        let character_start = i;
        let c = source[i..].chars().next().ok_or_else(&bad)?;
        i += c.len_utf8();
        let decoded = match c {
            c if c == quote => {
                if let Some((_, span)) = pending {
                    return Err(unpaired_surrogate(span));
                }
                return Ok((value, i));
            }
            '\n' | '\r' => return Err(bad()),
            '\\' => literal_escape(source, &mut i).ok_or_else(&bad)?,
            c => Some(c as u32),
        };
        if let Some(scalar) = decoded {
            push_literal_scalar(
                &mut value,
                &mut pending,
                scalar,
                Span::new(character_start, i),
            )?;
        }
    }
    Err(bad())
}

// Called immediately after a backslash. None rejects a malformed escape;
// Some(None) is an empty line continuation. Both literal kinds share this decoder.
fn literal_escape(source: &str, i: &mut usize) -> Option<Option<u32>> {
    let escaped = source[*i..].chars().next()?;
    *i += escaped.len_utf8();
    Some(match escaped {
        '\'' | '"' | '\\' => Some(escaped as u32),
        'n' => Some('\n' as u32),
        'r' => Some('\r' as u32),
        't' => Some('\t' as u32),
        'b' => Some(8),
        'f' => Some(12),
        'v' => Some(11),
        '0' if !source.as_bytes().get(*i).is_some_and(u8::is_ascii_digit) => Some(0),
        '0'..='9' => return None,
        '\n' | '\u{2028}' | '\u{2029}' => None,
        '\r' => {
            if source.as_bytes().get(*i) == Some(&b'\n') {
                *i += 1;
            }
            None
        }
        'x' => Some(hex_digits(source, i, 2)?),
        'u' => Some(unicode_escape(source, i)?),
        // Identity escapes include escaped backticks and dollars in templates.
        c => Some(c as u32),
    })
}

// start is an opening backtick or the closing brace of an interpolation.
// The bool distinguishes a ${ boundary from a closing backtick.
fn template_chunk(source: &str, start: usize) -> Result<(String, usize, bool), Diagnostic> {
    let mut i = start + 1;
    let mut value = String::new();
    let mut pending = None;
    let bad = || {
        Diagnostic::new(
            "syntax",
            "Invalid or unterminated template literal.",
            Span::new(start, source.len()),
        )
    };
    while i < source.len() {
        let character_start = i;
        let c = source[i..].chars().next().ok_or_else(&bad)?;
        i += c.len_utf8();
        if c == '`' || (c == '$' && source.as_bytes().get(i) == Some(&b'{')) {
            if let Some((_, span)) = pending {
                return Err(unpaired_surrogate(span));
            }
            let interpolation = c == '$';
            if interpolation {
                i += 1;
            }
            return Ok((value, i, interpolation));
        }
        let decoded = match c {
            '\\' => literal_escape(source, &mut i).ok_or_else(&bad)?,
            '\r' => {
                if source.as_bytes().get(i) == Some(&b'\n') {
                    i += 1;
                }
                Some('\n' as u32)
            }
            c => Some(c as u32),
        };
        if let Some(scalar) = decoded {
            push_literal_scalar(
                &mut value,
                &mut pending,
                scalar,
                Span::new(character_start, i),
            )?;
        }
    }
    Err(bad())
}

fn hex_digits(source: &str, i: &mut usize, width: usize) -> Option<u32> {
    let slice = source.as_bytes().get(*i..*i + width)?;
    let mut value = 0;
    for b in slice {
        value = value * 16 + (*b as char).to_digit(16)?;
    }
    *i += width;
    Some(value)
}

fn unicode_escape(source: &str, i: &mut usize) -> Option<u32> {
    if source.as_bytes().get(*i) != Some(&b'{') {
        return hex_digits(source, i, 4);
    }
    *i += 1;
    let start = *i;
    let mut value = 0u32;
    while let Some(&b) = source.as_bytes().get(*i) {
        if b == b'}' {
            if *i == start {
                return None;
            }
            *i += 1;
            return Some(value);
        }
        value = value
            .checked_mul(16)?
            .checked_add((b as char).to_digit(16)?)?;
        if value > 0x10ffff {
            return None;
        }
        *i += 1;
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    String,
    Int,
    Number,
    Boolean,
    Null,
    Undefined,
    Void,
    Record(usize),
    Enum(usize, EnumKind),
    Set(usize),
    Map(usize),
    Array(ArrayElement),
    Closure(usize),
    Callable(usize),
    Callback(usize),
    /// An owned storage family; its ID is separate from callback ownership paths.
    StoredCallable(usize),
    /// A scoped family retaining the lifetimes of its concrete environments.
    BorrowedCallable(usize),
    Union(UnionId),
    /// A scoped readonly view of an owned nullable payload.
    BorrowedUnion(UnionId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ArrayElement {
    Int(u8),
    Number(u8),
    Boolean(u8),
    Null(u8),
    Undefined(u8),
    String(u8),
    Record(usize, u8),
    Enum(usize, EnumKind, u8),
    Set(usize, u8),
    Map(usize, u8),
    StoredCallable(usize, u8),
    /// A tagged data union leaf; each element carries its own alternative.
    Union(UnionId, u8),
}
impl ArrayElement {
    /// Preserve the concrete leaf identity while adding one array container.
    /// Unsupported leaves and ranks above the 128-container limit return None.
    pub(crate) fn from_value_type(ty: Type) -> Option<Self> {
        Some(match ty {
            Type::Int => Self::Int(0),
            Type::Number => Self::Number(0),
            Type::Boolean => Self::Boolean(0),
            Type::Null => Self::Null(0),
            Type::Undefined => Self::Undefined(0),
            Type::String => Self::String(0),
            Type::Record(id) => Self::Record(id, 0),
            Type::Enum(id, kind) => Self::Enum(id, kind, 0),
            Type::Set(id) => Self::Set(id, 0),
            Type::Map(id) => Self::Map(id, 0),
            Type::StoredCallable(id) => Self::StoredCallable(id, 0),
            Type::Union(id) => Self::Union(id, 0),
            Type::Array(inner) if inner.depth() < 127 => inner.with_depth(inner.depth() + 1),
            _ => return None,
        })
    }
    /// Additional array dimensions around this element's scalar or record leaf.
    pub fn depth(self) -> u8 {
        match self {
            Self::Int(depth)
            | Self::Number(depth)
            | Self::Boolean(depth)
            | Self::Null(depth)
            | Self::Undefined(depth)
            | Self::String(depth)
            | Self::Enum(_, _, depth) | Self::Set(_, depth) | Self::Map(_, depth)
            | Self::Record(_, depth)
            | Self::StoredCallable(_, depth)
            | Self::Union(_, depth) => depth,
        }
    }
    pub fn with_depth(self, depth: u8) -> Self {
        match self {
            Self::Int(_) => Self::Int(depth),
            Self::Number(_) => Self::Number(depth),
            Self::Boolean(_) => Self::Boolean(depth),
            Self::Null(_) => Self::Null(depth),
            Self::Undefined(_) => Self::Undefined(depth),
            Self::String(_) => Self::String(depth),
            Self::Record(id, _) => Self::Record(id, depth),
            Self::Enum(id, kind, _) => Self::Enum(id, kind, depth),
            Self::Set(id, _) => Self::Set(id, depth),
            Self::Map(id, _) => Self::Map(id, depth),
            Self::StoredCallable(id, _) => Self::StoredCallable(id, depth),
            Self::Union(id, _) => Self::Union(id, depth),
        }
    }
    pub fn ty(self) -> Type {
        if self.depth() > 0 {
            return Type::Array(self.with_depth(self.depth() - 1));
        }
        match self {
            Self::Int(_) => Type::Int,
            Self::Number(_) => Type::Number,
            Self::Boolean(_) => Type::Boolean,
            Self::Null(_) => Type::Null,
            Self::Undefined(_) => Type::Undefined,
            Self::String(_) => Type::String,
            Self::Record(id, _) => Type::Record(id),
            Self::Enum(id, kind, _) => Type::Enum(id, kind),
            Self::Set(id, _) => Type::Set(id),
            Self::Map(id, _) => Type::Map(id),
            Self::StoredCallable(id, _) => Type::StoredCallable(id),
            Self::Union(id, _) => Type::Union(id),
        }
    }
}

/// The copy property is computed when canonical members are interned. The
/// identifier carries no runtime payload and remains stable within one program.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UnionId {
    pub(crate) index: u32,
    pub(crate) copy: bool,
}
impl UnionId {
    pub(crate) fn index(self) -> usize {
        self.index as usize
    }
}

impl Type {
    pub fn union(self) -> Option<UnionId> {
        match self {
            Self::Union(id) | Self::BorrowedUnion(id) => Some(id),
            _ => None,
        }
    }
    pub fn owning(self) -> Self {
        match self {
            Self::BorrowedUnion(id) => Self::Union(id),
            _ => self,
        }
    }
    pub fn is_copy(self) -> bool {
        matches!(
            self,
            Self::Int | Self::Number | Self::Boolean | Self::Null | Self::Undefined | Self::Enum(_, _)
        ) || matches!(self, Self::Union(id) if id.copy)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericConstant {
    MathE,
    MathLn10,
    MathLn2,
    MathLog10E,
    MathLog2E,
    MathPi,
    MathSqrtHalf,
    MathSqrt2,
    NumberEpsilon,
    NumberMax,
    NumberMin,
    NumberMaxSafe,
    NumberMinSafe,
    NumberNaN,
    NumberPositiveInfinity,
    NumberNegativeInfinity,
}
impl NumericConstant {
    pub fn lookup(namespace: &str, name: &str) -> Option<Self> {
        Some(match (namespace, name) {
            ("Math", "E") => Self::MathE,
            ("Math", "LN10") => Self::MathLn10,
            ("Math", "LN2") => Self::MathLn2,
            ("Math", "LOG10E") => Self::MathLog10E,
            ("Math", "LOG2E") => Self::MathLog2E,
            ("Math", "PI") => Self::MathPi,
            ("Math", "SQRT1_2") => Self::MathSqrtHalf,
            ("Math", "SQRT2") => Self::MathSqrt2,
            ("Number", "EPSILON") => Self::NumberEpsilon,
            ("Number", "MAX_VALUE") => Self::NumberMax,
            ("Number", "MIN_VALUE") => Self::NumberMin,
            ("Number", "MAX_SAFE_INTEGER") => Self::NumberMaxSafe,
            ("Number", "MIN_SAFE_INTEGER") => Self::NumberMinSafe,
            ("Number", "NaN") => Self::NumberNaN,
            ("Number", "POSITIVE_INFINITY") => Self::NumberPositiveInfinity,
            ("Number", "NEGATIVE_INFINITY") => Self::NumberNegativeInfinity,
            _ => return None,
        })
    }
    pub fn rust(self) -> &'static str {
        match self {
            Self::MathE => "std::f64::consts::E",
            Self::MathLn10 => "std::f64::consts::LN_10",
            Self::MathLn2 => "std::f64::consts::LN_2",
            Self::MathLog10E => "std::f64::consts::LOG10_E",
            Self::MathLog2E => "std::f64::consts::LOG2_E",
            Self::MathPi => "std::f64::consts::PI",
            Self::MathSqrtHalf => "std::f64::consts::FRAC_1_SQRT_2",
            Self::MathSqrt2 => "std::f64::consts::SQRT_2",
            Self::NumberEpsilon => "f64::EPSILON",
            Self::NumberMax => "f64::MAX",
            Self::NumberMin => "f64::from_bits(1)",
            Self::NumberMaxSafe => "9007199254740991.0f64",
            Self::NumberMinSafe => "-9007199254740991.0f64",
            Self::NumberNaN => "f64::NAN",
            Self::NumberPositiveInfinity => "f64::INFINITY",
            Self::NumberNegativeInfinity => "f64::NEG_INFINITY",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetMethod { Add, Has, Delete, Clear, Values, Entries(usize), IsSubsetOf, IsSupersetOf, IsDisjointFrom, Union, Intersection, Difference, SymmetricDifference }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapMethod { Set, Get, Has, Delete, Clear, Keys, Values, Entries(usize) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderedSetMethod { Add, Has, Delete, Clear }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeapMethod { Push, Peek, Pop, Clear }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DequeMethod { PushFront, PushBack, PopFront, PopBack, Front, Back, At, Clear }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallTarget {
    DequeMethod(DequeMethod),
    HeapMethod(HeapMethod),
    OrderedSetMethod(OrderedSetMethod),
    MapMethod(MapMethod),
    SetMethod(SetMethod),    ConditionalClosure(usize),
    ConditionalCallback(usize),
    StoredCallable(usize),
    Number,
    StoredCallableField(usize),
    BooleanConversion,
    StringConversion,
    ArrayIsArray,
    ObjectIs,
    ObjectKeys,
    ObjectHasOwn,
    ObjectValues,
    /// Carries the [string, value] pair tuple record.
    ObjectEntries(usize),
    StringRaw,
    StringFromCodePoint,
    StringFromCharCode,
    StringCodePointSlot,
    StringCharCodeAt,
    StringNormalize,
    ArrayOf,
    NumberParseFloat,
    NumberParseInt,
    NumberIsFinite,
    NumberIsNaN,
    NumberIsInteger,
    NumberIsSafeInteger,
    MathAbs,
    MathFloor,
    MathCeil,
    MathTrunc,
    MathSqrt,
    MathRound,
    MathSign,
    MathFround,
    MathF16round,
    MathRandom,
    MathMin,
    MathMax,
    MathClz32,
    MathImul,
    MathAcos,
    MathAcosh,
    MathAsin,
    MathAsinh,
    MathAtan,
    MathAtanh,
    MathCbrt,
    MathCos,
    MathCosh,
    MathExp,
    MathExpm1,
    MathLog,
    MathLog10,
    MathLog1p,
    MathLog2,
    MathSin,
    MathSinh,
    MathTan,
    MathTanh,
    MathAtan2,
    MathPow,
    MathHypot,
    ArrayPush,
    ArrayUnshift,
    ArraySplice,
    ArrayRemove,
    ArrayAt,
    ArrayPop,
    ArrayShift,
    ArrayReverse,
    ArraySort,
    ArrayToSorted,
    ArrayFill,
    ArrayCopyWithin,
    ArrayConcat,
    ArrayToLocaleString,
    ArrayJoin,
    ArraySlice,
    PatternRest(usize, usize),
    RecordSpread(usize),
    ArrayToReversed,
    ArrayKeys,
    ArrayEntries(usize),
    ArrayWith,
    ArrayToSpliced,
    ArrayReduce,
    ArrayReduceRight,
    ArrayForEach,
    ArrayMap,
    MapGroupBy,
    ArrayFlat(u8),
    ArrayFlatMap,
    ArrayFilter,
    ArraySome,
    ArrayEvery,
    ArrayFind,
    ArrayFindLast,
    ArrayFindIndex,
    ArrayFindLastIndex,
    ArrayIncludes,
    ArrayIndexOf,
    ArrayLastIndexOf,
    NativeBinding(usize),
    Clone,
    Log,
    Trim,
    TrimStart,
    TrimEnd,
    Lower,
    Upper,
    StringSlice,
    StringSubstring,
    StringSubstr,
    StringIdentity,
    StringIsWellFormed,
    BooleanIdentity,
    BooleanToString,
    NumberIdentity,
    IntIdentity,
    IntToLocaleString,
    NumberToLocaleString,
    IntToString,
    NumberToString,
    NumberToFixed,
    NumberToExponential,
    NumberToPrecision,
    StringCharAt,
    StringStartsWith,
    StringEndsWith,
    StringIncludes,
    StringIndexOf,
    StringLastIndexOf,
    StringHtml(HtmlWrapper),
    StringConcat,
    StringPadStart,
    StringPadEnd,
    StringRepeat,
    StringReplace,
    StringReplaceAll,
    StringSplit,
    Function(usize),
    /// A stable declared environment with this invocation's materialized target.
    ForwardFunction(usize),
    Closure(usize),
    ClosureMethod(usize),
    Callback(usize),
}

// Keep only a compact method kind in each expression's CallTarget. Fat string
// metadata would widen every Expr and every recursively retained parser frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HtmlWrapper {
    Anchor,
    Big,
    Blink,
    Bold,
    Fixed,
    Fontcolor,
    Fontsize,
    Italics,
    Link,
    Small,
    Strike,
    Sub,
    Sup,
}

impl HtmlWrapper {
    pub fn markup(self) -> (&'static str, Option<&'static str>) {
        match self {
            Self::Anchor => ("a", Some("name")),
            Self::Big => ("big", None),
            Self::Blink => ("blink", None),
            Self::Bold => ("b", None),
            Self::Fixed => ("tt", None),
            Self::Fontcolor => ("font", Some("color")),
            Self::Fontsize => ("font", Some("size")),
            Self::Italics => ("i", None),
            Self::Link => ("a", Some("href")),
            Self::Small => ("small", None),
            Self::Strike => ("strike", None),
            Self::Sub => ("sub", None),
            Self::Sup => ("sup", None),
        }
    }
}

pub fn string_html_target(name: &str) -> Option<CallTarget> {
    let wrapper = match name {
        "anchor" => HtmlWrapper::Anchor,
        "big" => HtmlWrapper::Big,
        "blink" => HtmlWrapper::Blink,
        "bold" => HtmlWrapper::Bold,
        "fixed" => HtmlWrapper::Fixed,
        "fontcolor" => HtmlWrapper::Fontcolor,
        "fontsize" => HtmlWrapper::Fontsize,
        "italics" => HtmlWrapper::Italics,
        "link" => HtmlWrapper::Link,
        "small" => HtmlWrapper::Small,
        "strike" => HtmlWrapper::Strike,
        "sub" => HtmlWrapper::Sub,
        "sup" => HtmlWrapper::Sup,
        _ => return None,
    };
    Some(CallTarget::StringHtml(wrapper))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    /// Checked tag test for one alternative of a closed union.
    VariantIs(Type),
    /// Internal default-pattern selection; missing positions produce undefined.
    PatternSelection,
    /// Checked-only identity for a statically selected nullish operand.
    CoalesceSelected,
    /// Checked-only owning collection of source rest arguments.
    RestPack,
    /// Source call argument expansion; never an ordinary unary value.
    Spread,
    /// Concrete environment adapter from rest to a fixed accepted signature.
    CallableRestAdapter,
    /// Checked supplied slot carrying an exclusive payload borrow by value.
    ParameterMutableInput,
    /// Checked selected default/supplied owners mapped to an existing family.
    ParameterDefaultStore,
    /// Checked-only ownership wrapper for a once-annotated local environment.
    CallableOnce,
    CallableStore,
    /// Checked owning record conversion preserves each original environment.
    RecordCarrier,
    /// Source optional boundary; lowered by the contiguous chain checker.
    OptionalBoundary,
    /// Checked identity for a statically present optional call.
    OptionalSelected,
    /// Source postfix assertion excluding null and undefined.
    NonNull,
    Negate,
    Positive,
    Not,
    Typeof,
    Void,
    BitNot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
    ShiftRightUnsigned,
    Exponent,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    And,
    Or,
    Coalesce,
    /// Internal destructuring default: select fallback for undefined only.
    PatternDefault,
}
impl BinaryOp {
    pub(crate) fn bitwise_or_power(self) -> bool {
        matches!(
            self,
            Self::BitAnd
                | Self::BitOr
                | Self::BitXor
                | Self::ShiftLeft
                | Self::ShiftRight
                | Self::ShiftRightUnsigned
                | Self::Exponent
        )
    }
    fn assignment(text: &str) -> Option<Self> {
        Some(match text {
            "??=" => Self::Coalesce,
            "+=" => Self::Add,
            "-=" => Self::Subtract,
            "*=" => Self::Multiply,
            "/=" => Self::Divide,
            "%=" => Self::Remainder,
            "**=" => Self::Exponent,
            "&=" => Self::BitAnd,
            "|=" => Self::BitOr,
            "^=" => Self::BitXor,
            "<<=" => Self::ShiftLeft,
            ">>=" => Self::ShiftRight,
            ">>>=" => Self::ShiftRightUnsigned,
            _ => return None,
        })
    }
    fn parse(text: &str) -> Option<(Self, u8)> {
        Some(match text {
            "??" => (Self::Coalesce, 1),
            "||" => (Self::Or, 2),
            "&&" => (Self::And, 3),
            "|" => (Self::BitOr, 4),
            "^" => (Self::BitXor, 5),
            "&" => (Self::BitAnd, 6),
            "===" => (Self::Equal, 7),
            "!==" => (Self::NotEqual, 7),
            "<" => (Self::Less, 8),
            "<=" => (Self::LessEqual, 8),
            ">" => (Self::Greater, 8),
            ">=" => (Self::GreaterEqual, 8),
            "<<" => (Self::ShiftLeft, 9),
            ">>" => (Self::ShiftRight, 9),
            ">>>" => (Self::ShiftRightUnsigned, 9),
            "+" => (Self::Add, 10),
            "-" => (Self::Subtract, 10),
            "*" => (Self::Multiply, 11),
            "/" => (Self::Divide, 11),
            "%" => (Self::Remainder, 11),
            "**" => (Self::Exponent, 12),
            _ => return None,
        })
    }
}

/// Specialized callback body and the access it lends to each array element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraversalMethod {
    pub id: usize,
    pub access: crate::Mode,
}

/// `Number` member named by a global number name: a function when `callee`
/// is set, a constant otherwise.
pub fn global_number_member(name: &str, callee: bool) -> Option<&'static str> {
    Some(match (name, callee) {
        ("parseInt", true) => "parseInt",
        ("parseFloat", true) => "parseFloat",
        ("isNaN", true) => "isNaN",
        ("isFinite", true) => "isFinite",
        ("NaN", false) => "NaN",
        ("Infinity", false) => "POSITIVE_INFINITY",
        _ => return None,
    })
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
    pub ty: Option<Type>,
    pub binding: Option<usize>,
    pub target: Option<CallTarget>,
    /// Original source argument count, before optional padding or rest packing.
    pub supplied_count: Option<CallArgumentCount>,
    /// Specialized callback body used by the enclosing array traversal.
    pub traversal_method: Option<TraversalMethod>,
    /// Conservative checked evaluation effect; set after callable summaries stabilize.
    pub may_fail: bool,
    /// Failure of this operation itself, excluding evaluation of its children.
    pub own_may_fail: bool,
    pub(crate) effects_checked: bool,
    /// Checked receiver or constructed environment identity, distinct from its
    /// local binding index and from a reused empty storage representation.
    pub(crate) callable_origin: Option<CallableOrigin>,
    /// Stable checked invocation ID; alternatives remain outside recursive nodes.
    pub(crate) generic_invocation: Option<usize>,
    /// Slots of the calling body that a direct nested function call passes
    /// as hidden capture arguments, in the callee's hidden parameter order.
    pub(crate) capture_slots: Option<Box<[usize]>>,
    depth: usize,
}

/// A checked receiver or constructor names one environment or callable contract.
#[derive(Clone,Copy,Debug)]
pub(crate) enum CallableOrigin {Closure(usize),Storage(usize),Callback(usize)}

/// Actual source cardinality, independent of padding and packed formal slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallArgumentCount {
    Static(usize),
    Runtime { slot: usize },
}
impl From<usize> for CallArgumentCount {
    fn from(value: usize) -> Self { Self::Static(value) }
}
impl std::fmt::Display for CallArgumentCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Static(value) => write!(formatter, "{value}"),
            Self::Runtime { slot } => write!(formatter, "spread_count{slot}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NullishBranch {
    Dynamic,
    Absent,
    Present,
}

#[derive(Clone, Debug)]
pub struct SelectedSource {
    pub readonly: bool,
    pub roots: Vec<usize>,
    pub place: Option<(usize, Vec<String>)>,
    pub span: Span,
}

/// Checked spread sources stay evaluable exactly once; selected slots contain
/// only provenance and refer to native locals created by this plan.
#[derive(Clone, Debug)]
pub struct SpreadSegment {
    pub value: Box<Expr>,
    pub spread: bool,
    pub element: Type,
    pub access: crate::analysis::Mode,
    pub length: Option<usize>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpreadSelection {
    Callee,
    Receiver,
    Argument(usize),
    Rest,
}
#[derive(Clone, Debug)]
pub struct SpreadCallee {
    pub value: Box<Expr>,
    pub access: crate::analysis::Mode,
    pub selection: SpreadSelection,
}
#[derive(Clone, Debug)]
pub struct SpreadSelected {
    pub normalized_union: Option<UnionId>,
    pub literal_integer: Option<i64>,
    pub stable: bool,
    pub carrier: bool,
    pub slot: usize,
    pub selection: SpreadSelection,
    pub source: Box<SelectedSource>,
    pub access: crate::analysis::Mode,
    pub owned: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpreadBuiltinKind {
    Fixed,
    NativeStringRest,
    MathVariadic,
    ObserveVariadic,
    ConsoleVariadic,
    StringConcat,
    ArrayConcat,
    ArrayUnshift,
    ArraySplice,
    ArrayToSpliced,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpreadVariantSelection {
    Count(usize),
    FirstSegment(usize),
    Empty,
}
#[derive(Clone, Debug)]
pub struct SpreadCallVariant {
    pub selection: SpreadVariantSelection,
    pub count: usize,
    pub call: Box<Expr>,
}
#[derive(Clone, Debug)]
pub struct CheckedSpreadCall {
    /// None for a static declaration or a builtin namespace.
    pub callee: Option<SpreadCallee>,
    pub segments: Vec<SpreadSegment>,
    pub call: Box<Expr>,
    pub parameters: Vec<crate::analysis::ArgumentContract>,
    pub selected_types: Vec<Type>,
    pub selected_unions: Vec<Option<UnionId>>,
    pub variants: Vec<SpreadCallVariant>,
    pub slot: usize,
    pub minimum: usize,
    pub maximum: Option<usize>,
    /// Builtin adapters can preserve special variadic/omission policies.
    pub builtin: Option<SpreadBuiltinKind>,
}
impl CheckedSpreadCall {
    pub fn expressions(&self) -> impl Iterator<Item = &Expr> {
        self.callee.iter().map(|callee| callee.value.as_ref())
            .chain(self.segments.iter().map(|segment| segment.value.as_ref()))
            .chain(std::iter::once(self.call.as_ref()))
            .chain(self.variants.iter().map(|variant| variant.call.as_ref()))
    }
    pub fn expressions_mut(&mut self) -> impl Iterator<Item = &mut Expr> {
        self.callee.iter_mut().map(|callee| callee.value.as_mut())
            .chain(self.segments.iter_mut().map(|segment| segment.value.as_mut()))
            .chain(std::iter::once(self.call.as_mut()))
            .chain(self.variants.iter_mut().map(|variant| variant.call.as_mut()))
    }
}
impl SpreadSelection {
    pub fn native_name(self, slot: usize) -> String {
        match self {
            Self::Callee => format!("spread_callee{slot}"),
            Self::Receiver => format!("spread_receiver{slot}"),
            Self::Argument(position) => format!("spread_argument{slot}_{position}"),
            Self::Rest => format!("spread_rest{slot}"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    SpreadCall(Box<CheckedSpreadCall>),
    SpreadValue(Box<SpreadSelected>),
    /// One uninterrupted postfix chain. Parentheses close this wrapper.
    OptionalChain(Box<Expr>),
    /// Checked branch; no new runtime representation is introduced.
    Optional {
        base: Box<Expr>,
        body: Box<Expr>,
        slot: usize,
        union: Option<UnionId>,
        access: crate::analysis::Mode,
        branch: NullishBranch,
        owned: bool,
    },
    /// Checked once-selected value, with its original ownership provenance.
    Selected {
        source: Box<SelectedSource>,
        slot: usize,
        owned: bool,
        access: crate::analysis::Mode,
    },
    Conditional {
        predicate: Box<Expr>,
        consequent: Box<Expr>,
        alternate: Box<Expr>,
    },
    CompoundAssign {
        target: Box<Expr>,
        value: Box<Expr>,
        op: BinaryOp,
        operator: Span,
    },
    CoalesceAssign {
        branch: NullishBranch,
        target: Box<Expr>,
        value: Box<Expr>,
        operator: Span,
    },
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
        operator: Span,
    },
    LogicalAssign {
        target: Box<Expr>,
        value: Box<Expr>,
        and: bool,
        operator: Span,
    },
    Update {
        target: Box<Expr>,
        increment: bool,
        prefix: bool,
        operator: Span,
    },
    Unary {
        op: UnaryOp,
        operator: Span,
        value: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        operator: Span,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Closure {
        id: usize,
        params: Vec<CallableParameter>,
        result: ResultAnnotation,
        body: Vec<Stmt>,
    },
    EnumMember { enumeration: usize, member: usize },
    Name(String),
    String(String),
    // Alternating decoded literal chunks and source interpolation expressions.
    Template(Vec<Expr>),
    /// Checked-only injection or widening; source expressions remain explicit.
    Inject {
        value: Box<Expr>,
        union: UnionId,
    },
    /// Checked-only view of a proven alternative; the source slot stays tagged.
    Project {
        value: Box<Expr>,
        union: UnionId,
        narrowed: Type,
    },
    // Keep spelling until contextual typing chooses checked i64 or IEEE f64.
    Integer(String),
    Number(f64),
    Boolean(bool),
    Null,
    Undefined,
    Field(Box<Expr>, String),
    Record(Vec<(Token, Expr)>),
    Array(Vec<Expr>),
    /// Source constructor annotation is substituted by the shared generic walkers.
    ArrayOf(Vec<Expr>, Option<Box<crate::type_syntax::ValueAnnotation>>),
    SetConstructor(Vec<Expr>, Option<Box<crate::type_syntax::ValueAnnotation>>),
    MapConstructor(Vec<Expr>, Option<Box<crate::type_syntax::ValueAnnotation>>),
    NativeConstructor(Vec<Expr>, Option<Box<crate::type_syntax::ValueAnnotation>>),
    TypeAssertion { value: Box<Expr>, annotation: Box<crate::type_syntax::ValueAnnotation>, satisfies: bool },
    /// Ordered discarded prefixes followed by the selected final value.
    Sequence(Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
}

impl Expr {
    /// Placeholder initializer of `let name: Type;`: an empty span after the
    /// declarator. A written `undefined` always has a nonempty span.
    pub(crate) fn deferred(after: Span) -> Self {
        Self::new(ExprKind::Undefined, Span::new(after.end, after.end))
    }
    pub(crate) fn is_deferred(&self) -> bool {
        matches!(self.kind, ExprKind::Undefined) && self.span.start == self.span.end
    }
    pub(crate) fn checked(kind: ExprKind, span: Span, ty: Option<Type>) -> Self {
        let mut value = Self::new(kind, span);
        value.ty = ty;
        value
    }

    pub(crate) fn project(&mut self, union: UnionId, narrowed: Type, actual: Type) {
        let value = std::mem::replace(self, Self::new(ExprKind::Null, self.span));
        self.depth = value.depth;
        self.ty = Some(actual);
        self.binding = value.binding;
        self.kind = ExprKind::Project {
            value: Box::new(value),
            union,
            narrowed,
        };
    }
    pub(crate) fn wrap_record_carrier(&mut self,ty:Type){
        let value=std::mem::replace(self,Self::new(ExprKind::Null,self.span));
        self.depth=value.depth;self.ty=Some(ty);
        self.kind=ExprKind::Unary{op:UnaryOp::RecordCarrier,operator:self.span,value:Box::new(value)};
    }
    pub(crate) fn wrap_storage(&mut self, family: usize) {
        let value = std::mem::replace(self, Self::new(ExprKind::Null, self.span));
        self.depth = value.depth;
        self.ty = Some(Type::StoredCallable(family));
        self.kind = ExprKind::Unary {
            op: UnaryOp::CallableStore,
            operator: self.span,
            value: Box::new(value),
        };
    }
    pub(crate) fn wrap_once(&mut self, origin: usize) {
        let value = std::mem::replace(self, Self::new(ExprKind::Null, self.span));
        self.depth = value.depth;
        self.ty = Some(Type::Closure(origin));
        self.kind = ExprKind::Unary {
            op: UnaryOp::CallableOnce,
            operator: self.span,
            value: Box::new(value),
        };
    }

    pub(crate) fn inject(&mut self, union: UnionId) {
        let placeholder = Self::new(ExprKind::Null, self.span);
        let value = std::mem::replace(self, placeholder);
        self.depth = value.depth;
        self.ty = Some(Type::Union(union));
        self.kind = ExprKind::Inject {
            value: Box::new(value),
            union,
        };
    }

    pub(crate) fn closure_origin(&self)->Option<usize> {match self.callable_origin {Some(CallableOrigin::Closure(id))=>Some(id),_=>None}}
    pub(crate) fn storage_family(&self)->Option<usize> {match self.callable_origin {Some(CallableOrigin::Storage(id))=>Some(id),_=>None}}
    pub(crate) fn callback_contract(&self)->Option<usize> {match self.callable_origin {Some(CallableOrigin::Callback(id))=>Some(id),_=>None}}

    pub(crate) fn new(kind: ExprKind, span: Span) -> Self {
        let child_depth = match &kind {
            ExprKind::Conditional {
                predicate,
                consequent,
                alternate,
            } => predicate.depth.max(consequent.depth).max(alternate.depth),
            ExprKind::SpreadCall(plan) => plan.segments.iter().map(|segment| segment.value.depth)
                .chain(plan.callee.iter().map(|callee| callee.value.depth))
                .chain(std::iter::once(plan.call.depth)).max().unwrap_or(0),
            ExprKind::SpreadValue(_) => 0,
            ExprKind::Closure { .. } => 0,
            ExprKind::EnumMember { .. } | ExprKind::Name(_)
            | ExprKind::String(_)
            | ExprKind::Integer(_)
            | ExprKind::Number(_)
            | ExprKind::Boolean(_)
            | ExprKind::Null
            | ExprKind::Undefined
            | ExprKind::Selected { .. } => 0,
            ExprKind::TypeAssertion {value,..}
            | ExprKind::OptionalChain(value)
            | ExprKind::Unary { value, .. }
            | ExprKind::Update { target: value, .. }
            | ExprKind::Inject { value, .. }
            | ExprKind::Project { value, .. } => value.depth,
            ExprKind::Optional {
                base: left,
                body: right,
                ..
            }
            | ExprKind::Binary { left, right, .. }
            | ExprKind::CompoundAssign {
                target: left,
                value: right,
                ..
            }
            | ExprKind::Assign {
                target: left,
                value: right,
                ..
            }
            | ExprKind::CoalesceAssign {
                target: left,
                value: right,
                ..
            }
            | ExprKind::LogicalAssign {
                target: left,
                value: right,
                ..
            } => left.depth.max(right.depth),
            ExprKind::Field(base, _) => base.depth,
            ExprKind::Index(base, index) => base.depth.max(index.depth),
            ExprKind::Array(values) | ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _) | ExprKind::NativeConstructor(values, _) | ExprKind::Template(values) | ExprKind::Sequence(values) => {
                values.iter().map(|value| value.depth).max().unwrap_or(0)
            }
            ExprKind::Record(fields) => fields
                .iter()
                .map(|(_, value)| value.depth)
                .max()
                .unwrap_or(0),
            ExprKind::Call(callee, args) => args
                .iter()
                .map(|arg| arg.depth)
                .max()
                .unwrap_or(0)
                .max(callee.depth),
        };
        let transparent = matches!(
            kind,
            ExprKind::OptionalChain(_)
                | ExprKind::SpreadCall(_)
                | ExprKind::SpreadValue(_)
                | ExprKind::Selected { .. }
                | ExprKind::Unary {
                    op: UnaryOp::OptionalBoundary | UnaryOp::OptionalSelected,
                    ..
                }
        );
        let supplied_count = if let ExprKind::Call(_, arguments) = &kind { Some(CallArgumentCount::Static(arguments.len())) } else { None };
        Self {
            kind,
            span,
            ty: None,
            supplied_count,
            binding: None,
            target: None,
            traversal_method: None,
            may_fail: false,
            own_may_fail: false,
            effects_checked: false,
            callable_origin: None,
            generic_invocation: None,
            capture_slots: None,
            depth: child_depth + usize::from(!transparent),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlTarget { Loop(usize), Tail(usize), Block(usize) }

#[derive(Clone, Debug)]
pub struct Stmt {
    pub labels: Vec<Token>,
    pub control_target: Option<ControlTarget>,
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum StmtKind {
    /// Lexical declaration, consumed by local-function resolution before checking.
    LocalFunction(Box<Function>),
    Try(Box<ExceptionRegion>),
    Throw(Expr),
    Block(Vec<Stmt>),
    // For initializers are enclosed in Block; update is executed on every
    // continuing edge. A post-test loop is do...while.
    Loop {
        condition: Option<Expr>,
        body: Vec<Stmt>,
        update: Vec<Stmt>,
        post_test: bool,
        tail_reachable: bool,
    },
    ForOf {
        name: Token,
        rebindable: bool,
        iterable: Expr,
        body: Vec<Stmt>,
        binding: Option<usize>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    },
    Let {
        rebindable: bool,
        name: Token,
        annotation: Option<crate::type_syntax::ValueAnnotation>,
        value: Expr,
        binding: Option<usize>,
    },
    Assign {
        target: Expr,
        value: Expr,
        operator: Option<(BinaryOp, Span)>,
    },
    Return(Option<Expr>),
    Expr(Expr),
}

#[derive(Clone, Debug)]
pub struct ExceptionRegion {
    pub body: Vec<Stmt>,
    pub catch: Option<CatchClause>,
    pub finally: Option<Vec<Stmt>>,
}
impl ExceptionRegion {
    pub(crate) fn statements(&self)->impl DoubleEndedIterator<Item=&Stmt> {
        self.body.iter().chain(self.catch.iter().flat_map(|handler|handler.body.iter())).chain(self.finally.iter().flat_map(|body|body.iter()))
    }
    pub(crate) fn statements_mut(&mut self)->impl DoubleEndedIterator<Item=&mut Stmt> {
        self.body.iter_mut().chain(self.catch.iter_mut().flat_map(|handler|handler.body.iter_mut())).chain(self.finally.iter_mut().flat_map(|body|body.iter_mut()))
    }
}

#[derive(Clone, Debug)]
pub struct CatchClause {
    pub(crate) error_type:Option<Type>,
    pub(crate) error_contract: Option<Box<crate::error_effects::FailureSet>>,
    pub name: Option<Token>,
    pub binding: Option<usize>,
    pub body: Vec<Stmt>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnumKind { Number, String }
#[derive(Clone, Debug)]
pub enum EnumValue { Number(f64), String(String) }
#[derive(Clone, Debug)]
pub struct EnumMember { pub name: Token, pub value: EnumValue }
#[derive(Clone, Debug)]
pub struct EnumDefinition { pub kind: EnumKind, pub members: Vec<EnumMember> }

#[derive(Clone, Debug)]
pub struct SetDefinition { pub key: Type }
#[derive(Clone, Debug)]
pub struct MapDefinition { pub key: Type, pub value: Type, pub entry: Option<usize> }

#[derive(Clone, Debug)]
pub struct Record {
    /// Native declaration identity is compiler-owned, never inferred from fields or names.
    pub native_type: Option<crate::native_signatures::NativeTypeId>,
    pub set_definition: Option<SetDefinition>,
    pub map_definition: Option<MapDefinition>,
    pub enum_definition: Option<EnumDefinition>,
    pub application: Option<Box<RecordApplication>>,
    pub name: Token,
    pub type_parameters: Vec<crate::type_syntax::TypeName>,
    pub fields: Vec<RecordField>,
}

impl Record {
    /// Native collection payloads are owned storage, not source-visible fields.
    pub(crate) fn native_element(&self) -> Option<Type> {
        if !matches!(self.native_type, Some(crate::native_signatures::NativeTypeId::Deque | crate::native_signatures::NativeTypeId::Heap | crate::native_signatures::NativeTypeId::OrderedSet)) { return None; }
        self.application.as_ref()?.resolved_arguments.first().copied()
    }
    /// Positional products share the native ownership machinery with records.
    /// Source record names are identifiers, so this marker cannot collide with one.
    pub(crate) fn is_tuple(&self) -> bool { self.name.kind == TokenKind::Symbol && self.name.text.starts_with('[') }
}

#[derive(Clone, Debug)]
pub struct RecordApplication {
    /// Checked argument identities retained independently of their display spelling.
    pub resolved_arguments: Vec<Type>,
    pub declaration: Token,
    pub arguments: Vec<crate::type_syntax::ValueAnnotation>,
}

#[derive(Clone, Debug)]
pub struct RecordField {
    pub name: Token,
    pub annotation: crate::type_syntax::ValueAnnotation,
    /// Filled after all nominal record names have been collected.
    pub ty: Option<Type>,
    /// Source callable declaration, distinct from its canonical storage family.
    pub callable_declaration: Option<usize>,
    /// Declared as `name?: Type`: a literal may omit it; its type includes `undefined`.
    pub optional: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CallbackKind {
    /// Owned reference-counted environment with readonly reusable invocation.
    SharedRead,
    Read,
    Reusable,
    Once,
    Owned,
    OwnedOnce,
}

impl CallbackKind {
    /// Reusable owned parameters may lend their environment for a scoped call.
    pub fn can_pass_to(self, target: Self) -> bool {
        self == target || (self == Self::OwnedOnce && target == Self::Once) || (self == Self::Owned && target == Self::Reusable)
            || (self == Self::SharedRead && matches!(target, Self::Read | Self::Reusable | Self::Owned))
    }
    pub fn invocation(self) -> crate::Mode {
        if matches!(self, Self::Read | Self::SharedRead) {
            crate::Mode::Read
        } else if self.is_once() {
            crate::Mode::Consume
        } else {
            crate::Mode::Mutate
        }
    }

    pub fn is_owned(self) -> bool {
        matches!(self, Self::Owned | Self::OwnedOnce | Self::SharedRead)
    }
    pub fn is_once(self) -> bool {
        matches!(self, Self::Once | Self::OwnedOnce)
    }
}

#[derive(Clone, Debug)]
pub enum ParameterInitializer {
    Required,
    Optional(Span),
    Default { value: Box<Expr>, span: Span },
}
impl ParameterInitializer {
    pub fn omission(&self) -> crate::ParameterOmission {
        match self {
            Self::Required => crate::ParameterOmission::Required,
            Self::Optional(_) => crate::ParameterOmission::Optional,
            Self::Default { .. } => crate::ParameterOmission::Defaulted,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CallableParameter {
    pub rest: bool,
    pub name: Token,
    pub ty: ParameterAnnotation,
    pub access: Option<Token>,
    pub initializer: ParameterInitializer,
}

#[derive(Clone, Debug)]
pub struct CallbackAnnotation {
    pub kind: CallbackKind,
    pub params: Vec<CallableParameter>,
    pub result: Box<ResultAnnotation>,
    pub span: Span,
}

/// Recursive source type syntax keeps parameter ownership distinct from results.
#[derive(Clone, Debug)]
pub enum ParameterAnnotation {
    Named(Box<crate::type_syntax::ValueAnnotation>),
    Callback(Box<CallbackAnnotation>),
    Inferred(Token),
}
impl ParameterAnnotation {
    pub fn span(&self) -> Span {
        match self {
            Self::Named(token) => token.span,
            Self::Callback(value) => value.span,
            Self::Inferred(token) => token.span,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ResultAnnotation {
    Inferred(Span),
    Named(Box<crate::type_syntax::ValueAnnotation>),
    Callable {
        kind: Option<CallbackKind>,
        params: Vec<CallableParameter>,
        result: Box<ResultAnnotation>,
        span: Span,
    },
}

fn structured_parameters(
    params: &[CallableParameter],
) -> Vec<crate::type_syntax::CallableParameter> {
    params
        .iter()
        .map(|param| {
            let annotation = match &param.ty {
                ParameterAnnotation::Named(value) => (**value).clone(),
                ParameterAnnotation::Callback(callback) => callback.value_annotation(),
                ParameterAnnotation::Inferred(token) => crate::type_syntax::ValueAnnotation { kind: crate::type_syntax::ValueAnnotationKind::Named(crate::type_syntax::TypeName { text: "__inferred_parameter".into(), span: token.span }), span: token.span },
            };
            crate::type_syntax::CallableParameter {
                name: crate::type_syntax::TypeName {
                    text: param.name.text.clone(),
                    span: param.name.span,
                },
                omission: param.initializer.omission(),
                rest: param.rest,
                access: param
                    .access
                    .as_ref()
                    .map(|access| crate::type_syntax::ParameterAccess {
                        mode: match access.text.as_str() {
                            "read" => crate::Mode::Read,
                            "mutate" => crate::Mode::Mutate,
                            "consume" => crate::Mode::Consume,
                            _ => unreachable!("parsed parameter access"),
                        },
                        span: access.span,
                    }),
                span: Span::new(param.name.span.start, annotation.span.end),
                annotation,
            }
        })
        .collect()
}

impl ResultAnnotation {
    pub(crate) fn value_annotation(&self) -> crate::type_syntax::ValueAnnotation {
        match self {
            Self::Inferred(span) => crate::type_syntax::ValueAnnotation { kind: crate::type_syntax::ValueAnnotationKind::Named(crate::type_syntax::TypeName { text: "__inferred_arrow_result".into(), span: *span }), span: *span },
            Self::Named(value) => (**value).clone(),
            Self::Callable {
                kind,
                params,
                result,
                span,
            } => crate::type_syntax::ValueAnnotation {
                kind: crate::type_syntax::ValueAnnotationKind::Callable {
                    ownership: match kind {
                        Some(CallbackKind::OwnedOnce) => {
                            crate::type_syntax::CallableOwnership::OwnedOnce
                        }
                        Some(CallbackKind::Owned) => crate::type_syntax::CallableOwnership::Owned,
                        Some(CallbackKind::SharedRead) => crate::type_syntax::CallableOwnership::SharedRead,
                        _ => crate::type_syntax::CallableOwnership::Default,
                    },
                    parameters: structured_parameters(params),
                    result: Box::new(result.value_annotation()),
                },
                span: *span,
            },
        }
    }
}

impl CallbackAnnotation {
    pub(crate) fn value_annotation(&self) -> crate::type_syntax::ValueAnnotation {
        use crate::type_syntax::{CallableOwnership, ValueAnnotation, ValueAnnotationKind};
        ValueAnnotation {
            kind: ValueAnnotationKind::Callable {
                ownership: match self.kind {
                    CallbackKind::Read => CallableOwnership::Read,
                    CallbackKind::Reusable => CallableOwnership::Reusable,
                    CallbackKind::Once => CallableOwnership::Once,
                    CallbackKind::Owned => CallableOwnership::Owned,
                    CallbackKind::OwnedOnce => CallableOwnership::OwnedOnce,
                    CallbackKind::SharedRead => CallableOwnership::SharedRead,
                },
                parameters: structured_parameters(&self.params),
                result: Box::new(self.result.value_annotation()),
            },
            span: self.span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub application: Option<Box<FunctionApplication>>,
    pub scope_span: Span,
    pub name: Token,
    pub type_parameters: Vec<crate::type_syntax::TypeName>,
    pub params: Vec<(
        Token,
        crate::type_syntax::ValueAnnotation,
        Option<CallbackAnnotation>,
    )>,
    pub parameter_initializers: Vec<ParameterInitializer>,
    /// Access stated with ReadOnly, Mutable or Owned; checked against the body.
    pub parameter_access: Vec<Option<(crate::Mode, Span)>>,
    pub rest_parameter: bool,
    pub result: Option<ResultAnnotation>,
    pub body: Vec<Stmt>,
}

#[derive(Clone, Debug)]
pub struct FunctionApplication {
    /// Concrete lexical owner, separate from the explicit type application.
    pub lexical_owner: Option<String>,
    pub declaration: Token,
    pub variables: Vec<crate::type_syntax::TypeName>,
    pub arguments: Vec<crate::type_syntax::ValueAnnotation>,
    pub patterns: Vec<crate::type_syntax::ValueAnnotation>,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub(crate) unresolved_local_function: Option<Token>,
    pub(crate) local_declarations: Vec<crate::local_functions::Declaration>,
    pub(crate) local_capture_plans: Vec<crate::local_capture_plan::Plan>,
    pub type_argument_checks: Vec<crate::type_syntax::ValueAnnotation>,
    pub call_type_arguments: Vec<(Span, Vec<crate::type_syntax::ValueAnnotation>)>,
    pub records: Vec<Record>,
    pub functions: Vec<Function>,
    /// Expanded `type Name = Type;` declarations of the file. Uses are
    /// already expanded; a project keeps these for importing modules.
    pub(crate) type_aliases: crate::type_aliases::Aliases,
    /// `extends` names that no declaration or import of the file provides.
    /// A standalone parse of a module keeps them; checking reports them.
    pub(crate) unresolved_bases: Vec<Token>,
}

pub fn parse(source: &str) -> Result<Program, Diagnostic> {
    parse_entry(source, false, Default::default())
}

/// Diagnostic-only parsing. A recovered tree never leaves this function.
pub(crate) fn diagnostics(source: &str) -> Vec<Diagnostic> {
    let errors = std::cell::RefCell::new(Vec::new());
    let result = Parser::new(source, Default::default(), Default::default()).and_then(|mut parser| {
        parser.diagnostics = Some(&errors);
        parser.program(false).map(|_| ())
    });
    if let Err(error) = result { errors.borrow_mut().push(error); }
    let mut errors = errors.into_inner();
    errors.sort_by_key(|error| (error.span.start, error.span.end));
    errors.dedup_by(|a,b| a.span == b.span && a.code == b.code && a.message == b.message);
    errors
}

pub(crate) fn parse_test_entry(source: &str) -> Result<Program, Diagnostic> {
    parse_entry(source, true, Default::default())
}

/// Parses a module whose imports include type aliases of other modules.
pub(crate) fn parse_module(source: &str, test_entry: bool, imported: crate::type_aliases::Aliases, records: crate::type_aliases::ImportedRecords) -> Result<Program, Diagnostic> {
    parse_entry_with(source, test_entry, imported, records)
}

fn parse_entry(source: &str, test_entry: bool, imported: crate::type_aliases::Aliases) -> Result<Program, Diagnostic> {
    parse_entry_with(source, test_entry, imported, Default::default())
}

fn parse_entry_with(source: &str, test_entry: bool, imported: crate::type_aliases::Aliases, records: crate::type_aliases::ImportedRecords) -> Result<Program, Diagnostic> {
    let mut program = parse_raw(source, test_entry, imported, records)?;
    crate::local_functions::resolve(&mut program)?;
    Ok(program)
}

fn parse_raw(source: &str, test_entry: bool, imported: crate::type_aliases::Aliases, records: crate::type_aliases::ImportedRecords) -> Result<Program, Diagnostic> {
    Parser::new(source, imported, records)?.program(test_entry)
}

impl<'a> Parser<'a> {
    fn new(source: &'a str, imported: crate::type_aliases::Aliases, records: crate::type_aliases::ImportedRecords) -> Result<Self, Diagnostic> {
        Ok(Self {
            source,
            diagnostics: None,
            tokens: lex(source)?,
            i: 0,
            depth: 0,
            next_closure: 0,
            unresolved_local_function: None,
            call_type_arguments: Vec::new(),
            pending: Vec::new(),
            next_pattern: 0,
            parameter_patterns: Vec::new(),
            aliases: imported,
            local_aliases: Vec::new(),
            imported_records: records,
        })
    }
}

// Registration statements execute in a synthetic function, but retain source-level
// module rules. Do not descend into expression callbacks or helper functions.
fn registration_return(body: &[Stmt]) -> Option<Span> {
    for statement in body {
        let found = match &statement.kind {
            StmtKind::Return(_) => Some(statement.span),
            StmtKind::Block(body) | StmtKind::While { body, .. } | StmtKind::ForOf { body, .. } => registration_return(body),
            StmtKind::Loop { body, update, .. } => registration_return(body).or_else(|| registration_return(update)),
            StmtKind::If { then_body, else_body, .. } => registration_return(then_body).or_else(|| registration_return(else_body)),
            _ => None,
        };
        if found.is_some() { return found; }
    }
    None
}

struct Parser<'a> {
    diagnostics: Option<&'a std::cell::RefCell<Vec<Diagnostic>>>,
    unresolved_local_function: Option<Token>,
    source: &'a str,
    tokens: Vec<Token>,
    i: usize,
    depth: usize,
    next_closure: usize,
    call_type_arguments: Vec<(Span, Vec<crate::type_syntax::ValueAnnotation>)>,
    /// Position bindings of a destructuring declaration; they follow the
    /// declaration that holds the whole value, in the same block.
    pending: Vec<Stmt>,
    next_pattern: usize,
    /// Position bindings of destructured parameters; they open the body.
    parameter_patterns: Vec<Stmt>,
    /// `type Name = Type;` declarations of the file, fully expanded.
    aliases: std::collections::HashMap<String, crate::type_syntax::ValueAnnotation>,
    /// Names of the aliases this file declares, as opposed to imports.
    local_aliases: Vec<String>,
    /// Fields of imported record types, for `interface ... extends`.
    imported_records: crate::type_aliases::ImportedRecords,
}

impl Parser<'_> {
    fn peek(&self) -> &Token {
        &self.tokens[self.i]
    }
    fn is(&self, text: &str) -> bool {
        !matches!(
            self.peek().kind,
            TokenKind::String
                | TokenKind::TemplateHead
                | TokenKind::TemplateMiddle
                | TokenKind::TemplateTail
        ) && self.peek().text == text
    }
    fn take(&mut self) -> Token {
        let token = self.peek().clone();
        if token.kind != TokenKind::End {
            self.i += 1;
        }
        token
    }
    fn accept(&mut self, text: &str) -> bool {
        if self.is(text) {
            self.take();
            true
        } else {
            false
        }
    }
    fn expect(&mut self, text: &str) -> Result<Token, Diagnostic> {
        if !self.is(text) {
            let error = Diagnostic::new("syntax", format!("Expected {text:?}."), self.peek().span);
            // Insert before whitespace/comments following the last accepted token.
            // The editor checks the complete edited source before offering this.
            return Err(if matches!(text, ";" | ")" | "]" | "}") && self.i > 0 {
                let offset = self.tokens[self.i - 1].span.end;
                error.fix(
                    format!("Insert missing {text}"),
                    Span::new(offset, offset),
                    text,
                )
            } else {
                error
            });
        }
        Ok(self.take())
    }
    fn name(&mut self) -> Result<Token, Diagnostic> {
        if self.peek().kind != TokenKind::Name
            || matches!(self.peek().text.as_str(), "true" | "false")
        {
            return Err(Diagnostic::new(
                "syntax",
                "Expected a name.",
                self.peek().span,
            ));
        }
        Ok(self.take())
    }
    fn value_annotation(&mut self) -> Result<crate::type_syntax::ValueAnnotation, Diagnostic> {
        let (mut annotation, index) =
            crate::type_syntax::parse_prefix(&mut self.tokens, self.i, self.depth)?;
        self.i = index;
        if !self.aliases.is_empty() {
            crate::type_aliases::expand(&mut annotation, &self.aliases);
        }
        Ok(annotation)
    }

    /// Reads every top-level `type Name = Type;` whose right side is not a
    /// record body, before the declarations that use the names are parsed.
    #[inline(never)]
    fn collect_type_aliases(&mut self) -> Result<(), Diagnostic> {
        let mut raw = Vec::new();
        let mut depth = 0usize;
        let mut index = 0;
        while index + 3 < self.tokens.len() {
            let token = &self.tokens[index];
            if token.kind == TokenKind::Symbol {
                match token.text.as_str() {
                    "{" => depth += 1,
                    "}" => depth = depth.saturating_sub(1),
                    _ => {}
                }
            }
            let declaration = depth == 0
                && token.kind == TokenKind::Name
                && token.text == "type"
                && self.tokens[index + 1].kind == TokenKind::Name
                && (index == 0 || matches!(self.tokens[index - 1].text.as_str(), ";" | "}"));
            if !declaration {
                index += 1;
                continue;
            }
            let name = self.tokens[index + 1].clone();
            let after = &self.tokens[index + 2];
            if after.text == "<" {
                // A generic declaration is a record unless its right side is not a body.
                let mut close = index + 3;
                while close < self.tokens.len() && self.tokens[close].text != "=" && self.tokens[close].text != ";" {
                    close += 1;
                }
                if self.tokens.get(close).is_some_and(|token| token.text == "=")
                    && self.tokens.get(close + 1).is_some_and(|token| token.text != "{")
                {
                    return Err(Diagnostic::new(
                        "unsupported",
                        "A type alias cannot have type parameters. Only record types can: `type Box<T> = { value: T; };`.",
                        name.span,
                    ));
                }
                index += 1;
                continue;
            }
            if after.text != "=" || self.tokens[index + 3].text == "{" {
                index += 1;
                continue;
            }
            if matches!(
                name.text.as_str(),
                "int" | "number" | "boolean" | "string" | "null" | "undefined" | "void" | "Function" | "Mutable" | "ReadOnly" | "Once" | "Owned" | "Set" | "Map" | "Array"
            ) {
                return Err(Diagnostic::new(
                    "duplicate",
                    format!("Type name {:?} is built in and cannot be declared.", name.text),
                    name.span,
                ));
            }
            let (annotation, end) = crate::type_syntax::parse_prefix(&mut self.tokens, index + 3, self.depth)?;
            raw.push((name, annotation));
            index = end;
        }
        let imported = std::mem::take(&mut self.aliases);
        let (aliases, local) = crate::type_aliases::resolve(raw, imported)?;
        self.aliases = aliases;
        self.local_aliases = local;
        Ok(())
    }

    /// `interface Name<T> extends Base { field: Type; }` after the keyword.
    /// Fields end with `;` or `,`; the last separator is optional.
    #[inline(never)]
    fn interface_declaration(&mut self) -> Result<(Record, Vec<Token>), Diagnostic> {
        let name = self.name()?;
        if self.aliases.contains_key(&name.text) {
            return Err(Diagnostic::new(
                "duplicate",
                format!("Type name {:?} is already declared.", name.text),
                name.span,
            ));
        }
        if matches!(name.text.as_str(), "Function" | "Mutable" | "ReadOnly" | "Once" | "Owned") {
            return Err(Diagnostic::new(
                "duplicate",
                format!("Type name {:?} is reserved for built-in function and access types.", name.text),
                name.span,
            ));
        }
        let type_parameters = self.type_parameters()?;
        let mut bases = Vec::new();
        if self.accept("extends") {
            loop {
                let base = self.name()?;
                if self.is("<") {
                    return Err(Diagnostic::new(
                        "unsupported",
                        "An interface cannot extend a generic type with arguments. Declare the fields in this interface.",
                        base.span,
                    ));
                }
                bases.push(base);
                if !self.accept(",") {
                    break;
                }
            }
        }
        self.expect("{")?;
        let mut fields = Vec::new();
        while !self.accept("}") {
            let field = self.name()?;
            if self.is("(") || self.is("<") {
                return Err(Diagnostic::new(
                    "unsupported",
                    format!(
                        "An interface cannot declare a method. Declare a function-typed field, for example `{}: Function<int, [int]>;`.",
                        field.text
                    ),
                    field.span,
                ));
            }
            let optional = self.accept("?");
            self.expect(":")?;
            let annotation = self.value_annotation()?;
            let annotation = if optional { Self::optional_field_type(annotation) } else { annotation };
            fields.push(RecordField { name: field, annotation, ty: None, callable_declaration: None, optional });
            if !self.accept(";") && !self.accept(",") && !self.is("}") {
                self.expect(";")?;
            }
        }
        self.accept(";");
        Ok((
            Record {
                native_type: None, map_definition: None,
                set_definition: None,
                enum_definition: None,
                name,
                type_parameters,
                fields,
                application: None,
            },
            bases,
        ))
    }

    /// Puts the fields of each `extends` base in front of an interface's own
    /// fields, bases in written order. Bases may be declared later.
    #[inline(never)]
    fn inherit_interface_fields(records: &mut [Record], mut pending: Vec<(Token, Vec<Token>)>, imported: &crate::type_aliases::ImportedRecords) -> Result<Vec<Token>, Diagnostic> {
        let mut unresolved = Vec::new();
        while !pending.is_empty() {
            let waiting = pending.iter().map(|(name, _)| name.text.clone()).collect::<Vec<_>>();
            let ready = pending
                .iter()
                .position(|(_, bases)| bases.iter().all(|base| !waiting.contains(&base.text)));
            let Some(ready) = ready else {
                let (name, _) = &pending[0];
                return Err(Diagnostic::new(
                    "recursive-interface",
                    format!("Interface {:?} extends itself.", name.text),
                    name.span,
                ));
            };
            let (name, bases) = pending.remove(ready);
            let mut inherited: Vec<RecordField> = Vec::new();
            for base in &bases {
                if let Some(fields) = imported.get(&base.text) {
                    // Spans of another file mean nothing here; use the base name.
                    inherited.extend(fields.iter().cloned().map(|mut field| {
                        field.name.span = base.span;
                        crate::type_aliases::respan(&mut field.annotation, base.span);
                        field
                    }));
                    continue;
                }
                let Some(found) = records.iter().find(|record| {
                    record.name.text == base.text
                        && record.enum_definition.is_none()
                        && record.map_definition.is_none()
                        && record.set_definition.is_none()
                }) else {
                    unresolved.push(base.clone());
                    continue;
                };
                if !found.type_parameters.is_empty() {
                    return Err(Diagnostic::new(
                        "unsupported",
                        "An interface cannot extend a generic type. Declare the fields in this interface.",
                        base.span,
                    ));
                }
                inherited.extend(found.fields.iter().cloned());
            }
            let record = records
                .iter_mut()
                .find(|record| record.name.span == name.span)
                .expect("parsed interface");
            inherited.append(&mut record.fields);
            for (index, field) in inherited.iter().enumerate() {
                if inherited[..index].iter().any(|earlier| earlier.name.text == field.name.text) {
                    return Err(Diagnostic::new(
                        "duplicate",
                        format!("Field {:?} is already declared by this interface or a type it extends.", field.name.text),
                        if field.name.span.start >= name.span.start { field.name.span } else { name.span },
                    ));
                }
            }
            record.fields = inherited;
        }
        Ok(unresolved)
    }

    /// The type of `name?: Type`: the written type or `undefined`.
    #[inline(never)]
    fn optional_field_type(annotation: crate::type_syntax::ValueAnnotation) -> crate::type_syntax::ValueAnnotation {
        use crate::type_syntax::{TypeName, ValueAnnotation, ValueAnnotationKind as Kind};
        let span = annotation.span;
        let absent = ValueAnnotation { kind: Kind::Named(TypeName { text: "undefined".into(), span }), span };
        let members = match annotation.kind {
            Kind::Union(mut members) => {
                if !members.iter().any(|member| matches!(&member.kind, Kind::Named(name) if name.text == "undefined")) {
                    members.push(absent);
                }
                members
            }
            kind => vec![ValueAnnotation { kind: Kind::Group(Box::new(ValueAnnotation { kind, span })), span }, absent],
        };
        ValueAnnotation { kind: Kind::Union(members), span }
    }

    fn type_parameters(&mut self) -> Result<Vec<crate::type_syntax::TypeName>, Diagnostic> {
        let mut parameters: Vec<crate::type_syntax::TypeName> = Vec::new();
        if !self.accept("<") {
            return Ok(parameters);
        }
        loop {
            let token = self.name()?;
            if matches!(
                token.text.as_str(),
                "int" | "number" | "boolean" | "string" | "null" | "undefined" | "void"
            ) {
                return Err(Diagnostic::new(
                    "type-parameter",
                    "A type parameter cannot replace a primitive type name.",
                    token.span,
                ));
            }
            if let Some(previous) = parameters
                .iter()
                .find(|parameter| parameter.text == token.text)
            {
                return Err(Diagnostic::new(
                    "duplicate",
                    format!("Type parameter {:?} is already declared.", token.text),
                    token.span,
                )
                .related(previous.span, "Previous type parameter here."));
            }
            if parameters.len() >= 128 {
                return Err(Diagnostic::new(
                    "unsupported",
                    "A declaration supports at most 128 type parameters.",
                    token.span,
                ));
            }
            parameters.push(crate::type_syntax::TypeName {
                text: token.text,
                span: token.span,
            });
            if !self.accept(",") {
                break;
            }
        }
        crate::type_syntax::split_closer(&mut self.tokens, self.i);
        self.expect(">")?;
        Ok(parameters)
    }

    fn program(mut self, test_entry: bool) -> Result<Program, Diagnostic> {
        let mut registration = Vec::new();
        let mut program = Program {
            unresolved_local_function: None,
            local_declarations: Vec::new(),
            local_capture_plans: Vec::new(),
            type_argument_checks: Vec::new(),
            type_aliases: Default::default(),
            unresolved_bases: Vec::new(),
            call_type_arguments: Vec::new(),
            records: Vec::new(),
            functions: Vec::new(),
        };
        self.collect_type_aliases()?;
        let mut interface_bases: Vec<(Token, Vec<Token>)> = Vec::new();
        while self.peek().kind != TokenKind::End {
            if self.accept("enum") {
                program.records.push(self.enum_declaration(&program.records)?);
            } else if self.is("type")
                && self.tokens.get(self.i + 1).is_some_and(|token| self.aliases.contains_key(&token.text))
                && self.tokens.get(self.i + 2).is_some_and(|token| token.text == "=")
                && self.tokens.get(self.i + 3).is_some_and(|token| token.text != "{")
            {
                // Already collected; skip the declaration.
                self.i += 3;
                let name = self.tokens[self.i - 2].clone();
                if program.records.iter().any(|record| record.name.text == name.text) {
                    return Err(Diagnostic::new(
                        "duplicate",
                        format!("Type name {:?} is already declared.", name.text),
                        name.span,
                    ));
                }
                let (_, end) = crate::type_syntax::parse_prefix(&mut self.tokens, self.i, self.depth)?;
                self.i = end;
                self.expect(";")?;
            } else if self.accept("type") {
                let name = self.name()?;
                if self.aliases.contains_key(&name.text) {
                    return Err(Diagnostic::new(
                        "duplicate",
                        format!("Type name {:?} is already declared.", name.text),
                        name.span,
                    ));
                }
                if matches!(name.text.as_str(), "Function" | "Mutable" | "ReadOnly" | "Once" | "Owned") {
                    return Err(Diagnostic::new(
                        "duplicate",
                        format!("Type name {:?} is reserved for built-in function and access types.", name.text),
                        name.span,
                    ));
                }
                let type_parameters = self.type_parameters()?;
                self.expect("=")?;
                self.expect("{")?;
                let mut fields = Vec::new();
                while !self.accept("}") {
                    let field = self.name()?;
                    let optional = self.accept("?");
                    self.expect(":")?;
                    let annotation = self.value_annotation()?;
                    let annotation = if optional { Self::optional_field_type(annotation) } else { annotation };
                    fields.push(RecordField {
                        name: field,
                        annotation,
                        ty: None,
                        callable_declaration: None,
                        optional,
                    });
                    self.expect(";")?;
                }
                self.expect(";")?;
                program.records.push(Record {
                    native_type: None, map_definition: None, set_definition: None,                    enum_definition: None,
                    name,
                    type_parameters,
                    fields,
                    application: None,
                });
            } else if self.accept("interface") {
                let (record, bases) = self.interface_declaration()?;
                if !bases.is_empty() {
                    interface_bases.push((record.name.clone(), bases));
                }
                program.records.push(record);
            } else if self.accept("function") {
                program.functions.push(self.named_function()?);
            } else if test_entry {
                registration.push(self.block_statement()?);
                registration.append(&mut self.pending);
            } else {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Expected a type or function declaration.",
                    self.peek().span,
                ));
            }
        }
        if test_entry {
            if let Some(span) = registration_return(&registration) {
                return Err(Diagnostic::new("test-entry", "return is only valid inside a function or callback.", span));
            }
            if let Some(main) = program.functions.iter().find(|function| function.name.text == "main") {
                return Err(Diagnostic::new("test-entry", "Test entry files use top-level registration and must not declare main.", main.name.span));
            }
            let span = Span::default();
            registration.push(Stmt { labels: Vec::new(), control_target: None, span, kind: StmtKind::Expr(Expr::new(
                ExprKind::Call(Box::new(Expr::new(ExprKind::Name("@native:std:testing-runtime#finishTests".into()), span)), Vec::new()), span
            )) });
            program.functions.push(Function {
                application: None, scope_span: Span::new(0, self.source.len()),
                name: Token { text: "main".into(), kind: TokenKind::Name, span },
                type_parameters: Vec::new(), params: Vec::new(), parameter_initializers: Vec::new(),
                parameter_access: Vec::new(), rest_parameter: false, result: None, body: registration,
            });
        }
        program.unresolved_local_function = self.unresolved_local_function;
        program.call_type_arguments = self.call_type_arguments;
        program.unresolved_bases = Self::inherit_interface_fields(&mut program.records, interface_bases, &self.imported_records)?;
        program.type_aliases = self
            .local_aliases
            .iter()
            .map(|name| (name.clone(), self.aliases[name].clone()))
            .collect();
        Ok(program)
    }
    fn result_annotation(&mut self) -> Result<ResultAnnotation, Diagnostic> {
        Ok(crate::callable_annotations::result_annotation(
            &self.value_annotation()?,
        ))
    }

    // The opening parenthesis is already consumed. Types are checked later.
    fn callable_parameters(&mut self, allow_defaults: bool) -> Result<Vec<CallableParameter>, Diagnostic> {
        let mut params = Vec::new();
        if !self.is(")") {
            loop {
                let rest = self.accept("...");
                let pattern = allow_defaults && matches!(self.peek().text.as_str(), "[" | "{");
                let name = if pattern { self.parameter_pattern()? } else { self.name()? };
                let optional = if self.accept("?") { Some(self.tokens[self.i-1].span) } else { None };
                let annotated = self.accept(":");
                if !annotated && !allow_defaults {
                    return Err(Diagnostic::new("syntax", "Callable signature parameters require a type annotation.", name.span));
                }
                let (ty, mut access) = if !annotated {
                    (ParameterAnnotation::Inferred(name.clone()), None)
                } else if self.access_wrapper_start() {
                    self.wrapped_parameter_annotation()?
                } else {
                    (self.parameter_annotation()?, None)
                };
                if pattern && access.is_none() {
                    access = Some(Self::pattern_access(&name));
                }
                let initializer = self.parameter_initializer(optional, allow_defaults)?;
                Self::rest_initializer(rest, &initializer, name.span)?;
                params.push(CallableParameter { rest, name, ty, access, initializer });
                if rest && self.is(",") { return Err(Diagnostic::new("rest-parameter", "A rest parameter must be last.", self.peek().span)); }
                if !self.accept(",") {
                    break;
                }
            }
        }
        self.expect(")")?;
        Self::parameter_order(&params.iter().filter(|p|!p.rest).map(|p|p.initializer.clone()).collect::<Vec<_>>(), params.iter().map(|p|p.name.span))?;
        Ok(params)
    }

    /// A destructuring pattern in a parameter position, held under a hidden name.
    #[inline(never)]
    fn parameter_pattern(&mut self) -> Result<Token, Diagnostic> {
        let mut whole = self.pattern_holder();
        let mut positions = Vec::new();
        self.binding_pattern(&whole, false, &mut positions, 0)?;
        // Retain the parameter pattern boundary for initialization scheduling.
        whole.span.end = self.tokens[self.i - 1].span.end;
        self.parameter_patterns.append(&mut positions);
        Ok(whole)
    }

    /// A destructured parameter without a stated access is read: its names
    /// borrow the argument's positions. `Owned<T>` lets them move out.
    /// Put destructured parameter names first in a body.
    #[inline(never)]
    fn open_with_patterns(mut patterns: Vec<Stmt>, body: &mut Vec<Stmt>) {
        if !patterns.is_empty() {
            patterns.append(body);
            *body = patterns;
        }
    }

    #[inline(never)]
    fn pattern_access(name: &Token) -> Token {
        Token { text: "read".to_owned(), span: name.span, kind: TokenKind::Name }
    }

    fn access_wrapper_start(&self) -> bool {
        matches!(self.peek().text.as_str(), "ReadOnly" | "Mutable" | "Owned" | "Once" | "Function")
            && self.tokens.get(self.i + 1).is_some_and(|token| token.text == "<")
    }

    // `values: Mutable<int[]>` states data access through the type; a wrapped
    // Function stays an ordinary callable annotation.
    fn wrapped_parameter_annotation(&mut self) -> Result<(ParameterAnnotation, Option<Token>), Diagnostic> {
        if self.depth >= 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Type annotation nesting exceeds the prototype limit of 128.",
                self.peek().span,
            ));
        }
        let (access, mut annotation, next) =
            crate::type_syntax::parse_parameter_prefix(&mut self.tokens, self.i, self.depth)?;
        self.i = next;
        crate::type_aliases::expand(&mut annotation, &self.aliases);
        let access = access.map(|access| Token {
            text: access.mode.as_str().to_owned(),
            span: access.span,
            kind: TokenKind::Name,
        });
        Ok((
            match crate::callable_annotations::callback(&annotation) {
                Some(callback) => ParameterAnnotation::Callback(Box::new(callback)),
                None => ParameterAnnotation::Named(Box::new(annotation)),
            },
            access,
        ))
    }

    fn rest_initializer(rest: bool, initializer: &ParameterInitializer, span: Span) -> Result<(), Diagnostic> {
        if rest && !matches!(initializer, ParameterInitializer::Required) {
            return Err(Diagnostic::new("rest-parameter", "A rest parameter cannot be optional or have a default initializer.", span));
        }
        Ok(())
    }

    fn parameter_initializer(&mut self, optional: Option<Span>, allow_default: bool) -> Result<ParameterInitializer, Diagnostic> {
        if self.accept("=") {
            let operator = self.tokens[self.i-1].span;
            if optional.is_some() {
                return Err(Diagnostic::new("parameter-initializer", "An optional parameter cannot also have a default initializer.", operator));
            }
            if !allow_default {
                return Err(Diagnostic::new("parameter-initializer", "Executable defaults belong to declarations and arrows, not callable type annotations.", operator));
            }
            let declared = self.pending.len();
            let value = self.expression()?;
            // A default runs before the body block exists, so there is no block
            // to hold the private declaration of a named function expression.
            if let Some(statement) = self.pending[declared..].iter().find(|statement| matches!(statement.kind, StmtKind::LocalFunction(_))) {
                return Err(Diagnostic::new("unsupported", "A named function expression cannot be a parameter default yet. Use an arrow function, or declare the function and write its name here.", statement.span));
            }
            return Ok(ParameterInitializer::Default { span: Span::new(operator.start, value.span.end), value: Box::new(value) });
        }
        Ok(optional.map_or(ParameterInitializer::Required, ParameterInitializer::Optional))
    }

    fn parameter_order(initializers: &[ParameterInitializer], names: impl Iterator<Item=Span>) -> Result<(), Diagnostic> {
        let mut optional = false;
        for (initializer, span) in initializers.iter().zip(names) {
            if optional && matches!(initializer, ParameterInitializer::Required) {
                return Err(Diagnostic::new("parameter-order", "A required parameter cannot follow an optional parameter.", span));
            }
            optional |= matches!(initializer, ParameterInitializer::Optional(_));
        }
        Ok(())
    }

    // All recursive parameter/result annotations share the original128 guard.
    // Keeping this parser frame separate avoids widening general expression parsing.
    fn parameter_annotation(&mut self) -> Result<ParameterAnnotation, Diagnostic> {
        let annotation = self.value_annotation()?;
        Ok(match crate::callable_annotations::callback(&annotation) {
            Some(callback) => ParameterAnnotation::Callback(Box::new(callback)),
            None => ParameterAnnotation::Named(Box::new(annotation)),
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        if self.depth >= 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Block nesting exceeds the prototype limit of 128.",
                self.peek().span,
            ));
        }
        self.expect("{")?;
        self.depth += 1;
        let mut body = Vec::new();
        while !self.accept("}") {
            if self.peek().kind == TokenKind::End {
                self.expect("}")?;
            }
            let start = self.i;
            let depth = self.depth;
            let pending = self.pending.len();
            let arguments = self.call_type_arguments.len();
            let patterns = self.parameter_patterns.len();
            match self.block_statement() {
                Ok(statement) => {
                    body.push(statement);
                    body.append(&mut self.pending);
                }
                Err(error) => {
                    let Some(errors) = self.diagnostics else { return Err(error); };
                    if error.code != "syntax" || errors.borrow().len() >= 63 { return Err(error); }
                    errors.borrow_mut().push(error);
                    self.depth = depth;
                    self.pending.truncate(pending);
                    self.call_type_arguments.truncate(arguments);
                    self.parameter_patterns.truncate(patterns);
                    self.recover_statement(start);
                }
            }
        }
        self.depth -= 1;
        Ok(body)
    }
    // Scan token kinds, so delimiter-looking strings cannot change recovery.
    // Braces preserve the containing block; a semicolon also bounds malformed
    // parentheses. A fresh declaration can follow a missing semicolon.
    fn recover_statement(&mut self, start: usize) {
        let failed = self.i;
        let mut braces = 0usize;
        self.i = start;
        while self.peek().kind != TokenKind::End {
            if self.i > start && self.i >= failed && braces == 0
                && matches!(self.peek().text.as_str(), "const" | "let" | "return" | "throw" | "if" | "while" | "for")
                && self.peek().kind == TokenKind::Name {
                return;
            }
            if self.peek().kind == TokenKind::Symbol {
                match self.peek().text.as_str() {
                    "{" => braces += 1,
                    "}" if braces == 0 => return,
                    "}" => {
                        braces -= 1;
                        if braces == 0 && self.i >= failed { self.i += 1; return; }
                    }
                    ";" if braces == 0 => { self.i += 1; return; }
                    _ => {}
                }
            }
            self.i += 1;
        }
    }
    /// `switch (value) { case a: ... default: ... }` becomes an if chain over
    /// strict equality, tested in source order. The discriminant is evaluated
    /// once. Each case ends with break, return or continue, or groups with the
    /// next case; running from one case body into the next is not supported.
    #[inline(never)]
    fn switch_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let span = self.expect("switch")?.span;
        self.expect("(")?;
        let discriminant = self.comma_expression()?;
        self.expect(")")?;
        self.expect("{")?;
        if self.depth >= 128 {
            return Err(Diagnostic::new("unsupported", "Block nesting exceeds the prototype limit of 128.", span));
        }
        self.depth += 1;
        // Each clause: its case values (none for default) and its body.
        let mut clauses: Vec<(Vec<Expr>, bool, Vec<Stmt>, Span)> = Vec::new();
        let mut default_seen = false;
        while !self.accept("}") {
            let at = self.peek().span;
            let mut values = Vec::new();
            let mut default = false;
            if !self.is("case") && !self.is("default") {
                return Err(Diagnostic::new("syntax", "Expected case or default in a switch body.", at));
            }
            // Adjacent labels share one body.
            while self.is("case") || self.is("default") {
                if self.take().text == "default" {
                    if default_seen {
                        return Err(Diagnostic::new("syntax", "A switch can have one default clause.", at));
                    }
                    default_seen = true;
                    default = true;
                } else {
                    values.push(self.comma_expression()?);
                }
                self.expect(":")?;
            }
            let mut body = Vec::new();
            while !self.is("case") && !self.is("default") && !self.is("}") {
                if self.peek().kind == TokenKind::End {
                    self.expect("}")?;
                }
                body.push(self.block_statement()?);
                body.append(&mut self.pending);
            }
            clauses.push((values, default, body, at));
        }
        self.depth -= 1;
        if clauses.len() > 120 {
            return Err(Diagnostic::new("unsupported", "A switch supports at most 120 clauses.", span));
        }
        let last = clauses.len().saturating_sub(1);
        for (index, (_, _, body, at)) in clauses.iter_mut().enumerate() {
            let ended = Self::end_switch_clause(body);
            if !ended && index != last && !body.is_empty() {
                return Err(Diagnostic::new(
                    "unsupported",
                    "This case runs into the next one. End it with break, return or continue, or leave it empty to share the next body.",
                    *at,
                ));
            }
            if let Some(nested) = Self::switch_break(body) {
                return Err(Diagnostic::new(
                    "unsupported",
                    "A break inside a nested statement of a case is not supported; make break the last statement of the case, or restructure with if and else.",
                    nested,
                ));
            }
        }
        // A named discriminant is compared in place; any other value is held once.
        let mut held = None;
        let subject = if matches!(discriminant.kind, ExprKind::Name(_)) {
            discriminant
        } else {
            let holder = self.pattern_holder();
            let name = Expr::new(ExprKind::Name(holder.text.clone()), discriminant.span);
            held = Some(Stmt { labels: Vec::new(), control_target: None,
                kind: StmtKind::Let { rebindable: false, name: holder, annotation: None, value: discriminant, binding: None },
                span,
            });
            name
        };
        // Build the chain from the last clause back; default is the final else.
        let mut default_body = None;
        let mut chain: Vec<Stmt> = Vec::new();
        let mut tested = Vec::new();
        for (values, default, body, at) in clauses {
            if default {
                default_body = Some(body);
            } else {
                tested.push((values, body, at));
            }
        }
        if let Some(body) = default_body {
            chain = body;
        }
        for (values, body, at) in tested.into_iter().rev() {
            let mut condition: Option<Expr> = None;
            for value in values {
                let value_span = value.span;
                let equal = Expr::new(
                    ExprKind::Binary {
                        op: BinaryOp::Equal,
                        operator: value_span,
                        left: Box::new(Expr::new(subject.kind.clone(), subject.span)),
                        right: Box::new(value),
                    },
                    value_span,
                );
                condition = Some(match condition {
                    None => equal,
                    Some(previous) => Expr::new(
                        ExprKind::Binary { op: BinaryOp::Or, operator: value_span, left: Box::new(previous), right: Box::new(equal) },
                        value_span,
                    ),
                });
            }
            let condition = condition.expect("case clause has a value");
            chain = vec![Stmt { labels: Vec::new(), control_target: None, kind: StmtKind::If { condition, then_body: body, else_body: chain }, span: at }];
        }
        match held {
            Some(first) => {
                self.pending.append(&mut chain);
                Ok(first)
            }
            None => Ok(Stmt { labels: Vec::new(), control_target: None, kind: StmtKind::Block(chain), span }),
        }
    }

    /// Remove the break that ends a clause, also when it ends the clause's
    /// final block. Report whether the clause ends explicitly.
    #[inline(never)]
    fn end_switch_clause(body: &mut Vec<Stmt>) -> bool {
        if body.last().is_some_and(|statement|matches!(statement.kind,StmtKind::Break)&&!statement.labels.is_empty()) {return true;}
        match body.last_mut().map(|statement| &mut statement.kind) {
            Some(StmtKind::Break) => {
                body.pop();
                true
            }
            Some(StmtKind::Return(_) | StmtKind::Continue) => true,
            Some(StmtKind::Block(inner)) => Self::end_switch_clause(inner),
            _ => false,
        }
    }

    /// A break that would leave the switch from inside a nested statement.
    /// Loops own their breaks.
    #[inline(never)]
    fn switch_break(body: &[Stmt]) -> Option<Span> {
        for statement in body {
            let found = match &statement.kind {
                StmtKind::Break if statement.labels.is_empty() => Some(statement.span),
                StmtKind::Block(inner) => Self::switch_break(inner),
                StmtKind::If { then_body, else_body, .. } => Self::switch_break(then_body).or_else(|| Self::switch_break(else_body)),
                _ => None,
            };
            if found.is_some() {
                return found;
            }
        }
        None
    }

    /// The body of if, else, while, do, for or for...of: a block, or one
    /// statement that forms a block of its own.
    #[inline(never)]
    fn body(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        if self.is("{") {
            return self.block();
        }
        if self.depth >= 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Block nesting exceeds the prototype limit of 128.",
                self.peek().span,
            ));
        }
        if self.is(";") {
            return Err(Diagnostic::new("syntax", "An empty statement cannot be a body; write {} for an empty body.", self.peek().span));
        }
        if self.is("const") || self.is("let") || self.is("function") {
            return Err(Diagnostic::new("syntax", "A declaration cannot be a body without braces; put it in a block.", self.peek().span));
        }
        self.depth += 1;
        let statement = self.block_statement();
        self.depth -= 1;
        let mut body = vec![statement?];
        body.append(&mut self.pending);
        Ok(body)
    }
    // Return directly so recursive blocks retain only one statement temporary.
    fn block_statement(&mut self) -> Result<Stmt, Diagnostic> {
        if self.peek().kind == TokenKind::Name && self.tokens.get(self.i+1).is_some_and(|token|token.text==":") {
            return self.labeled_statement();
        }
        if self.is("function") {
            self.local_function_statement()
        } else if self.match_statement_header() {
            self.lower_match_statement()
        } else if self.is("try") {
            self.try_statement()
        } else if self.is("for") {
            if self.for_of_header() {
                self.for_of_statement()
            } else {
                self.for_statement()
            }
        } else if self.is("if") {
            self.if_statement()
        } else if self.is("while") {
            self.while_statement()
        } else if self.is("do") {
            self.do_statement()
        } else if self.is("switch") && self.tokens.get(self.i + 1).is_some_and(|token| token.text == "(") {
            self.switch_statement()
        } else if self.is("{") {
            let span = self.peek().span;
            self.block().map(|body| Stmt { labels: Vec::new(), control_target: None,
                kind: StmtKind::Block(body),
                span,
            })
        } else {
            self.statement(true)
        }
    }
    #[inline(never)]
    fn try_statement(&mut self)->Result<Stmt,Diagnostic> {
        let start=self.expect("try")?.span;
        let body=self.block()?;
        let catch=if self.accept("catch") {
            let name=if self.accept("(") {let name=self.name()?;self.expect(")")?;Some(name)}else{None};
            Some(CatchClause{error_type:None,error_contract:None,name,binding:None,body:self.block()?})
        }else{None};
        let finally=if self.accept("finally") {Some(self.block()?)}else{None};
        if catch.is_none() && finally.is_none() {
            return Err(Diagnostic::new("syntax","A try block requires catch, finally, or both.",start));
        }
        let span=Span::new(start.start,self.tokens[self.i-1].span.end);
        Ok(Stmt{labels:Vec::new(),control_target:None,kind:StmtKind::Try(Box::new(ExceptionRegion{body,catch,finally})),span})
    }
    #[inline(never)]
    fn labeled_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let mut labels=Vec::new();
        while self.peek().kind==TokenKind::Name && self.tokens.get(self.i+1).is_some_and(|token|token.text==":") {
            labels.push(self.name()?);self.expect(":")?;
            if labels.len()>32 {return Err(Diagnostic::new("unsupported","A statement supports at most 32 labels.",labels[0].span));}
        }
        if matches!(self.peek().text.as_str(),"let"|"const"|"type"|"function") {
            return Err(Diagnostic::new("syntax","A label must precede a statement, not a declaration.",self.peek().span));
        }
        if self.is(";") {let span=self.take().span;return Ok(Stmt{labels,control_target:None,kind:StmtKind::Block(Vec::new()),span});}
        let classic_for=self.is("for") && !self.for_of_header();
        let mut statement=self.block_statement()?;
        if classic_for {
            if let StmtKind::Block(body)=&mut statement.kind {
                body.last_mut().expect("classic-for loop").labels=labels;
            }
        } else if matches!(statement.kind,StmtKind::While{..}|StmtKind::Loop{..}|StmtKind::ForOf{..}|StmtKind::Block(_)) {
            statement.labels=labels;
        } else {
            let span=statement.span;
            statement=Stmt{labels,control_target:None,kind:StmtKind::Block(vec![statement]),span};
        }
        Ok(statement)
    }

    fn if_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let span = self.expect("if")?.span;
        self.expect("(")?;
        let condition = self.comma_expression()?;
        self.expect(")")?;
        let then_body = self.body()?;
        let else_body = self.if_else_body()?;
        Ok(Self::conditional_statement(span, condition, then_body, else_body))
    }

    // Else-chain temporaries do not occupy every recursively retained block.
    #[inline(never)]
    fn if_else_body(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        if !self.accept("else") {
            return Ok(Vec::new());
        }
        if !self.is("if") {
            return self.body();
        }
        if self.depth >= 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Conditional nesting exceeds the prototype limit of 128.",
                self.peek().span,
            ));
        }
        self.depth += 1;
        let nested = self.if_statement();
        self.depth -= 1;
        Ok(vec![nested?])
    }

    // Construct the large statement value after recursive parsing returns.
    #[inline(never)]
    fn conditional_statement(span: Span, condition: Expr, then_body: Vec<Stmt>, else_body: Vec<Stmt>) -> Stmt {
        Stmt { labels: Vec::new(), control_target: None,
            kind: StmtKind::If { condition, then_body, else_body },
            span,
        }
    }
    fn while_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let span = self.expect("while")?.span;
        self.expect("(")?;
        let condition = self.comma_expression()?;
        self.expect(")")?;
        let body = self.body()?;
        Ok(Stmt { labels: Vec::new(), control_target: None,
            kind: StmtKind::While { condition, body },
            span,
        })
    }
    /// `const [a, b] = value;` and `const { x } = value;`. Kept out of the
    /// recursive statement frame.
    #[inline(never)]
    fn pattern_declaration(&mut self) -> Result<StmtKind, Diagnostic> {
        let rebindable = self.take().text == "let";
        self.pattern_declarator(rebindable)
    }

    /// One pattern declarator after `const`, `let` or a comma. Its position
    /// declarations are appended to `pending`.
    #[inline(never)]
    fn pattern_declarator(&mut self, rebindable: bool) -> Result<StmtKind, Diagnostic> {
        Ok({
            // `const [a, b] = value;` binds the whole value once, then each
            // name takes its position under the ordinary binding rules.
            let whole = self.pattern_holder();
            let mut positions = Vec::new();
            self.binding_pattern(&whole, rebindable, &mut positions, 0)?;
            let annotation = if self.accept(":") {
                Some(self.value_annotation()?)
            } else {
                None
            };
            self.expect("=")?;
            let value = self.expression()?;
            // A named source needs no holder: each name takes its position
            // from the source itself, so copied positions leave it usable.
            let direct = match (&value.kind, &annotation) {
                (ExprKind::Name(source), None) if !positions.is_empty() => Some(source.clone()),
                _ => None,
            };
            if let Some(source) = direct {
                for position in &mut positions {
                    if let StmtKind::Let{value:selected,..}=&mut position.kind {Self::replace_pattern_source(selected,&whole.text,&source,value.span);}
                }
                let first = positions.remove(0);
                self.pending.append(&mut positions);
                first.kind
            } else {
                self.pending.append(&mut positions);
                StmtKind::Let {
                    rebindable: false,
                    name: whole,
                    annotation,
                    value,
                    binding: None,
                }
            }
        })
    }

    /// `[a, b] = value;` and `({ x, y: z } = value);` assign to existing
    /// places. The statement becomes ordinary assignments in source order; the
    /// first is returned and the others follow it through `pending`.
    #[inline(never)]
    fn destructuring_assignment(&mut self, expression: Expr) -> Result<StmtKind, Diagnostic> {
        let span = expression.span;
        let ExprKind::Assign { target, value, .. } = expression.kind else {
            unreachable!("destructuring assignment expression")
        };
        let (target, value) = (*target, *value);
        let mut out = Vec::new();
        let spread = |element: &Expr| matches!(element.kind, ExprKind::Unary { op: UnaryOp::Spread, .. });
        // A written array with fewer values than targets can never fill them.
        if let (ExprKind::Array(targets), ExprKind::Array(values)) = (&target.kind, &value.kind)
            && !values.iter().any(spread)
        {
            let needed = targets.iter().filter(|element| !spread(element)).count();
            if values.len() < needed {
                return Err(Diagnostic::new(
                    "bounds",
                    format!("This destructuring assignment has {needed} targets but the array has {} values.", values.len()),
                    value.span,
                ));
            }
        }
        match (target.kind, value.kind) {
            // Each written value is held first, so `[a, b] = [b, a]` swaps
            // without building an array and owned values can move.
            (ExprKind::Array(targets), ExprKind::Array(values))
                if values.len() >= targets.len() && !targets.iter().any(spread) && !values.iter().any(spread) =>
            {
                let mut held = Vec::new();
                for value in values {
                    let holder = self.pattern_holder();
                    held.push(Expr::new(ExprKind::Name(holder.text.clone()), value.span));
                    out.push(Stmt { labels: Vec::new(), control_target: None,
                        kind: StmtKind::Let { rebindable: false, name: holder, annotation: None, value, binding: None },
                        span,
                    });
                }
                for (target, value) in targets.into_iter().zip(held) {
                    self.assign_through_pattern(target, value, span, &mut out, 0)?;
                }
            }
            (target_kind, value_kind) => {
                let target = Expr::new(target_kind, target.span);
                let value = Expr::new(value_kind, value.span);
                // A named source is read in place; any other value is held once.
                let source = if matches!(value.kind, ExprKind::Name(_)) {
                    value
                } else {
                    let holder = self.pattern_holder();
                    let name = Expr::new(ExprKind::Name(holder.text.clone()), value.span);
                    out.push(Stmt { labels: Vec::new(), control_target: None,
                        kind: StmtKind::Let { rebindable: false, name: holder, annotation: None, value, binding: None },
                        span,
                    });
                    name
                };
                self.assign_pattern_positions(target, source, span, &mut out, 0)?;
            }
        }
        if out.is_empty() {
            return Err(Diagnostic::new("syntax", "A destructuring assignment needs at least one target.", span));
        }
        let first = out.remove(0);
        self.pending.append(&mut out);
        Ok(first.kind)
    }

    /// Assign each position or field of `source` to the matching target.
    #[inline(never)]
    fn assign_pattern_positions(&mut self, target: Expr, source: Expr, span: Span, out: &mut Vec<Stmt>, depth: usize) -> Result<(), Diagnostic> {
        if depth > 32 {
            return Err(Diagnostic::new("unsupported", "Destructuring pattern nesting exceeds the limit of 32.", target.span));
        }
        let select = |source: &Expr| Expr::new(source.kind.clone(), source.span);
        match target.kind {
            ExprKind::Array(targets) => {
                let count = targets.len();
                for (position, element) in targets.into_iter().enumerate() {
                    let at = element.span;
                    if let ExprKind::Unary { op: UnaryOp::Spread, value: rest, .. } = element.kind {
                        if position + 1 != count {
                            return Err(Diagnostic::new("syntax", "A rest target must be last in a destructuring assignment.", at));
                        }
                        // Rest explicitly copies the remaining elements.
                        let slice = Expr::new(ExprKind::Field(Box::new(select(&source)), "slice".to_owned()), at);
                        let start = Expr::new(ExprKind::Integer(position.to_string()), at);
                        let copied = Expr::new(ExprKind::Call(Box::new(slice), vec![start]), at);
                        self.assign_through_pattern(*rest, copied, span, out, depth + 1)?;
                        continue;
                    }
                    let index = Expr::new(ExprKind::Integer(position.to_string()), at);
                    let selected = Expr::new(ExprKind::Index(Box::new(select(&source)), Box::new(index)), at);
                    self.assign_through_pattern(element, selected, span, out, depth + 1)?;
                }
            }
            ExprKind::Record(fields) => {
                for (key, element) in fields {
                    if key.text == "record#spread" {
                        return Err(Diagnostic::new("unsupported", "A record rest target is not supported in a destructuring assignment; declare it with const or let.", key.span));
                    }
                    let selected = Expr::new(ExprKind::Field(Box::new(select(&source)), key.text.clone()), key.span);
                    self.assign_through_pattern(element, selected, span, out, depth + 1)?;
                }
            }
            _ => unreachable!("pattern target"),
        }
        Ok(())
    }

    /// Assign `value` to one target, which may itself be a nested pattern.
    #[inline(never)]
    fn assign_through_pattern(&mut self, target: Expr, value: Expr, span: Span, out: &mut Vec<Stmt>, depth: usize) -> Result<(), Diagnostic> {
        match &target.kind {
            ExprKind::Array(_) | ExprKind::Record(_) => {
                let holder = self.pattern_holder();
                let name = Expr::new(ExprKind::Name(holder.text.clone()), value.span);
                out.push(Stmt { labels: Vec::new(), control_target: None,
                    kind: StmtKind::Let { rebindable: false, name: holder, annotation: None, value, binding: None },
                    span,
                });
                self.assign_pattern_positions(target, name, span, out, depth)
            }
            ExprKind::Assign { .. } => Err(Diagnostic::new(
                "unsupported",
                "Default values are not supported in a destructuring assignment; assign first, then use ??= on the target.",
                target.span,
            )),
            _ => {
                out.push(Stmt { labels: Vec::new(), control_target: None, kind: StmtKind::Assign { target, value, operator: None }, span });
                Ok(())
            }
        }
    }

    /// Initializer of one named declarator. `let name: Type;` has none and
    /// gets the deferred placeholder; the variable is assigned later.
    fn declarator_value(
        &mut self,
        rebindable: bool,
        name: &Token,
        annotated: bool,
    ) -> Result<Expr, Diagnostic> {
        if rebindable && (self.is(";") || self.is(",")) {
            if !annotated {
                return Err(Diagnostic::new(
                    "annotation-required",
                    format!(
                        "Declare the type of {:?} when it has no initializer, for example `let {}: int;`.",
                        name.text, name.text
                    ),
                    name.span,
                ));
            }
            return Ok(Expr::deferred(self.tokens[self.i - 1].span));
        }
        self.expect("=")?;
        self.expression()
    }

    /// The declarators after the first in `let a = 1, [b, c] = pair;`. Each
    /// becomes an ordinary declaration that follows the first, in source order.
    #[inline(never)]
    fn declaration_list(&mut self, rebindable: bool) -> Result<(), Diagnostic> {
        while self.accept(",") {
            let span = self.peek().span;
            if matches!(self.peek().text.as_str(), "[" | "{") {
                // The declarator's own positions follow its whole-value declaration.
                let at = self.pending.len();
                let kind = self.pattern_declarator(rebindable)?;
                self.pending.insert(at, Stmt { labels: Vec::new(), control_target: None, kind, span });
                continue;
            }
            let name = self.name()?;
            let annotation = if self.accept(":") { Some(self.value_annotation()?) } else { None };
            let value = self.declarator_value(rebindable, &name, annotation.is_some())?;
            self.pending.push(Stmt { labels: Vec::new(), control_target: None,
                kind: StmtKind::Let { rebindable, name, annotation, value, binding: None },
                span,
            });
        }
        Ok(())
    }

    /// `const name = value` or `let name: Type;`. Kept out of the statement
    /// frame: nested function bodies recurse through this path.
    #[inline(never)]
    fn named_declaration(&mut self) -> Result<StmtKind, Diagnostic> {
        let rebindable = self.take().text == "let";
        let name = self.name()?;
        let annotation = if self.accept(":") {
            Some(self.value_annotation()?)
        } else {
            None
        };
        let value = self.declarator_value(rebindable, &name, annotation.is_some())?;
        Ok(StmtKind::Let {
            rebindable,
            name,
            annotation,
            value,
            binding: None,
        })
    }

    /// Every statement form except a named declaration.
    #[inline(never)]
    fn other_statement(&mut self, span: Span, control_label: &mut Option<Token>) -> Result<StmtKind, Diagnostic> {
        Ok(if self.accept("break") {
            if self.peek().kind==TokenKind::Name {*control_label=Some(self.name()?);}
            StmtKind::Break
        } else if self.accept("continue") {
            if self.peek().kind==TokenKind::Name {*control_label=Some(self.name()?);}
            StmtKind::Continue
        } else if self.is("const") || self.is("let") {
            self.pattern_declaration()?
        } else if self.is("function") {
            // A statement that starts with `function` would be a declaration.
            return Err(Diagnostic::new(
                "syntax",
                "A function declaration belongs at the top level of the file. Inside a function, bind a function expression or an arrow function to a const.",
                span,
            ));
        } else if self.accept("throw") {
            if self.source[span.end..self.peek().span.start].contains(['\n','\r']) {
                return Err(Diagnostic::new("syntax","A thrown value must begin on the same line as throw.",span));
            }
            StmtKind::Throw(self.comma_expression()?)
        } else if self.accept("return") {
            StmtKind::Return(if self.is(";") {
                None
            } else {
                Some(self.comma_expression()?)
            })
        } else {
            let expression = self.comma_expression()?;
            if matches!(&expression.kind, ExprKind::Assign { target, .. } if matches!(target.kind, ExprKind::Array(_) | ExprKind::Record(_))) {
                self.destructuring_assignment(expression)?
            } else {
                Self::expression_statement(expression)
            }
        })
    }

    // Share clause parsing without retaining an extra Stmt-sized caller frame.
    fn statement(&mut self, terminated: bool) -> Result<Stmt, Diagnostic> {
        let span = self.peek().span;
        let mut control_label=None;
        let named_declaration = (self.is("const") || self.is("let"))
            && !self.tokens.get(self.i + 1).is_some_and(|token| matches!(token.text.as_str(), "[" | "{"));
        let kind = if named_declaration {
            self.named_declaration()?
        } else {
            self.other_statement(span, &mut control_label)?
        };
        // A declaration began with its keyword at `span`; a lowered
        // destructuring assignment also starts with a declaration but did not.
        if self.is(",")
            && matches!(kind, StmtKind::Let { .. })
            && matches!(&self.source[span.start..span.end], "const" | "let")
        {
            self.declaration_list(self.source[span.start..span.end] == *"let")?;
        }
        if terminated {
            self.expect(";")?;
        }
        Ok(Stmt { labels: control_label.into_iter().collect(), control_target: None, kind, span })
    }
    // Discarding the result preserves existing standalone owned replacement.
    // Keep Stmt-sized construction out of recursive parser statement frames.
    fn expression_statement(expression: Expr) -> StmtKind {
        match expression.kind {
            ExprKind::CompoundAssign {
                target,
                value,
                op,
                operator,
            } => StmtKind::Assign {
                target: *target,
                value: *value,
                operator: Some((op, operator)),
            },
            ExprKind::Assign { target, value, .. } => StmtKind::Assign {
                target: *target,
                value: *value,
                operator: None,
            },
            _ => StmtKind::Expr(expression),
        }
    }

    fn expression(&mut self) -> Result<Expr, Diagnostic> {
        self.precedence(0)
    }
    #[inline(never)]
    fn comma_expression(&mut self)->Result<Expr,Diagnostic>{
        let first=self.expression()?;
        if !self.accept(","){return Ok(first);}
        self.comma_tail(first)
    }
    // Construction and later operands must not occupy every nested index frame.
    #[inline(never)]
    fn comma_tail(&mut self, first: Expr)->Result<Expr,Diagnostic>{
        let mut values=vec![first];
        loop{
            if matches!(self.peek().text.as_str(),"let"|"const"|"var"){return Err(Diagnostic::new("syntax","A declaration cannot be a comma operand.",self.peek().span));}
            values.push(self.expression()?);if !self.accept(","){break;}
        }
        Self::finish_sequence(values)
    }
    #[inline(never)]
    fn finish_sequence(values:Vec<Expr>)->Result<Expr,Diagnostic>{
        let span=Span::new(values.first().unwrap().span.start,values.last().unwrap().span.end);
        Ok(Expr::new(ExprKind::Sequence(values),span))
    }


    fn do_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let span = self.expect("do")?.span;
        let body = self.body()?;
        self.expect("while")?;
        self.expect("(")?;
        let condition = self.comma_expression()?;
        self.expect(")")?;
        self.expect(";")?;
        Ok(Stmt { labels: Vec::new(), control_target: None,
            kind: StmtKind::Loop {
                condition: Some(condition),
                body,
                update: Vec::new(),
                post_test: true,
                tail_reachable: true,
            },
            span,
        })
    }

    fn for_of_header(&self) -> bool {
        // Select the recursive parser before retaining classic-for temporaries.
        let declaration = self
            .tokens
            .get(self.i + 2)
            .is_some_and(|token| token.text == "const" || token.text == "let");
        // A destructuring pattern is skipped to its matching close.
        let start = self.i + if declaration { 3 } else { 2 };
        if declaration && self.tokens.get(start).is_some_and(|token| matches!(token.text.as_str(), "[" | "{")) {
            let mut closes=Vec::new();
            for (offset,token) in self.tokens[start..].iter().enumerate(){
                if token.kind!=TokenKind::Symbol {continue;}
                match token.text.as_str(){
                    "["=>closes.push("]"),"{"=>closes.push("}"),"("=>closes.push(")"),
                    "]"|"}"|")"=>{
                        if closes.pop()!=Some(token.text.as_str()){return false;}
                        if closes.is_empty(){return self.tokens.get(start+offset+1).is_some_and(|token|matches!(token.text.as_str(), "of" | "in"));}
                    },
                    ";" if closes.len()==1=>return false,
                    _=>{},
                }
            }
            return false;
        }
        self.tokens
            .get(self.i + if declaration { 4 } else { 3 })
            .is_some_and(|token| matches!(token.text.as_str(), "of" | "in"))
    }

    /// A hidden name for the whole value of a destructuring pattern. `#`
    /// cannot appear in a source name.
    #[inline(never)]
    fn pattern_holder(&mut self) -> Token {
        let span = self.peek().span;
        self.next_pattern += 1;
        Token { text: format!("pattern#{}", self.next_pattern), span, kind: TokenKind::Name }
    }

    /// Parse `[a, , b]` or `{ x, label: text }` and append one declaration per
    /// name. A nested pattern first binds its own position to a hidden name.
    #[inline(never)]
    fn binding_pattern(&mut self, whole: &Token, rebindable: bool, output: &mut Vec<Stmt>, depth: usize) -> Result<(), Diagnostic> {
        if depth > 32 {
            return Err(Diagnostic::new("unsupported", "Destructuring pattern nesting exceeds the limit of 32.", self.peek().span));
        }
        let array = self.peek().text == "[";
        let close = if array { "]" } else { "}" };
        self.take();
        let mut position = 0usize;
        let mut excluded = Vec::new();
        while !self.accept(close) {
            if array && self.accept(",") {
                // An empty slot skips one position.
                position += 1;
                continue;
            }
            if self.is("...") {
                if array {self.array_rest_pattern(whole, rebindable, output, position, depth)?;}
                else {self.record_rest_pattern(whole,rebindable,output,&excluded)?;}
                self.expect(close)?;
                break;
            }
            let start = self.peek().span;
            let source = |span: Span| Expr::new(ExprKind::Name(whole.text.clone()), span);
            // Record patterns name a field, optionally followed by a target.
            let (mut selected, shorthand) = if array {
                let index = Expr::new(ExprKind::Integer(position.to_string()), start);
                (Expr::new(ExprKind::Index(Box::new(source(start)), Box::new(index)), start), None)
            } else {
                let field = self.name()?;
                excluded.push(field.text.clone());
                let selected = Expr::new(ExprKind::Field(Box::new(source(field.span)), field.text.clone()), field.span);
                if self.accept(":") { (selected, None) } else { (selected, Some(field)) }
            };
            let name = match shorthand {
                Some(field) => field,
                None if matches!(self.peek().text.as_str(), "[" | "{") => {
                    let holder = self.pattern_holder();
                    let mut children=Vec::new();
                    self.binding_pattern(&holder, rebindable, &mut children, depth + 1)?;
                    selected=self.pattern_default(selected)?;
                    output.push(Stmt { labels: Vec::new(), control_target: None,
                        kind: StmtKind::Let { rebindable: false, name: holder.clone(), annotation: None, value: selected, binding: None },
                        span: start,
                    });
                    output.append(&mut children);
                    position += 1;
                    if !self.is(close) { self.expect(",")?; }
                    continue;
                }
                None => self.name()?,
            };
            selected=self.pattern_default(selected)?;
            // The selection carries the span of its name, which marks the
            // declaration as a pattern position for editor type facts.
            selected.span = name.span;
            output.push(Stmt { labels: Vec::new(), control_target: None,
                kind: StmtKind::Let { rebindable, name, annotation: None, value: selected, binding: None },
                span: start,
            });
            position += 1;
            if !self.is(close) { self.expect(",")?; }
        }
        Ok(())
    }

    #[inline(never)]
    fn record_rest_pattern(&mut self,whole:&Token,rebindable:bool,output:&mut Vec<Stmt>,excluded:&[String])->Result<(),Diagnostic>{
        let start=self.expect("...")?.span;let name=self.name()?;
        if !self.is("}"){return Err(Diagnostic::new("syntax","A rest binding must be last and cannot have a default or trailing comma.",self.peek().span));}
        let base=Expr::new(ExprKind::Name(whole.text.clone()),name.span);
        let method=Expr::new(ExprKind::Field(Box::new(base),"pattern#record-rest".into()),name.span);
        let args=excluded.iter().map(|key|Expr::new(ExprKind::String(key.clone()),name.span)).collect();
        let value=Expr::new(ExprKind::Call(Box::new(method),args),name.span);
        output.push(Stmt{labels:Vec::new(),control_target:None,kind:StmtKind::Let{rebindable,name,annotation:None,value,binding:None},span:start});Ok(())
    }

    /// Rest syntax explicitly requests the independent copy provided by slice.
    #[inline(never)]
    fn array_rest_pattern(&mut self, whole:&Token, rebindable:bool, output:&mut Vec<Stmt>, position:usize, depth:usize)->Result<(),Diagnostic>{
        let start=self.expect("...")?.span;
        let mut children=Vec::new();
        let name=if matches!(self.peek().text.as_str(),"["|"{"){
            let holder=self.pattern_holder();self.binding_pattern(&holder,rebindable,&mut children,depth+1)?;holder
        }else{self.name()?};
        if !self.is("]"){return Err(Diagnostic::new("syntax","A rest binding must be last and cannot have a default or trailing comma.",self.peek().span));}
        let source=Expr::new(ExprKind::Name(whole.text.clone()),name.span);
        let method=Expr::new(ExprKind::Field(Box::new(source),"pattern#rest".into()),name.span);
        let index=Expr::new(ExprKind::Integer(position.to_string()),name.span);
        let value=Expr::new(ExprKind::Call(Box::new(method),vec![index]),name.span);
        output.push(Stmt{labels:Vec::new(),control_target:None,kind:StmtKind::Let{rebindable:rebindable && children.is_empty(),name,annotation:None,value,binding:None},span:start});
        output.append(&mut children);
        Ok(())
    }

    #[inline(never)]
    fn replace_pattern_source(expr:&mut Expr,whole:&str,source:&str,span:Span){
        match &mut expr.kind {
            ExprKind::Binary{op:BinaryOp::PatternDefault,left,..}=>Self::replace_pattern_source(left,whole,source,span),
            ExprKind::Unary{op:UnaryOp::PatternSelection,value,..}=>Self::replace_pattern_source(value,whole,source,span),
            ExprKind::Call(fun,_)=>Self::replace_pattern_source(fun,whole,source,span),
            ExprKind::Index(base,_)|ExprKind::Field(base,_) if matches!(&base.kind,ExprKind::Name(name) if name==whole)=>{base.kind=ExprKind::Name(source.into());base.span=span;},
            _=>{},
        }
    }

    #[inline(never)]
    fn pattern_default(&mut self,selected:Expr)->Result<Expr,Diagnostic>{
        if !self.is("="){return Ok(selected);}
        let operator=self.take().span;
        let span=selected.span;
        let selected=Expr::new(ExprKind::Unary{op:UnaryOp::PatternSelection,operator,value:Box::new(selected)},span);
        let fallback=self.expression()?;
        Ok(Expr::new(ExprKind::Binary{op:BinaryOp::PatternDefault,operator,left:Box::new(selected),right:Box::new(fallback)},span))
    }

    // Nested loop bodies recurse through this frame; the header and the
    // statement construction keep their temporaries in separate helpers.
    fn for_of_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let header = self.for_of_header_clause()?;
        let body = self.body()?;
        Ok(Self::for_of_loop(header, body))
    }

    #[inline(never)]
    fn for_of_header_clause(&mut self) -> Result<Box<(Token, bool, Expr, Vec<Stmt>, Span)>, Diagnostic> {
        let span = self.expect("for")?.span;
        self.expect("(")?;
        if !self.is("const") && !self.is("let") {
            return Err(Diagnostic::new(
                "unsupported",
                "for...of requires a const or let iteration binding.",
                self.peek().span,
            ));
        }
        let rebindable = self.take().text == "let";
        // A pattern iterates a hidden whole element; its names are declared
        // first in the body and borrow the element's positions.
        let mut positions = Vec::new();
        let (name, rebindable) = if matches!(self.peek().text.as_str(), "[" | "{") {
            let whole = self.pattern_holder();
            self.binding_pattern(&whole, rebindable, &mut positions, 0)?;
            (whole, false)
        } else {
            (self.name()?, rebindable)
        };
        let keys = self.is("in");
        if keys && !positions.is_empty() {
            return Err(Diagnostic::new("unsupported", "for...in requires a named binding.", name.span));
        }
        if keys { self.expect("in")?; } else { self.expect("of")?; }
        let iterable = if keys{self.comma_expression()?}else{self.expression()?};
        let iterable = if keys { Self::object_key_call("keys#in", vec![iterable], span) } else { iterable };
        self.expect(")")?;
        Ok(Box::new((name, rebindable, iterable, positions, span)))
    }

    #[inline(never)]
    fn for_of_loop(header: Box<(Token, bool, Expr, Vec<Stmt>, Span)>, mut body: Vec<Stmt>) -> Stmt {
        let (name, rebindable, iterable, mut positions, span) = *header;
        if !positions.is_empty() {
            positions.append(&mut body);
            body = positions;
        }
        Stmt { labels: Vec::new(), control_target: None,
            kind: StmtKind::ForOf {
                name,
                rebindable,
                iterable,
                body,
                binding: None,
            },
            span,
        }
    }

    /// The update clause: one or more assignments or expressions separated
    /// by commas, run in source order on every continuing edge.
    #[inline(never)]
    fn for_update(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        self.for_expression_list()
    }

    #[inline(never)]
    fn for_expression_list(&mut self)->Result<Vec<Stmt>,Diagnostic>{
        let mut statements=Vec::new();
        loop {
            if matches!(self.peek().text.as_str(),"let"|"const"|"var"){
                return Err(Diagnostic::new("syntax","A declaration cannot be a comma operand.",self.peek().span));
            }
            let expression=self.expression()?;
            let span=expression.span;
            let pattern=matches!(&expression.kind,ExprKind::Assign{target,..} if matches!(target.kind,ExprKind::Array(_) | ExprKind::Record(_)));
            let kind=if pattern {self.destructuring_assignment(expression)?}
                else if matches!(expression.kind,ExprKind::Assign{..} | ExprKind::CompoundAssign{..}){Self::expression_statement(expression)}
                else {StmtKind::Expr(Expr::new(ExprKind::Sequence(vec![expression]),span))};
            statements.push(Stmt{labels:Vec::new(),control_target:None,kind,span});
            statements.append(&mut self.pending);
            if !self.accept(","){return Ok(statements);}
        }
    }

    fn for_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let span = self.expect("for")?.span;
        self.expect("(")?;
        let mut scoped = Vec::new();
        if !self.is(";") && !matches!(self.peek().text.as_str(),"let"|"const") {
            scoped.extend(self.for_expression_list()?);
        } else if !self.is(";") {
            let initializer = self.statement(false)?;
            if !matches!(
                initializer.kind,
                StmtKind::Let { .. } | StmtKind::Assign { .. } | StmtKind::Expr(_)
            ) {
                return Err(Diagnostic::new(
                    "syntax",
                    "A for initializer must be a declaration, assignment, or expression.",
                    initializer.span,
                ));
            }
            scoped.push(initializer);
            scoped.append(&mut self.pending);
        }
        self.expect(";")?;
        let condition = if self.is(";") {
            None
        } else {
            Some(self.comma_expression()?)
        };
        self.expect(";")?;
        let update = if self.is(")") {
            Vec::new()
        } else {
            self.for_update()?
        };
        self.expect(")")?;
        let body = self.body()?;
        scoped.push(Stmt { labels: Vec::new(), control_target: None,
            kind: StmtKind::Loop {
                condition,
                body,
                update,
                post_test: false,
                tail_reachable: true,
            },
            span,
        });
        Ok(Stmt { labels: Vec::new(), control_target: None,
            kind: StmtKind::Block(scoped),
            span,
        })
    }

    fn precedence(&mut self, minimum: u8) -> Result<Expr, Diagnostic> {
        // Keep hostile/deep source from overflowing the recursive parser/AST stack.
        if self.depth >= 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Expression nesting exceeds the prototype limit of 128.",
                self.peek().span,
            ));
        }
        self.depth += 1;
        let result = self.binary(minimum);
        self.depth -= 1;
        result
    }
    fn binary(&mut self, minimum: u8) -> Result<Expr, Diagnostic> {
        // Token provenance stays in this parser frame: grouping is erased by
        // primary(), and negative integer spelling is folded by atom().
        let unary_base = matches!(self.peek().text.as_str(), "-" | "+" | "!" | "~");
        let left_start = self.i;
        let left = self.unary()?;
        self.binary_tail(minimum, left, left_start, unary_base)
    }

    // Operator construction begins only after the recursively parsed primary
    // returns, so its temporary operands do not occupy every argument frame.
    fn binary_tail(
        &mut self,
        minimum: u8,
        left: Expr,
        left_start: usize,
        unary_base: bool,
    ) -> Result<Expr, Diagnostic> {
        // Finish ordinary operators before entering a conditional arm. Their
        // temporaries must not remain on every recursive conditional frame.
        let left = self.binary_operator_tail(minimum, left, left_start, unary_base)?;
        if minimum == 0 && self.is("?") {
            self.conditional_tail(left)
        } else {
            Ok(left)
        }
    }

    fn binary_operator_tail(
        &mut self,
        minimum: u8,
        mut left: Expr,
        left_start: usize,
        mut unary_base: bool,
    ) -> Result<Expr, Diagnostic> {
        loop {
            if minimum <= 8 && (self.is("as") || self.is("satisfies")) {
                left = self.type_assertion_tail(left)?;
                unary_base = false;
                continue;
            }
            // The caller parses conditional arms after this frame returns.
            if minimum == 0 && self.is("?") {
                break;
            }
            if self.is("in") && minimum <= 8 {
                self.take();
                let right = self.precedence(9)?;
                let span = Span::new(self.tokens[left_start].span.start, self.tokens[self.i - 1].span.end);
                left = Self::object_key_call("hasOwn#in", vec![right, left], span);
                if left.depth > 128 {
                    return Err(Diagnostic::new("unsupported", "Expression nesting exceeds the prototype limit of 128.", left.span));
                }
                unary_base = false;
                continue;
            }
            let Some((op, precedence)) = (self.peek().kind == TokenKind::Symbol)
                .then(|| match self.peek().text.as_str() {
                    "=" => Some((BinaryOp::Equal, 0)),
                    "&&=" => Some((BinaryOp::And, 0)),
                    "||=" => Some((BinaryOp::Or, 0)),
                    _ => BinaryOp::assignment(&self.peek().text)
                        .map(|op| (op, 0))
                        .or_else(|| BinaryOp::parse(&self.peek().text)),
                })
                .flatten()
            else {
                break;
            };
            if precedence < minimum {
                break;
            }
            let assignment = precedence == 0;
            let operator = self.take().span;
            if op == BinaryOp::Exponent && unary_base {
                return Err(Diagnostic::new(
                    "syntax",
                    "Parenthesize a unary expression used as an exponentiation base.",
                    operator,
                ));
            }
            let right_start = self.i;
            let right = self.precedence(if assignment || op == BinaryOp::Exponent {
                precedence
            } else {
                precedence + 1
            })?;
            if !assignment {
                self.nullish_mixing(&left, &right, op, operator, left_start, right_start)?;
            }
            unary_base = false;
            left = Self::binary_expression(left, right, op, operator, assignment);
            if op == BinaryOp::Coalesce {
                left.span = Span::new(
                    self.tokens[left_start].span.start,
                    self.tokens[self.i - 1].span.end,
                );
            }
            if left.depth > 128 {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Expression nesting exceeds the prototype limit of 128.",
                    left.span,
                ));
            }
        }
        Ok(left)
    }

    #[inline(never)]
    fn type_assertion_tail(&mut self, value: Expr) -> Result<Expr, Diagnostic> {
        let satisfies = self.take().text == "satisfies";
        let annotation = Box::new(self.value_annotation()?);
        let span = Span::new(value.span.start, annotation.span.end);
        let expression = Expr::new(ExprKind::TypeAssertion {
            value: Box::new(value), annotation, satisfies,
        }, span);
        if expression.depth > 128 {
            return Err(Diagnostic::new("unsupported", "Expression nesting exceeds the prototype limit of 128.", span));
        }
        Ok(expression)
    }

    #[inline(never)]
    fn object_key_call(member: &str, args: Vec<Expr>, span: Span) -> Expr {
        let object = Expr::new(ExprKind::Name("Object#in".into()), span);
        let callee = Expr::new(ExprKind::Field(Box::new(object), member.into()), span);
        Expr::new(ExprKind::Call(Box::new(callee), args), span)
    }

    fn parenthesized_tokens(&self, start: usize, end: usize) -> bool {
        if self.tokens[start].text != "(" {
            return false;
        }
        let mut depth = 0usize;
        for (offset, token) in self.tokens[start..end].iter().enumerate() {
            if token.kind != TokenKind::Symbol {
                continue;
            }
            if token.text == "(" {
                depth += 1;
            }
            if token.text == ")" {
                depth -= 1;
                if depth == 0 {
                    return start + offset + 1 == end;
                }
            }
        }
        false
    }
    fn nullish_mixing(
        &self,
        left: &Expr,
        right: &Expr,
        op: BinaryOp,
        operator: Span,
        left_start: usize,
        right_start: usize,
    ) -> Result<(), Diagnostic> {
        let mixed = |expr: &Expr| {
            matches!(&expr.kind, ExprKind::Binary { op: inner, .. }
            if (op == BinaryOp::Coalesce && matches!(inner, BinaryOp::And | BinaryOp::Or))
            || (matches!(op, BinaryOp::And | BinaryOp::Or) && *inner == BinaryOp::Coalesce))
        };
        if (mixed(left) && !self.parenthesized_tokens(left_start, right_start - 1))
            || (mixed(right) && !self.parenthesized_tokens(right_start, self.i))
        {
            return Err(Diagnostic::new(
                "syntax",
                "Parenthesize nullish coalescing when mixing it with && or ||.",
                operator,
            ));
        }
        Ok(())
    }

    fn conditional_tail(&mut self, predicate: Expr) -> Result<Expr, Diagnostic> {
        self.expect("?")?;
        let consequent = self.precedence(0)?;
        self.expect(":")?;
        let alternate = self.precedence(0)?;
        let expression = Self::conditional_expression(predicate, consequent, alternate);
        if expression.depth > 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Expression nesting exceeds the prototype limit of 128.",
                expression.span,
            ));
        }
        Ok(expression)
    }
    fn conditional_expression(predicate: Expr, consequent: Expr, alternate: Expr) -> Expr {
        let span = Span::new(predicate.span.start, alternate.span.end);
        Expr::new(
            ExprKind::Conditional {
                predicate: Box::new(predicate),
                consequent: Box::new(consequent),
                alternate: Box::new(alternate),
            },
            span,
        )
    }

    // Construct only after recursive RHS parsing has returned. Keeping both
    // ExprKind temporaries out of binary() bounds the recursive parser frame.
    fn binary_expression(
        left: Expr,
        right: Expr,
        op: BinaryOp,
        operator: Span,
        assignment: bool,
    ) -> Expr {
        let span = Span::new(left.span.start, right.span.end);
        Expr::new(
            if assignment && op == BinaryOp::Equal {
                ExprKind::Assign {
                    target: Box::new(left),
                    value: Box::new(right),
                    operator,
                }
            } else if assignment && op == BinaryOp::Coalesce {
                ExprKind::CoalesceAssign {
                    target: Box::new(left),
                    value: Box::new(right),
                    operator,
                    branch: NullishBranch::Dynamic,
                }
            } else if assignment && matches!(op, BinaryOp::And | BinaryOp::Or) {
                ExprKind::LogicalAssign {
                    target: Box::new(left),
                    value: Box::new(right),
                    and: op == BinaryOp::And,
                    operator,
                }
            } else if assignment {
                ExprKind::CompoundAssign {
                    target: Box::new(left),
                    value: Box::new(right),
                    op,
                    operator,
                }
            } else {
                ExprKind::Binary {
                    op,
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                }
            },
            span,
        )
    }

    fn unary(&mut self) -> Result<Expr, Diagnostic> {
        let mut operators = Vec::new();
        while self.is("!")
            || self.is("typeof")
            || self.is("void")
            || self.is("~")
            || self.is("+")
            || self.is("++")
            || self.is("--")
            || (self.is("-")
                && self
                    .tokens
                    .get(self.i + 1)
                    .is_some_and(|t| t.kind != TokenKind::Integer))
        {
            let token = self.take();
            if operators.len() >= 128 {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Expression nesting exceeds the prototype limit of 128.",
                    token.span,
                ));
            }
            operators.push(token);
        }
        let value = self.primary()?;
        Self::apply_unary_operators(value, operators)
    }

    // Keep operator construction off the recursive primary parsing stack.
    fn apply_unary_operators(mut value: Expr, operators: Vec<Token>) -> Result<Expr, Diagnostic> {
        for token in operators.into_iter().rev() {
            let span = Span::new(token.span.start, value.span.end);
            value = Expr::new(
                if token.text == "++" || token.text == "--" {
                    ExprKind::Update {
                        target: Box::new(value),
                        increment: token.text == "++",
                        prefix: true,
                        operator: token.span,
                    }
                } else {
                    ExprKind::Unary {
                        op: if token.text == "typeof" {
                            UnaryOp::Typeof
                        } else if token.text == "void" {
                            UnaryOp::Void
                        } else if token.text == "!" {
                            UnaryOp::Not
                        } else if token.text == "~" {
                            UnaryOp::BitNot
                        } else if token.text == "+" {
                            UnaryOp::Positive
                        } else {
                            UnaryOp::Negate
                        },
                        operator: token.span,
                        value: Box::new(value),
                    }
                },
                span,
            );
            if value.depth > 128 {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Expression nesting exceeds the prototype limit of 128.",
                    span,
                ));
            }
        }
        Ok(value)
    }

    fn atom(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.take();
        let expr = match token.kind {
            TokenKind::Symbol if token.text == "-" && self.peek().kind == TokenKind::Integer => {
                let digits = self.take();
                let span = Span::new(token.span.start, digits.span.end);
                Expr::new(ExprKind::Integer(format!("-{}", numeric_literals::integer_spelling(&digits.text))), span)
            }
            TokenKind::String => Expr::new(ExprKind::String(token.text), token.span),
            TokenKind::Name if matches!(token.text.as_str(), "true" | "false") => {
                Expr::new(ExprKind::Boolean(token.text == "true"), token.span)
            }
            TokenKind::Name if token.text == "null" => Expr::new(ExprKind::Null, token.span),
            TokenKind::Integer => Expr::new(ExprKind::Integer(numeric_literals::integer_spelling(&token.text)), token.span),
            TokenKind::Number => {
                let value = token.text.replace('_', "").parse::<f64>().map_err(|_| {
                    Diagnostic::new("syntax", "Invalid decimal number literal.", token.span)
                })?;
                Expr::new(ExprKind::Number(value), token.span)
            }
            TokenKind::Name if token.text == "new" => return self.set_constructor(token),
            TokenKind::Name => {
                if ["if", "else", "while", "for", "let", "async", "await", "new"]
                    .contains(&token.text.as_str())
                {
                    return Err(Diagnostic::new(
                        "unsupported",
                        format!("{:?} is outside this milestone.", token.text),
                        token.span,
                    ));
                }
                // A statement keyword cannot stand where a value is expected.
                if ["return", "break", "continue", "const", "throw", "try", "catch", "finally", "do", "switch", "case"]
                    .contains(&token.text.as_str())
                {
                    return Err(Diagnostic::new(
                        "syntax",
                        format!("Expected an expression, found the keyword {:?}.", token.text),
                        token.span,
                    ));
                }
                Expr::new(ExprKind::Name(token.text), token.span)
            }
            _ => {
                return Err(Diagnostic::new(
                    "syntax",
                    "Expected an expression.",
                    token.span,
                ));
            }
        };
        Ok(expr)
    }

    fn record_literal(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.expect("{")?;
        let mut fields = Vec::new();
        if !self.is("}") {
            loop {
                if self.is("...") {
                    let mut key=self.take();key.text="record#spread".into();
                    fields.push((key,self.expression()?));
                }else{
                    let key = self.name()?;
                    // `{ x }` is shorthand for `{ x: x }`.
                    if self.is(",") || self.is("}") {
                        let value = Expr::new(ExprKind::Name(key.text.clone()), key.span);
                        fields.push((key, value));
                    } else {
                        self.expect(":")?;
                        fields.push((key, self.expression()?));
                    }
                }
                if !self.accept(",") || self.is("}") {
                    break;
                }
            }
        }
        let end = self.expect("}")?;
        Ok(Expr::new(
            ExprKind::Record(fields),
            Span::new(token.span.start, end.span.end),
        ))
    }

    fn closure_literal(&mut self) -> Result<Expr, Diagnostic> {
        // Parameter defaults can contain another arrow. Its pattern prologue
        // belongs to that arrow, not to the enclosing parameter list.
        let outer_patterns=std::mem::take(&mut self.parameter_patterns);
        let start = self.peek().span;
        // Boxed: nested function bodies recurse through this frame.
        let header = self.closure_signature(start)?;
        let id = self.next_closure;
        self.next_closure += 1;
        let patterns = std::mem::take(&mut self.parameter_patterns);
        self.parameter_patterns=outer_patterns;
        let mut body = self.closure_body()?;
        Self::open_with_patterns(patterns, &mut body);
        let end = self.tokens[self.i - 1].span.end;
        Ok(Self::closure_expression(id, header, body, Span::new(start.start, end)))
    }

    /// Parameters, result annotation and the arrow of an arrow function.
    #[inline(never)]
    fn closure_signature(&mut self, start: Span) -> Result<Box<(Vec<CallableParameter>, ResultAnnotation)>, Diagnostic> {
        let params = if self.accept("(") { self.callable_parameters(true)? } else {
            let name = self.name()?;
            vec![CallableParameter { rest: false, ty: ParameterAnnotation::Inferred(name.clone()), name, access: None, initializer: ParameterInitializer::Required }]
        };
        let result = if self.accept(":") { self.result_annotation()? } else {
            ResultAnnotation::Inferred(start)
        };
        self.expect("=>")?;
        Ok(Box::new((params, result)))
    }

    /// A block body, or an expression body as its return statement.
    #[inline(never)]
    fn closure_body(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        if self.is("{") {
            return self.block();
        }
        // A named function expression in the body is declared inside the arrow,
        // where the arrow parameters are visible.
        let outer = std::mem::take(&mut self.pending);
        let value = self.expression();
        let mut body = std::mem::replace(&mut self.pending, outer);
        body.append(&mut Self::returned(Box::new(value?)));
        Ok(body)
    }

    #[inline(never)]
    fn returned(value: Box<Expr>) -> Vec<Stmt> {
        let span = value.span;
        vec![Stmt { labels: Vec::new(), control_target: None, kind: StmtKind::Return(Some(*value)), span }]
    }

    #[inline(never)]
    fn closure_expression(id: usize, header: Box<(Vec<CallableParameter>, ResultAnnotation)>, body: Vec<Stmt>, span: Span) -> Expr {
        let (params, result) = *header;
        Expr::new(ExprKind::Closure { id, params, result, body }, span)
    }

    // Keep array recursion outside the larger record parser frame.
    fn array_literal(&mut self) -> Result<Expr, Diagnostic> {
        let start = self.expect("[")?;
        let mut values = Vec::new();
        if !self.is("]") {
            loop {
                values.push(if self.accept("...") { self.spread_argument()? } else { self.expression()? });
                if !self.accept(",") || self.is("]") {
                    break;
                }
            }
        }
        let end = self.expect("]")?;
        Ok(Expr::new(
            ExprKind::Array(values),
            Span::new(start.span.start, end.span.end),
        ))
    }

    // Keep recursive interpolation parsing in a separate small frame.
    fn template_literal(&mut self) -> Result<Expr, Diagnostic> {
        let head = self.take();
        let start = head.span.start;
        let mut parts = vec![Expr::new(ExprKind::String(head.text), head.span)];
        loop {
            parts.push(self.comma_expression()?);
            let chunk = self.take();
            if !matches!(
                chunk.kind,
                TokenKind::TemplateMiddle | TokenKind::TemplateTail
            ) {
                return Err(Diagnostic::new(
                    "syntax",
                    "Expected a template interpolation boundary.",
                    chunk.span,
                ));
            }
            let tail = chunk.kind == TokenKind::TemplateTail;
            let end = chunk.span.end;
            parts.push(Expr::new(ExprKind::String(chunk.text), chunk.span));
            if tail {
                let result = Expr::new(ExprKind::Template(parts), Span::new(start, end));
                if result.depth > 128 {
                    return Err(Diagnostic::new(
                        "unsupported",
                        "Expression nesting exceeds the prototype limit of 128.",
                        result.span,
                    ));
                }
                return Ok(result);
            }
        }
    }

    // Keep the template Result/Expr temporaries out of ordinary recursive
    // primary parsing, including hostile indexed expressions rejected at depth 128.
    fn template_primary(&mut self) -> Result<Expr, Diagnostic> {
        let expression = self.template_literal()?;
        self.postfix(expression)
    }

    // Keep the unparenthesized typed-arrow diagnostic temporaries out of the
    // recursively retained primary frame.
    #[inline(never)]
    fn typed_arrow_without_parentheses(&self) -> Result<(), Diagnostic> {
        if self.peek().kind == TokenKind::Name && self.tokens.get(self.i + 1).is_some_and(|token| token.text == ":") {
            let mut cursor = self.i + 2;
            let mut closing = None;
            while let Some(token) = self.tokens.get(cursor) {
                if matches!(token.text.as_str(), ";" | "{" | "}" | "(" | "?" | "=") { break; }
                if token.text == ")" { closing = Some(cursor); }
                if token.text == "=>" {
                    if closing.is_some() {
                        let parameter = self.peek();
                        return Err(Diagnostic::new("syntax", format!("Typed arrow parameters need parentheses. Put the parameter list in parentheses, for example `({}: Type) => ...`; the opening `(` is missing before `{}`.", parameter.text, parameter.text), self.tokens[self.i + 1].span)
                            .related(parameter.span, "Start the arrow parameter list here.")
                            .fix("Insert opening parenthesis for arrow parameter", Span::new(parameter.span.start, parameter.span.start), "("));
                    }
                    break;
                }
                cursor += 1;
            }
        }
        Ok(())
    }

    fn primary(&mut self) -> Result<Expr, Diagnostic> {
        self.typed_arrow_without_parentheses()?;
        if self.peek().kind == TokenKind::TemplateHead {
            return self.template_primary();
        }
        // Keep recursive literals and grouping outside the larger atom frame.
        let single_arrow = self.peek().kind == TokenKind::Name && self.tokens.get(self.i + 1).is_some_and(|token| token.text == "=>");
        if single_arrow { return self.closure_primary(); }
        if self.is("function") {
            return self.function_primary();
        }
        let arrow = self.is("(") && self.closure_header();
        if self.is("(") && !arrow {
            return self.grouped_primary();
        }
        if self.is("(") {
            return self.closure_primary();
        }
        if self.is("{") {
            return self.record_primary();
        }
        if self.is("[") {
            return self.array_primary();
        }
        self.atom_primary()
    }

    // Parse exactly one result type. Scanning for a later arrow mistakes a
    // conditional arm's separating colon for an outer closure annotation.
    fn annotated_arrow_follows(&self, start: usize) -> bool {
        let mut tokens = self.tokens.clone();
        match crate::type_syntax::parse_prefix(&mut tokens, start, 0) {
            Ok((_, end)) => tokens.get(end).is_some_and(|token| token.text == "=>"),
            // A type past the nesting limit is still a type; the arrow parser reports it.
            Err(error) => error.code == "unsupported",
        }
    }

    fn closure_header(&self) -> bool {
        let mut nesting = 0;
        for (position, token) in self.tokens.iter().enumerate().skip(self.i) {
            match token.text.as_str() {
                "(" => nesting += 1,
                ")" => { nesting -= 1; if nesting == 0 {return match self.tokens.get(position + 1).map(|token| token.text.as_str()) {
                    Some("=>") => true,
                    Some(":") => self.annotated_arrow_follows(position + 2),
                    _ => false,
                };} },
                _ => {},
            }
        }
        false
    }

    fn grouped_primary(&mut self) -> Result<Expr, Diagnostic> {
        self.take();
        let expr = self.comma_expression()?;
        self.expect(")")?;
        self.postfix(expr)
    }

    /// `function (a: int): int { ... }` as a value: an anonymous function
    /// with the parameters, result and block body of an arrow function.
    #[inline(never)]
    fn function_primary(&mut self) -> Result<Expr, Diagnostic> {
        if self.tokens.get(self.i + 1).is_some_and(|token| token.kind == TokenKind::Name && token.text != "(") {
            let expr = self.named_function_expression()?;
            return self.postfix(expr);
        }
        let token = self.take();
        self.expect("(")?;
        let params = self.callable_parameters(true)?;
        let result = if self.accept(":") { self.result_annotation()? } else { ResultAnnotation::Inferred(token.span) };
        if !self.is("{") {
            return Err(Diagnostic::new(
                "syntax",
                "A function expression needs a block body; use an arrow function for an expression body.",
                self.peek().span,
            ));
        }
        let id = self.next_closure;
        self.next_closure += 1;
        let patterns = std::mem::take(&mut self.parameter_patterns);
        let mut body = self.block()?;
        Self::open_with_patterns(patterns, &mut body);
        let end = self.tokens[self.i - 1].span.end;
        let expr = Expr::new(ExprKind::Closure { id, params, result, body }, Span::new(token.span.start, end));
        self.postfix(expr)
    }

    fn closure_primary(&mut self) -> Result<Expr, Diagnostic> {
        let expr = self.closure_literal()?;
        self.postfix(expr)
    }

    fn record_primary(&mut self) -> Result<Expr, Diagnostic> {
        let expr = self.record_literal()?;
        self.postfix(expr)
    }

    fn array_primary(&mut self) -> Result<Expr, Diagnostic> {
        let expr = self.array_literal()?;
        self.postfix(expr)
    }

    fn atom_primary(&mut self) -> Result<Expr, Diagnostic> {
        let expr = self.atom()?;
        self.postfix(expr)
    }

    fn postfix_update(&mut self, expr: Expr) -> Result<Expr, Diagnostic> {
        if !self.is("++") && !self.is("--") {
            return Ok(expr);
        }
        let previous_end = self.tokens[self.i - 1].span.end;
        let token = self.peek();
        if self.source[previous_end..token.span.start]
            .contains(['\n', '\r', '\u{2028}', '\u{2029}'])
        {
            return Err(Diagnostic::new(
                "syntax",
                "A postfix update cannot follow a line terminator.",
                token.span,
            ));
        }
        let token = self.take();
        let span = Span::new(expr.span.start, token.span.end);
        let result = Expr::new(
            ExprKind::Update {
                target: Box::new(expr),
                increment: token.text == "++",
                prefix: false,
                operator: token.span,
            },
            span,
        );
        if result.depth > 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Expression nesting exceeds the prototype limit of 128.",
                span,
            ));
        }
        Ok(result)
    }

    // Keep call argument temporaries out of the recursively retained index frame.
    fn call_arguments(&mut self) -> Result<(Vec<Expr>, Token), Diagnostic> {
        let mut args = Vec::new();
        if !self.is(")") {
            loop {
                self.call_argument(&mut args)?;
                if !self.accept(",") {
                    break;
                }
            }
        }
        Ok((args, self.expect(")")?))
    }

    // One argument's Expr temporaries live only while that argument parses.
    #[inline(never)]
    fn call_argument(&mut self, args: &mut Vec<Expr>) -> Result<(), Diagnostic> {
        if self.accept("...") {
            return self.spread_call_argument(args);
        }
        args.push(self.expression()?);
        Ok(())
    }

    #[inline(never)]
    fn spread_call_argument(&mut self, args: &mut Vec<Expr>) -> Result<(), Diagnostic> {
        args.push(self.spread_argument()?);
        Ok(())
    }

    // Spread wrapper temporaries stay out of the ordinary argument frame.
    #[inline(never)]
    fn spread_argument(&mut self) -> Result<Expr, Diagnostic> {
        let operator = self.tokens[self.i - 1].span;
        let value = self.expression()?;
        let span = Span::new(operator.start, value.span.end);
        Ok(Expr::new(ExprKind::Unary {
            op: UnaryOp::Spread,
            operator,
            value: Box::new(value),
        }, span))
    }

    // The indexed-expression parser recursively retains postfix frames. Keep
    // the template-only predicate and diagnostic construction outside them.
    fn is_tagged_template(&self) -> bool {
        self.peek().kind == TokenKind::TemplateHead
            || (self.peek().kind == TokenKind::String
                && self.source[self.peek().span.start..].starts_with('`'))
    }

    fn reject_tagged_template(&self) -> Result<Expr, Diagnostic> {
        Err(Diagnostic::new(
            "unsupported",
            "Tagged templates remain unsupported.",
            self.peek().span,
        ))
    }

    fn has_explicit_call_arguments(&self) -> bool {
        if !self.is("<") {
            return false;
        }
        let mut depth = 0usize;
        for (index, token) in self.tokens.iter().enumerate().skip(self.i) {
            match token.text.as_str() {
                "<" => depth += 1,
                ">" | ">>" | ">>>" => {
                    let count = token.text.len();
                    if count > depth {
                        return false;
                    }
                    depth -= count;
                    if depth == 0 {
                        return self
                            .tokens
                            .get(index + 1)
                            .is_some_and(|token| token.text == "(");
                    }
                }
                ";" | "{" | "}" => return false,
                _ => {}
            }
        }
        false
    }
    fn explicit_call_arguments(
        &mut self,
    ) -> Result<Vec<crate::type_syntax::ValueAnnotation>, Diagnostic> {
        self.expect("<")?;
        let mut arguments = Vec::new();
        loop {
            arguments.push(self.value_annotation()?);
            if arguments.len() > 128 {
                return Err(Diagnostic::new(
                    "generic-limit",
                    "Explicit type argument count exceeds 128.",
                    self.peek().span,
                ));
            }
            if self.accept(",") {
                continue;
            }
            crate::type_syntax::split_closer(&mut self.tokens, self.i);
            self.expect(">")?;
            return Ok(arguments);
        }
    }

    fn optional_identity(mut expr: &Expr) -> &Expr {
        while let ExprKind::Unary {
            op: UnaryOp::OptionalBoundary,
            value,
            ..
        } = &expr.kind
        {
            expr = value;
        }
        expr
    }
    fn is_array_of(expr: &Expr) -> bool {
        matches!(&Self::optional_identity(expr).kind, ExprKind::Field(base, method) if method == "of" && matches!(&Self::optional_identity(base).kind, ExprKind::Name(name) if name == "Array"))
    }

    #[inline(never)]
    fn call_expression(&mut self, expr: Expr) -> Result<Expr, Diagnostic> {
        let expr = Box::new(expr);
        let (args, end) = self.call_arguments()?;
        Self::finish_call_expression(expr, args, end)
    }

    // Construction temporaries must not remain on the recursive argument stack.
    #[inline(never)]
    fn finish_call_expression(expr: Box<Expr>, args: Vec<Expr>, end: Token) -> Result<Expr, Diagnostic> {
        let span = Span::new(expr.span.start, end.span.end);
        if Self::is_array_of(&expr) {
            Ok(Expr::new(ExprKind::ArrayOf(args, None), span))
        } else {
            Ok(Expr::new(ExprKind::Call(expr, args), span))
        }
    }

    fn explicit_call_expression(&mut self, expr: Expr) -> Result<Expr, Diagnostic> {
        let arguments = self.explicit_call_arguments()?;
        self.expect("(")?;
        let (args, end) = self.call_arguments()?;
        let span = Span::new(expr.span.start, end.span.end);
        if Self::is_array_of(&expr) {
            if arguments.len() != 1 {
                return Err(Diagnostic::new(
                    "arity",
                    "Array.of accepts exactly one explicit element type.",
                    span,
                ));
            }
            return Ok(Expr::new(
                ExprKind::ArrayOf(args, Some(Box::new(arguments.into_iter().next().unwrap()))),
                span,
            ));
        }
        if matches!(Self::optional_identity(&expr).kind, ExprKind::Field(_, _)) {
            return Err(Diagnostic::new(
                "unsupported",
                "Explicit member type arguments are only supported for Array.of.",
                span,
            ));
        }
        let call = Expr::new(ExprKind::Call(Box::new(expr), args), span);
        self.call_type_arguments.push((span, arguments));
        Ok(call)
    }
    fn postfix_call(&mut self, expr: Expr) -> Result<Expr, Diagnostic> {
        if self.accept("(") {
            self.call_expression(expr)
        } else {
            self.explicit_call_expression(expr)
        }
    }

    fn optional_postfix(&mut self, expr: Expr, operator: Span) -> Result<Expr, Diagnostic> {
        let span = expr.span;
        let boundary = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::OptionalBoundary,
                operator,
                value: Box::new(expr),
            },
            span,
        );
        if self.accept("[") {
            let index = self.comma_expression()?;
            let end = self.expect("]")?;
            Ok(Expr::new(
                ExprKind::Index(Box::new(boundary), Box::new(index)),
                Span::new(span.start, end.span.end),
            ))
        } else if self.is("(") || self.has_explicit_call_arguments() {
            self.postfix_call(boundary)
        } else {
            let name = self.name()?;
            let end = name.span.end;
            Ok(Expr::new(
                ExprKind::Field(Box::new(boundary), name.text),
                Span::new(span.start, end),
            ))
        }
    }

    fn field_expression(&mut self, base: Expr) -> Result<Expr, Diagnostic> {
        let name = self.name()?;
        let span = Span::new(base.span.start, name.span.end);
        Ok(Expr::new(ExprKind::Field(Box::new(base), name.text), span))
    }

    fn finish_postfix(&mut self, mut expr: Expr, optional: bool) -> Result<Expr, Diagnostic> {
        if optional {
            let span = expr.span;
            expr = Expr::new(ExprKind::OptionalChain(Box::new(expr)), span);
        }
        self.postfix_update(expr)
    }

    // Assertion wrapper temporaries stay out of the recursively retained postfix frame.
    #[inline(never)]
    fn non_null_postfix(&mut self, mut expr: Expr, optional: bool) -> Expr {
        let operator = self.tokens[self.i - 1].span;
        if optional {
            let span = expr.span;
            expr = Expr::new(ExprKind::OptionalChain(Box::new(expr)), span);
        }
        let span = Span::new(expr.span.start, operator.end);
        Expr::new(ExprKind::Unary { op: UnaryOp::NonNull, operator, value: Box::new(expr) }, span)
    }

    // Each postfix operator owns its Expr temporaries in a dedicated step.
    // The recursively retained postfix frame keeps one slot only.
    #[inline(never)]
    fn optional_step(&mut self, slot: &mut Option<Expr>) -> Result<(), Diagnostic> {
        let operator = self.tokens[self.i - 1].span;
        let expr = slot.take().expect("postfix operand");
        *slot = Some(self.optional_postfix(expr, operator)?);
        Ok(())
    }

    #[inline(never)]
    fn non_null_step(&mut self, slot: &mut Option<Expr>, optional: bool) {
        let expr = slot.take().expect("postfix operand");
        *slot = Some(self.non_null_postfix(expr, optional));
    }

    #[inline(never)]
    fn field_step(&mut self, slot: &mut Option<Expr>) -> Result<(), Diagnostic> {
        let expr = slot.take().expect("postfix operand");
        *slot = Some(self.field_expression(expr)?);
        Ok(())
    }

    // The nested index is parsed while the base stays in the caller's slot,
    // so this retained frame holds one expression, not the base as well.
    #[inline(never)]
    fn index_step(&mut self, slot: &mut Option<Expr>) -> Result<(), Diagnostic> {
        let index = Box::new(self.comma_expression()?);
        self.finish_index(slot, index)
    }

    #[inline(never)]
    fn finish_index(&mut self, slot: &mut Option<Expr>, index: Box<Expr>) -> Result<(), Diagnostic> {
        let end = self.expect("]")?;
        let base = Box::new(slot.take().expect("postfix operand"));
        let span = Span::new(base.span.start, end.span.end);
        *slot = Some(Expr::new(ExprKind::Index(base, index), span));
        Ok(())
    }

    #[inline(never)]
    fn call_step(&mut self, slot: &mut Option<Expr>) -> Result<(), Diagnostic> {
        if !self.accept("(") {
            return self.explicit_call_step(slot);
        }
        let base = Self::take_call_operand(slot);
        let (arguments, end) = self.call_arguments()?;
        Self::finish_call_slot(slot, base, arguments, end)
    }

    // The existing boxed call operand is prepared before argument recursion.
    #[inline(never)]
    fn take_call_operand(slot: &mut Option<Expr>) -> Box<Expr> {
        Box::new(slot.take().expect("postfix operand"))
    }

    #[inline(never)]
    fn finish_call_slot(slot: &mut Option<Expr>, base: Box<Expr>, arguments: Vec<Expr>, end: Token) -> Result<(), Diagnostic> {
        *slot = Some(Self::finish_call_expression(base, arguments, end)?);
        Ok(())
    }

    // Explicit type argument construction stays outside ordinary call frames.
    #[inline(never)]
    fn explicit_call_step(&mut self, slot: &mut Option<Expr>) -> Result<(), Diagnostic> {
        let expr = slot.take().expect("postfix operand");
        *slot = Some(self.explicit_call_expression(expr)?);
        Ok(())
    }

    #[inline(never)]
    fn postfix_depth(expr: &Expr) -> Result<(), Diagnostic> {
        if expr.depth > 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Expression nesting exceeds the prototype limit of 128.",
                expr.span,
            ));
        }
        Ok(())
    }

    #[inline(never)]
    fn finish_postfix_slot(&mut self, slot: &mut Option<Expr>, optional: bool) -> Result<Expr, Diagnostic> {
        self.finish_postfix(slot.take().expect("postfix operand"), optional)
    }

    fn postfix(&mut self, expr: Expr) -> Result<Expr, Diagnostic> {
        let mut optional = false;
        let mut slot = Some(expr);
        loop {
            Self::postfix_depth(slot.as_ref().expect("postfix operand"))?;
            if self.is_tagged_template() {
                return self.reject_tagged_template();
            }
            if self.accept("?.") {
                optional = true;
                self.optional_step(&mut slot)?;
            } else if self.accept("!") {
                self.non_null_step(&mut slot, optional);
                optional = false;
            } else if self.accept(".") {
                self.field_step(&mut slot)?;
            } else if self.accept("[") {
                self.index_step(&mut slot)?;
            } else if self.is("(")
                || (matches!(slot.as_ref().expect("postfix operand").kind, ExprKind::Name(_) | ExprKind::Field(_, _) | ExprKind::Sequence(_) | ExprKind::Assign { .. } | ExprKind::Conditional { .. } | ExprKind::Call(..) | ExprKind::Index(..) | ExprKind::Unary { op: UnaryOp::OptionalBoundary | UnaryOp::OptionalSelected | UnaryOp::NonNull, .. })
                    && self.has_explicit_call_arguments())
            {
                self.call_step(&mut slot)?;
            } else {
                return self.finish_postfix_slot(&mut slot, optional);
            }
        }
    }
}

#[cfg(test)]
#[test]
fn checked_array_leaf_conversion_preserves_identity_and_rank_limit() {
    for leaf in [
        Type::Int,
        Type::Number,
        Type::Boolean,
        Type::Null,
        Type::Undefined,
        Type::String,
        Type::Record(17),
        Type::StoredCallable(23),
    ] {
        let mut element = ArrayElement::from_value_type(leaf).unwrap();
        assert_eq!(element.ty(), leaf);
        for depth in 1..=127 {
            element = ArrayElement::from_value_type(Type::Array(element)).unwrap();
            assert_eq!(element.depth(), depth);
            assert_eq!(element.with_depth(0).ty(), leaf);
        }
        assert!(ArrayElement::from_value_type(Type::Array(element)).is_none());
    }
    for unsupported in [
        Type::Void,
        Type::Closure(0),
        Type::Callback(0),
        Type::Callable(0),
    ] {
        assert!(ArrayElement::from_value_type(unsupported).is_none());
    }
}

#[cfg(test)]
#[test]
fn shared_read_callbacks_own_their_environment_and_only_lend_reusable_invocation() {
    let kind=CallbackKind::SharedRead;
    assert!(kind.is_owned());assert!(!kind.is_once());assert_eq!(kind.invocation(),crate::Mode::Read);
    for target in [CallbackKind::Read,CallbackKind::Reusable,CallbackKind::Owned] {assert!(kind.can_pass_to(target));}
    assert!(!kind.can_pass_to(CallbackKind::Once));assert!(!kind.can_pass_to(CallbackKind::OwnedOnce));
    assert!(!CallbackKind::Read.can_pass_to(kind));assert!(!CallbackKind::Owned.can_pass_to(kind));
}

#[cfg(test)]
mod exception_syntax_tests {
    use super::*;
    #[test]
    fn exception_regions_and_completion_fixtures_parse() {
        for source in [include_str!("../tests/fixtures/exception-handling/basic.tsr"),include_str!("../tests/fixtures/exception-handling/completions.tsr")] {
            parse(source.split_once('\n').expect("fixture import line").1).unwrap();
        }
    }
    #[test]
    fn exception_regions_preserve_optional_handlers_and_finally() {
        let program=parse("function main(){try{throw 1;}catch(error){}finally{}try{}catch{}try{}finally{}}").unwrap();
        let body=&program.functions[0].body;
        let StmtKind::Try(region)=&body[0].kind else{panic!("try region");};
        assert!(matches!(region.body[0].kind,StmtKind::Throw(_)));
        assert_eq!(region.catch.as_ref().unwrap().name.as_ref().unwrap().text,"error");
        assert!(region.finally.as_ref().unwrap().is_empty());
        let StmtKind::Try(region)=&body[1].kind else{panic!("catch region");};
        assert!(region.catch.as_ref().unwrap().name.is_none());assert!(region.finally.is_none());
        let StmtKind::Try(region)=&body[2].kind else{panic!("finally region");};
        assert!(region.catch.is_none());assert!(region.finally.is_some());
    }
    #[test]
    fn throw_line_break_and_exception_depth_are_bounded() {
        assert_eq!(parse("function main(){throw\n1;}").unwrap_err().code,"syntax");
        let source=format!("function main(){{{}throw 1;{}}}","try{".repeat(200),"}finally{}".repeat(200));
        assert_eq!(parse(&source).unwrap_err().code,"unsupported");
    }
    #[test]
    fn try_requires_braces_and_a_handler_or_finally() {
        for source in ["function main(){try{}}","function main(){try console.log(1);catch{}}","function main(){try{}catch(error)console.log(error);}","function main(){try{}finally console.log(1);}"] {
            assert_eq!(parse(source).unwrap_err().code,"syntax", "{source}");
        }
    }
}

#[cfg(test)]
#[test]
fn hostile_default_call_nesting_returns_a_source_error() {
    let source=format!("function id(value:int):int{{return value;}}function use(value:int={}1{}):int{{return value;}}function main(){{use();}}","id(".repeat(200),")".repeat(200));
    let error=parse(&source).err().expect("nesting error");
    assert_eq!(error.code,"unsupported");
}

#[path = "enum_syntax.rs"]
mod enum_syntax;

#[path = "set_syntax.rs"]
mod set_syntax;

/// Returns the validated numeric literal range at a UTF-8 source offset.
/// Incomplete or invalid numeric spellings have no range.
pub fn numeric_literal_span(source:&str,start:usize)->Option<Span>{
    let bytes=source.as_bytes();let first=*bytes.get(start)?;
    if !(first.is_ascii_digit()||first==b'.'&&bytes.get(start+1).is_some_and(u8::is_ascii_digit)){return None;}
    numeric_literals::scan(source,start).ok().map(|(token,_)|token.span)
}

#[cfg(test)]
#[test]
fn explicit_optional_calls_retain_the_optional_receiver_boundary() {
    let program = parse("function main(){const result=action?.<int>(1);}").unwrap();
    let StmtKind::Let { value, .. } = &program.functions[0].body[0].kind else { panic!("initializer") };
    let call = match &value.kind { ExprKind::OptionalChain(inner) => inner.as_ref(), _ => value };
    let ExprKind::Call(callee, arguments) = &call.kind else { panic!("call") };
    assert_eq!(arguments.len(), 1);
    assert!(matches!(callee.kind, ExprKind::Unary { op: UnaryOp::OptionalBoundary, .. }));
    assert_eq!(program.call_type_arguments.len(), 1);
    assert_eq!(program.call_type_arguments[0].0, call.span);
}

#[cfg(test)]
#[test]
fn nested_call_arguments_keep_normal_parser_stack() {
    for depth in [100, 200] {
        let source = format!(
            "function id(value:int):int{{return value;}}function main(){{const action:(Function<int, [value:int]>)|null=id;if(action!==null){{console.log(action({}1{}));}}}}",
            "id(".repeat(depth), ")".repeat(depth)
        );
        let result = parse(&source);
        if depth == 100 {
            assert!(result.is_ok(), "100 nested call arguments must parse");
            let formatted = crate::format(&source).expect("deep source formats");
            assert_eq!(crate::format(&formatted).unwrap(), formatted);
            assert!(crate::check(&formatted).is_ok(), "formatted nested calls remain valid");
        } else {
            let error = result.err().expect("source nesting remains bounded");
            assert_eq!(error.code, "unsupported");
            assert!(error.message.contains("128"));
        }
    }
}
