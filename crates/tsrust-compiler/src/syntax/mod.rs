mod declaration_work;
mod expression_work;
pub(crate) mod jsx;
mod nesting_policy;
#[cfg(test)]
mod nesting_tests;
#[cfg(test)]
mod parity_dump;
#[cfg(test)]
mod parser_probe;
mod scheduler;
mod statement_work;
pub(crate) mod diagnostic_directives;
mod named_function_syntax;
mod async_syntax;
mod annotation_conversion;
mod annotation_storage;
mod tree_storage;
mod tree_clone;
pub(crate) mod numeric_literals;
pub mod match_syntax;
use crate::diagnostic::{Diagnostic, Span};
pub(crate) use nesting_policy::NestingPolicy;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    JsxOpen,
    JsxClose,
    JsxEnd,
    JsxText,
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
    lex_with(source, NestingPolicy::CONSUMERS)
}

/// Lex under an explicit admission policy. Template frames are heap state.
pub(crate) fn lex_with(source: &str, policy: NestingPolicy) -> Result<Vec<Token>, Diagnostic> {
    let mut tokens = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    // Only interpolating templates need a frame; nested braces belong to its expression.
    let mut templates: Vec<(usize, usize)> = Vec::new();
    let mut jsx_modes = Vec::new();
    while i < bytes.len() {
        if !(source[i..].starts_with('}') && templates.last().is_some_and(|(_, braces)| *braces == 0))
            && jsx::lex_step(source, &mut i, &mut tokens, &mut jsx_modes, policy)? { continue; }
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
                if policy.reached(templates.len()) {
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
    Literal(usize, crate::literal_types::LiteralKind),
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
    Literal(usize, crate::literal_types::LiteralKind, u8),
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
            Type::Literal(id, kind) => Self::Literal(id, kind, 0),
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
            | Self::Literal(_, _, depth) | Self::Enum(_, _, depth) | Self::Set(_, depth) | Self::Map(_, depth)
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
            Self::Literal(id, kind, _) => Self::Literal(id, kind, depth),
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
            Self::Literal(id, kind, _) => Type::Literal(id, kind),
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
    /// Identity or a literal widened to its native scalar; never collection covariance.
    pub fn accepts_scalar_value(self, actual: Self) -> bool { self == actual || self == actual.scalar_base() }
    pub fn scalar_base(self)->Self {if let Self::Literal(_,kind)=self{kind.base()}else{self}}
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
        ) || matches!(self, Self::Literal(_,kind) if kind!=crate::literal_types::LiteralKind::String) || matches!(self, Self::Union(id) if id.copy)
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
    /// Checked projection after a stable JSON boolean guard.
    JsonBoolean,
    JsonStringLength,
    JsonStringView,
    JsonDecode,
    JsonEncode,
    SchemaValidate,
    /// Carries the structured export error record identity.
    SchemaExport(usize),
    ResultUnwrap(Type),
    MatchYield(usize),
    SchemaAddIssue(usize),
    /// Forward a scalar constraint through an owned diagnostic-message wrapper.
    SchemaConstraint(usize),
    SchemaMessagePolicy(bool),
    SchemaStrip,
    SchemaRetain,
    SchemaPresence,
    SchemaDefault,
    /// Ordered schema plan identity; output union normalization is independent.
    SchemaChoice(usize),
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
    Await,
    AwaitCallee,
    TaskCreate,
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

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
    pub ty: Option<Type>,
    pub binding: Option<usize>,
    pub target: Option<CallTarget>,
    /// Owning schema factories that this validation may invoke.
    pub(crate) schema_invocations: Option<Box<[usize]>>,
    /// A checked string-category proof for this JSON read, without changing storage.
    pub(crate) json_string_view: bool,
    pub(crate) moved_spread: bool,
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
    /// A value-producing match executes in the enclosing function and lexical context.
    Match { id: usize, body: Vec<Stmt> },
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
            ExprKind::Closure { .. } | ExprKind::Match { .. } => 0,
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
            schema_invocations: None,
            json_string_view: false,
            moved_spread: false,
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

#[derive(Debug)]
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
    pub literal_definition: Option<crate::literal_types::LiteralValue>,
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
    Task,
    /// Immediate registration callback with invocation-time program access.
    ProgramReusable,
    /// Registry-owned environment with invocation-time program access.
    Program,
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
        self == target || (matches!(self, Self::Owned | Self::Reusable) && target == Self::ProgramReusable) || (self == Self::Owned && target == Self::Program) || (self == Self::OwnedOnce && target == Self::Once) || (self == Self::Owned && target == Self::Reusable)
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
        matches!(self, Self::Owned | Self::OwnedOnce | Self::Task | Self::SharedRead | Self::Program)
    }
    pub fn is_once(self) -> bool {
        matches!(self, Self::Once | Self::OwnedOnce | Self::Task)
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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
    pub(crate) diagnostic_first: Option<Span>,
    pub(crate) diagnostic_skips: std::collections::HashSet<usize>,
    /// Diagnostic replay: declared-result functions whose failed return was skipped.
    pub(crate) diagnostic_contracts: std::collections::HashSet<usize>,
    pub(crate) unresolved_local_function: Option<Token>,
    pub(crate) local_declarations: Vec<crate::local_functions::Declaration>,
    pub(crate) local_capture_plans: Vec<crate::local_capture_plan::Plan>,
    pub interface_argument_checks: Vec<(Vec<crate::type_syntax::TypeName>, crate::type_syntax::ValueAnnotation)>,
    pub type_argument_checks: Vec<crate::type_syntax::ValueAnnotation>,
    pub call_type_arguments: Vec<(Span, Vec<crate::type_syntax::ValueAnnotation>)>,
    pub records: Vec<Record>,
    pub functions: Vec<Function>,
    /// Module-owned declarations in initialization order, before ownership lowering.
    pub module_bindings: Vec<Stmt>,
    pub(crate) module_owner_bindings: Vec<Span>,
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
    module_diagnostics(source, false)
}

pub(crate) fn module_diagnostics(source: &str, test_entry: bool) -> Vec<Diagnostic> {
    let errors = std::cell::RefCell::new(Vec::new());
    let result = Parser::new(source, Default::default(), Default::default()).and_then(|mut parser| {
        parser.diagnostics = Some(&errors);
        parser.program(test_entry).map(|_| ())
    });
    if let Err(error) = result { errors.borrow_mut().push(error); }
    let mut errors = errors.into_inner();
    errors.sort_by_key(|error| (error.span.start, error.span.end));
    errors.dedup_by(|a,b| a.span == b.span && a.code == b.code && a.message == b.message);
    errors
}

/// Parses a module whose imports include type aliases of other modules.
pub(crate) fn parse_module(source: &str, test_entry: bool, imported: crate::type_aliases::Aliases, records: crate::type_aliases::ImportedRecords) -> Result<Program, Diagnostic> {
    parse_raw(source, test_entry, imported, records)
}

fn parse_entry(source: &str, test_entry: bool, imported: crate::type_aliases::Aliases) -> Result<Program, Diagnostic> {
    parse_entry_with(source, test_entry, imported, Default::default())
}

fn parse_entry_with(source: &str, test_entry: bool, imported: crate::type_aliases::Aliases, records: crate::type_aliases::ImportedRecords) -> Result<Program, Diagnostic> {
    let mut program = parse_raw(source, test_entry, imported, records)?;
    crate::callables::module_state::lower(&mut program)?;
    crate::local_functions::resolve(&mut program)?;
    Ok(program)
}

fn parse_raw(source: &str, test_entry: bool, imported: crate::type_aliases::Aliases, records: crate::type_aliases::ImportedRecords) -> Result<Program, Diagnostic> {
    Parser::new(source, imported, records)?.program(test_entry)
}

/// Parse one file without nesting limits and without the later lowering
/// passes. The tree may be deeper than recursive consumers accept.
#[allow(dead_code)]
pub(crate) fn parse_unbounded(source: &str) -> Result<Program, Diagnostic> {
    Parser::with_policy(source, Default::default(), Default::default(), NestingPolicy::UNBOUNDED)?.program(false)
}

/// Syntax diagnostics with statement recovery and without nesting limits.
#[allow(dead_code)]
pub(crate) fn diagnostics_unbounded(source: &str) -> Vec<Diagnostic> {
    let errors = std::cell::RefCell::new(Vec::new());
    let result = Parser::with_policy(source, Default::default(), Default::default(), NestingPolicy::UNBOUNDED).and_then(|mut parser| {
        parser.diagnostics = Some(&errors);
        parser.program(false).map(|_| ())
    });
    if let Err(error) = result { errors.borrow_mut().push(error); }
    errors.into_inner()
}

impl<'a> Parser<'a> {
    fn new(source: &'a str, imported: crate::type_aliases::Aliases, records: crate::type_aliases::ImportedRecords) -> Result<Self, Diagnostic> {
        Self::with_policy(source, imported, records, NestingPolicy::CONSUMERS)
    }

    fn with_policy(source: &'a str, imported: crate::type_aliases::Aliases, records: crate::type_aliases::ImportedRecords, policy: NestingPolicy) -> Result<Self, Diagnostic> {
        let tokens = lex_with(source, policy)?;
        let has_await = tokens.iter().any(|token| token.text == "await");
        Ok(Self {
            source,
            diagnostics: None,
            policy,
            scheduling: false,
            parenthesis_closers: Default::default(),
            typed_arrow_clear: None,
            validated_switches: Default::default(),
            tokens,
            has_await,
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
    let mut pending = vec![body.iter()];
    while let Some(statements) = pending.last_mut() {
        let Some(statement) = statements.next() else {
            pending.pop();
            continue;
        };
        match &statement.kind {
            StmtKind::Return(_) => return Some(statement.span),
            StmtKind::Block(body) | StmtKind::While { body, .. } | StmtKind::ForOf { body, .. } => pending.push(body.iter()),
            StmtKind::Loop { body, update, .. } => {
                pending.push(update.iter());
                pending.push(body.iter());
            }
            StmtKind::If { then_body, else_body, .. } => {
                pending.push(else_body.iter());
                pending.push(then_body.iter());
            }
            _ => {}
        }
    }
    None
}

/// One pending target of a destructuring assignment.
enum PatternAssignment {
    Target { target: Expr, value: Expr, depth: usize },
    Element { element: Expr, source: Expr, position: usize, count: usize, depth: usize },
    Field { key: Token, element: Expr, source: Expr, depth: usize },
}

struct Parser<'a> {
    diagnostics: Option<&'a std::cell::RefCell<Vec<Diagnostic>>>,
    /// Admission bound for nested syntax; the scheduler itself needs none.
    policy: NestingPolicy,
    /// One scheduler run is active. Grammar steps queue work instead of
    /// starting another run.
    scheduling: bool,
    /// Source offsets of known parenthesis pairs; none when unclosed.
    parenthesis_closers: std::collections::HashMap<usize, Option<usize>>,
    /// Source range already scanned without finding an unparenthesized typed arrow.
    typed_arrow_clear: Option<(usize, usize)>,
    /// Statements produced by a validated switch, by source offset. Their
    /// bodies hold no break that could leave an enclosing switch.
    validated_switches: std::collections::HashSet<usize>,
    unresolved_local_function: Option<Token>,
    source: &'a str,
    tokens: Vec<Token>,
    has_await: bool,
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
    fn property_name(&mut self) -> Result<Token, Diagnostic> {
        if self.peek().kind == TokenKind::String {
            let token = self.take();
            if token.text.contains('\0') {
                return Err(Diagnostic::new("unsupported", "Record property names cannot contain NUL.", token.span));
            }
            Ok(token)
        } else {
            self.name()
        }
    }
    fn value_annotation(&mut self) -> Result<crate::type_syntax::ValueAnnotation, Diagnostic> {
        let (mut annotation, index) =
            crate::type_syntax::parse_prefix(&mut self.tokens, self.i, self.depth, self.policy)?;
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
                "int" | "number" | "boolean" | "string" | "null" | "undefined" | "void" | "Task" | "Function" | "Mutable" | "ReadOnly" | "Once" | "Owned" | "Set" | "Map" | "Array"
            ) {
                return Err(Diagnostic::new(
                    "duplicate",
                    format!("Type name {:?} is built in and cannot be declared.", name.text),
                    name.span,
                ));
            }
            let (annotation, end) = crate::type_syntax::parse_prefix(&mut self.tokens, index + 3, self.depth, self.policy)?;
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
    fn interface_declaration(&mut self) -> Result<(Record, Vec<(Token, Vec<crate::type_syntax::ValueAnnotation>)>), Diagnostic> {
        let name = self.name()?;
        if self.aliases.contains_key(&name.text) {
            return Err(Diagnostic::new(
                "duplicate",
                format!("Type name {:?} is already declared.", name.text),
                name.span,
            ));
        }
        if matches!(name.text.as_str(), "Task" | "Function" | "Mutable" | "ReadOnly" | "Once" | "Owned") {
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
                let arguments=if self.is("<"){self.explicit_call_arguments()?}else{Vec::new()};
                bases.push((base,arguments));
                if !self.accept(",") {
                    break;
                }
            }
        }
        self.expect("{")?;
        let mut fields = Vec::new();
        while !self.accept("}") {
            let field = self.property_name()?;
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
                enum_definition: None,literal_definition:None,
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
    fn inherit_interface_fields(records: &mut [Record], mut pending: Vec<(Token, Vec<(Token, Vec<crate::type_syntax::ValueAnnotation>)>)>, imported: &crate::type_aliases::ImportedRecords, checks: &mut Vec<(Vec<crate::type_syntax::TypeName>,crate::type_syntax::ValueAnnotation)>) -> Result<Vec<Token>, Diagnostic> {
        let mut unresolved = Vec::new();
        while !pending.is_empty() {
            let waiting = pending.iter().map(|(name, _)| name.text.clone()).collect::<Vec<_>>();
            let ready = pending
                .iter()
                .position(|(_, bases)| bases.iter().all(|(base,_)| !waiting.contains(&base.text)));
            let Some(ready) = ready else {
                let (name, _) = &pending[0];
                return Err(Diagnostic::new(
                    "recursive-interface",
                    format!("Interface {:?} extends itself.", name.text),
                    name.span,
                ));
            };
            let (name, bases) = pending.remove(ready);
            let parameters=records.iter().find(|record|record.name.span==name.span).unwrap().type_parameters.clone();
            let mut inherited: Vec<RecordField> = Vec::new();
            for (base,arguments) in &bases {
                checks.extend(arguments.iter().cloned().map(|argument|(parameters.clone(),argument)));
                let (parameters,fields,external)=if let Some((parameters,fields))=imported.get(&base.text) {
                    (parameters,fields,true)
                }else if let Some(found)=records.iter().find(|record|record.name.text==base.text && record.enum_definition.is_none() && record.map_definition.is_none() && record.set_definition.is_none()) {
                    (&found.type_parameters,&found.fields,false)
                }else{unresolved.push(base.clone());continue;};
                if parameters.len()!=arguments.len(){
                    return Err(Diagnostic::new("type-arguments",format!("Interface base {:?} requires {} type arguments, got {}.",base.text,parameters.len(),arguments.len()),base.span));
                }
                let bindings=parameters.iter().zip(arguments).map(|(parameter,argument)|(parameter.text.clone(),argument.clone())).collect();
                for mut field in fields.iter().cloned(){
                    if external {
                        field.name.span=base.span;
                        crate::type_aliases::respan(&mut field.annotation,base.span);
                    }
                    crate::type_substitution::apply(&mut field.annotation,&bindings)?;
                    inherited.push(field);
                }
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
        let members = match annotation.into_kind() {
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
            diagnostic_first: None,
            diagnostic_skips: Default::default(),
            diagnostic_contracts: Default::default(),
            unresolved_local_function: None,
            local_declarations: Vec::new(),
            local_capture_plans: Vec::new(),
            interface_argument_checks: Vec::new(),
            type_argument_checks: Vec::new(),
            type_aliases: Default::default(),
            unresolved_bases: Vec::new(),
            call_type_arguments: Vec::new(),
            records: Vec::new(),
            functions: Vec::new(),
            module_bindings: Vec::new(),
            module_owner_bindings: Vec::new(),
        };
        self.collect_type_aliases()?;
        let mut interface_bases = Vec::new();
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
                let (_, end) = crate::type_syntax::parse_prefix(&mut self.tokens, self.i, self.depth, self.policy)?;
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
                if matches!(name.text.as_str(), "Task" | "Function" | "Mutable" | "ReadOnly" | "Once" | "Owned") {
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
                    let field = self.property_name()?;
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
                    native_type: None, map_definition: None, set_definition: None,                    enum_definition: None,literal_definition:None,
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
            } else if self.accept("async") {
                self.expect("function")?;
                let mut function=self.named_function()?;
                if function.name.text=="main" {
                    if !function.params.is_empty() { return Err(Diagnostic::new("async-main", "async main takes no arguments.",function.name.span)); }
                    function.name.text="async#entry".into();
                    let span=function.scope_span;
                    let call=Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name(function.name.text.clone()),span)),Vec::new()),span);
                    let value=Expr::new(ExprKind::Unary{op:UnaryOp::Await,value:Box::new(call),operator:function.name.span},span);
                    let mut entry=function.clone(); entry.name.text="main".into(); entry.result=None;
                    entry.body=vec![Stmt{labels:Vec::new(),control_target:None,span,kind:StmtKind::Expr(value)}];
                    program.functions.push(function); program.functions.push(entry);
                } else {program.functions.push(function);}
            } else if self.accept("function") {
                program.functions.push(self.named_function()?);
            } else if !test_entry && matches!(self.peek().text.as_str(), "let" | "const") {
                program.module_bindings.push(self.block_statement()?);
                program.module_bindings.append(&mut self.pending);
            } else if test_entry {
                registration.push(self.block_statement()?);
                registration.append(&mut self.pending);
            } else {
                let statement = self.block_statement()?;
                if !matches!(&statement.kind, StmtKind::Expr(Expr { kind: ExprKind::Call(_, _), .. })) {
                    return Err(Diagnostic::new("unsupported", "Expected a declaration or a module initialization call.", statement.span));
                }
                program.module_bindings.push(statement);
                program.module_bindings.append(&mut self.pending);
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
        program.unresolved_bases = Self::inherit_interface_fields(&mut program.records, interface_bases, &self.imported_records, &mut program.interface_argument_checks)?;
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
        if self.policy.reached(self.depth) {
            return Err(Diagnostic::new(
                "unsupported",
                "Type annotation nesting exceeds the prototype limit of 128.",
                self.peek().span,
            ));
        }
        let (access, mut annotation, next) =
            crate::type_syntax::parse_parameter_prefix(&mut self.tokens, self.i, self.depth, self.policy)?;
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

    fn parameter_order<'p>(initializers: impl Iterator<Item = &'p ParameterInitializer>, names: impl Iterator<Item = Span>) -> Result<(), Diagnostic> {
        let mut optional = false;
        for (initializer, span) in initializers.zip(names) {
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
    // Construct the large statement value after recursive parsing returns.
    #[inline(never)]
    fn conditional_statement(span: Span, condition: Expr, then_body: Vec<Stmt>, else_body: Vec<Stmt>) -> Stmt {
        Stmt { labels: Vec::new(), control_target: None,
            kind: StmtKind::If { condition, then_body, else_body },
            span,
        }
    }
    /// `[a, b] = value;` and `({ x, y: z } = value);` assign to existing
    /// places. The statement becomes ordinary assignments in source order; the
    /// first is returned and the others follow it through `pending`.
    #[inline(never)]
    fn destructuring_assignment(&mut self, expression: Expr) -> Result<StmtKind, Diagnostic> {
        let span = expression.span;
        let ExprKind::Assign { target, value, .. } = expression.into_kind() else {
            unreachable!("destructuring assignment expression")
        };
        let (mut target, mut value) = (*target, *value);
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
        // Targets in source order. Each nested pattern is expanded when its
        // turn comes, so hidden names and errors keep their source order.
        let mut work = Vec::new();
        match (target.take_kind(), value.take_kind()) {
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
                work.extend(targets.into_iter().zip(held).rev().map(|(target, value)| PatternAssignment::Target { target, value, depth: 0 }));
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
                self.assign_pattern_positions(target, source, 0, &mut work)?;
            }
        }
        while let Some(step) = work.pop() {
            let (target, value, depth) = match step {
                PatternAssignment::Target { target, value, depth } => (target, value, depth),
                PatternAssignment::Element { element, source, position, count, depth } => {
                    let at = element.span;
                    if matches!(element.kind, ExprKind::Unary { op: UnaryOp::Spread, .. }) {
                        let ExprKind::Unary { value: rest, .. } = element.into_kind() else { unreachable!() };
                        if position + 1 != count {
                            return Err(Diagnostic::new("syntax", "A rest target must be last in a destructuring assignment.", at));
                        }
                        // Rest explicitly copies the remaining elements.
                        let slice = Expr::new(ExprKind::Field(Box::new(source), "slice".to_owned()), at);
                        let start = Expr::new(ExprKind::Integer(position.to_string()), at);
                        (*rest, Expr::new(ExprKind::Call(Box::new(slice), vec![start]), at), depth + 1)
                    } else {
                        let index = Expr::new(ExprKind::Integer(position.to_string()), at);
                        (element, Expr::new(ExprKind::Index(Box::new(source), Box::new(index)), at), depth + 1)
                    }
                }
                PatternAssignment::Field { key, element, source, depth } => {
                    if key.text == "record#spread" {
                        return Err(Diagnostic::new("unsupported", "A record rest target is not supported in a destructuring assignment; declare it with const or let.", key.span));
                    }
                    (element, Expr::new(ExprKind::Field(Box::new(source), key.text.clone()), key.span), depth + 1)
                }
            };
            // Assign `value` to one target, which may itself be a nested pattern.
            match &target.kind {
                ExprKind::Array(_) | ExprKind::Record(_) => {
                    let holder = self.pattern_holder();
                    let name = Expr::new(ExprKind::Name(holder.text.clone()), value.span);
                    out.push(Stmt { labels: Vec::new(), control_target: None,
                        kind: StmtKind::Let { rebindable: false, name: holder, annotation: None, value, binding: None },
                        span,
                    });
                    self.assign_pattern_positions(target, name, depth, &mut work)?;
                }
                ExprKind::Assign { .. } => {
                    return Err(Diagnostic::new(
                        "unsupported",
                        "Default values are not supported in a destructuring assignment; assign first, then use ??= on the target.",
                        target.span,
                    ));
                }
                _ => out.push(Stmt { labels: Vec::new(), control_target: None, kind: StmtKind::Assign { target, value, operator: None }, span }),
            }
        }
        if out.is_empty() {
            return Err(Diagnostic::new("syntax", "A destructuring assignment needs at least one target.", span));
        }
        let first = out.remove(0);
        self.pending.append(&mut out);
        Ok(first.into_kind())
    }

    /// Queue each position or field of `source` for its matching target.
    fn assign_pattern_positions(&mut self, target: Expr, source: Expr, depth: usize, work: &mut Vec<PatternAssignment>) -> Result<(), Diagnostic> {
        if self.policy.bounded() && depth > 32 {
            return Err(Diagnostic::new("unsupported", "Destructuring pattern nesting exceeds the limit of 32.", target.span));
        }
        let select = |source: &Expr| Expr::new(source.kind.clone(), source.span);
        match target.into_kind() {
            ExprKind::Array(targets) => {
                let count = targets.len();
                work.extend(targets.into_iter().enumerate().rev().map(|(position, element)| PatternAssignment::Element { element, source: select(&source), position, count, depth }));
            }
            ExprKind::Record(fields) => {
                work.extend(fields.into_iter().rev().map(|(key, element)| PatternAssignment::Field { key, element, source: select(&source), depth }));
            }
            _ => unreachable!("pattern target"),
        }
        Ok(())
    }

    // Discarding the result preserves existing standalone owned replacement.
    // Keep Stmt-sized construction out of recursive parser statement frames.
    fn expression_statement(expression: Expr) -> StmtKind {
        if !matches!(expression.kind, ExprKind::CompoundAssign { .. } | ExprKind::Assign { .. }) { return StmtKind::Expr(expression); }
        match expression.into_kind() {
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
            _ => unreachable!("assignment statement"),
        }
    }

    #[inline(never)]
    fn finish_sequence(values:Vec<Expr>)->Result<Expr,Diagnostic>{
        let span=Span::new(values.first().unwrap().span.start,values.last().unwrap().span.end);
        Ok(Expr::new(ExprKind::Sequence(values),span))
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

    #[inline(never)]
    pub(super) fn record_rest_pattern(&mut self,whole:&str,rebindable:bool,output:&mut Vec<Stmt>,excluded:&[String])->Result<(),Diagnostic>{
        let start=self.expect("...")?.span;let name=self.name()?;
        if !self.is("}"){return Err(Diagnostic::new("syntax","A rest binding must be last and cannot have a default or trailing comma.",self.peek().span));}
        let base=Expr::new(ExprKind::Name(whole.to_owned()),name.span);
        let method=Expr::new(ExprKind::Field(Box::new(base),"pattern#record-rest".into()),name.span);
        let args=excluded.iter().map(|key|Expr::new(ExprKind::String(key.clone()),name.span)).collect();
        let value=Expr::new(ExprKind::Call(Box::new(method),args),name.span);
        output.push(Stmt{labels:Vec::new(),control_target:None,kind:StmtKind::Let{rebindable,name,annotation:None,value,binding:None},span:start});Ok(())
    }

    #[inline(never)]
    fn replace_pattern_source(mut expr: &mut Expr, whole: &str, source: &str, span: Span) {
        loop {
            let next = match &mut expr.kind {
                ExprKind::Binary { op: BinaryOp::PatternDefault, left, .. } => left,
                ExprKind::Unary { op: UnaryOp::PatternSelection, value, .. } => value,
                ExprKind::Call(fun, _) => fun,
                ExprKind::Index(base, _) | ExprKind::Field(base, _) if matches!(&base.kind, ExprKind::Name(name) if name == whole) => {
                    base.kind = ExprKind::Name(source.into());
                    base.span = span;
                    return;
                }
                _ => return,
            };
            expr = next;
        }
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

    #[inline(never)]
    fn type_assertion_tail(&mut self, value: Expr) -> Result<Expr, Diagnostic> {
        let satisfies = self.take().text == "satisfies";
        let annotation = Box::new(self.value_annotation()?);
        let span = Span::new(value.span.start, annotation.span.end);
        let expression = Expr::new(ExprKind::TypeAssertion {
            value: Box::new(value), annotation, satisfies,
        }, span);
        if self.policy.exceeded(expression.depth) {
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

    fn unary_operators(&mut self) -> Result<Vec<Token>, Diagnostic> {
        let mut operators = Vec::new();
        while self.is("await") || self.is("!")
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
            if self.policy.reached(operators.len()) {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Expression nesting exceeds the prototype limit of 128.",
                    token.span,
                ));
            }
            operators.push(token);
        }
        Ok(operators)
    }

    // Keep operator construction off the recursive primary parsing stack.
    fn apply_unary_operators(&self, mut value: Expr, operators: Vec<Token>) -> Result<Expr, Diagnostic> {
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
                        op: if token.text == "await" { UnaryOp::Await } else if token.text == "typeof" {
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
            if self.policy.exceeded(value.depth) {
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

    // Keep the unparenthesized typed-arrow diagnostic temporaries out of the
    // recursively retained primary frame.
    #[inline(never)]
    fn typed_arrow_without_parentheses(&mut self) -> Result<(), Diagnostic> {
        if self.peek().kind == TokenKind::Name && self.tokens.get(self.i + 1).is_some_and(|token| token.text == ":") {
            let mut cursor = self.i + 2;
            let first = self.tokens.get(cursor).map(|token| token.span.start);
            // A scan that starts inside a clear range ends where that scan ended.
            if let (Some(first), Some((from, until))) = (first, self.typed_arrow_clear)
                && from <= first
                && first <= until
            {
                return Ok(());
            }
            let mut closing = None;
            // Packed closers can still be split into `=` or `=>`; a range that
            // holds one is scanned again.
            let mut stable = true;
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
                stable &= !matches!(token.text.as_str(), ">>" | ">>>" | ">=" | ">>=" | ">>>=");
                cursor += 1;
            }
            if let (true, Some(first), Some(last)) = (stable, first, self.tokens.get(cursor.min(self.tokens.len() - 1))) {
                self.typed_arrow_clear = Some((first, last.span.start));
            }
        }
        Ok(())
    }

    // Parse exactly one result type. Scanning for a later arrow mistakes a
    // conditional arm's separating colon for an outer closure annotation.
    fn annotated_arrow_follows(&mut self, start: usize) -> bool {
        crate::type_syntax::arrow_follows_prefix(&mut self.tokens, start, self.policy)
    }

    /// The `)` that closes the `(` at the cursor. One scan records every
    /// pair inside it, so nested groups are not scanned again. Pairs are
    /// recorded by source offset: packed closer splits move token indices but
    /// never add or remove a parenthesis.
    fn closing_parenthesis(&mut self) -> Option<usize> {
        let position = |tokens: &[Token], offset: usize| tokens.partition_point(|token| token.span.start < offset);
        if let Some(close) = self.parenthesis_closers.get(&self.peek().span.start) {
            return close.map(|offset| position(&self.tokens, offset));
        }
        let mut open = Vec::new();
        for (index, token) in self.tokens.iter().enumerate().skip(self.i) {
            match token.text.as_str() {
                "(" => open.push(token.span.start),
                ")" => {
                    let Some(start) = open.pop() else { break };
                    self.parenthesis_closers.insert(start, Some(token.span.start));
                    if open.is_empty() {
                        return Some(index);
                    }
                }
                _ => {}
            }
        }
        for start in open {
            self.parenthesis_closers.insert(start, None);
        }
        None
    }

    fn closure_header(&mut self) -> bool {
        let Some(position) = self.closing_parenthesis() else { return false };
        let annotated = match self.tokens.get(position + 1).map(|token| token.text.as_str()) {
            Some("=>") => return true,
            Some(":") => true,
            _ => false,
        };
        annotated && self.annotated_arrow_follows(position + 2)
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
        if self.policy.exceeded(result.depth) {
            return Err(Diagnostic::new(
                "unsupported",
                "Expression nesting exceeds the prototype limit of 128.",
                span,
            ));
        }
        Ok(result)
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
    fn explicit_call_arguments(&mut self) -> Result<Vec<crate::type_syntax::ValueAnnotation>, Diagnostic> {
        self.expect("<")?;
        let (arguments, next) = crate::type_syntax::parse_arguments(&mut self.tokens, self.i, self.depth, self.policy);
        self.i = next;
        let mut arguments = arguments?;
        if !self.aliases.is_empty() {
            for argument in &mut arguments {
                crate::type_aliases::expand(argument, &self.aliases);
            }
        }
        Ok(arguments)
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

    fn finish_explicit_call(&mut self, expr: Expr,
        arguments: Vec<crate::type_syntax::ValueAnnotation>, args: Vec<Expr>, end: Token,
    ) -> Result<Expr, Diagnostic> {
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
        if matches!(&Self::optional_identity(&expr).kind, ExprKind::Field(base, member)
            if member != "recursive" && !matches!(&base.kind, ExprKind::Name(name) if name == "JSON" && matches!(member.as_str(), "parse" | "tryParse"))) {
            return Err(Diagnostic::new(
                "unsupported",
                "Explicit member type arguments are supported for Array.of, JSON.parse, JSON.tryParse and schema recursive definitions.",
                span,
            ));
        }
        let call = Expr::new(ExprKind::Call(Box::new(expr), args), span);
        self.call_type_arguments.push((span, arguments));
        Ok(call)
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

    #[inline(never)]
    fn postfix_depth(&self, expr: &Expr) -> Result<(), Diagnostic> {
        if self.policy.exceeded(expr.depth) {
            return Err(Diagnostic::new(
                "unsupported",
                "Expression nesting exceeds the prototype limit of 128.",
                expr.span,
            ));
        }
        Ok(())
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
        for source in [include_str!("../../tests/fixtures/exception-handling/basic.tsr"),include_str!("../../tests/fixtures/exception-handling/completions.tsr")] {
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

mod enum_syntax;

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
