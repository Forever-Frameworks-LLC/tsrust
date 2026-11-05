mod tsrust_native_regex {
    #[derive(Debug)]
    pub struct Regex { inner: regex::Regex }
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Match { pub start: i64, pub end: i64 }
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Captures { pub matched: Match, pub groups: Vec<Option<Match>> }
    #[derive(Debug, PartialEq, Eq)]
    pub struct Error { pub code: &'static str, pub message: std::string::String }
    #[derive(Clone, Copy, Debug, Default)]
    pub struct Options {
        pub size_limit: Option<i64>,
        pub dfa_size_limit: Option<i64>,
        pub nest_limit: Option<i64>,
    }
    fn range_error(field: &str, range: &str) -> Error {
        Error { code: "Range", message: format!("{field} must be {range}") }
    }
    fn byte_limit(value: i64, field: &str) -> Result<usize, Error> {
        usize::try_from(value).map_err(|_| range_error(field, "a nonnegative byte count representable on this target"))
    }
    pub fn compile(pattern: &str, options: Option<&Options>) -> Result<Regex, Error> {
        let mut builder = regex::RegexBuilder::new(pattern);
        if let Some(options) = options {
            if let Some(value) = options.size_limit { builder.size_limit(byte_limit(value, "sizeLimit")?); }
            if let Some(value) = options.dfa_size_limit { builder.dfa_size_limit(byte_limit(value, "dfaSizeLimit")?); }
            if let Some(value) = options.nest_limit {
                builder.nest_limit(u32::try_from(value).map_err(|_| range_error("nestLimit", "in 0..4294967295"))?);
            }
        }
        builder.build().map(|inner| Regex { inner }).map_err(|error| {
            let code = match &error {
                regex::Error::Syntax(_) => "Syntax",
                regex::Error::CompiledTooBig(_) => "SizeLimit",
                _ => "Compile",
            };
            Error { code, message: error.to_string() }
        })
    }
    pub fn escape(text: &str) -> std::string::String { regex::escape(text) }
    fn offset(value: usize) -> i64 {
        // Valid Rust string slices fit isize, hence int on supported 32/64-bit targets.
        i64::try_from(value).expect("Regex byte offset exceeds int")
    }
    fn matched(value: regex::Match<'_>) -> Match { Match { start: offset(value.start()), end: offset(value.end()) } }
    fn captured(value: regex::Captures<'_>) -> Captures {
        Captures { matched: matched(value.get_match()), groups: value.iter().skip(1).map(|group| group.map(matched)).collect() }
    }
    pub fn is_match(value: &Regex, text: &str) -> bool { value.inner.is_match(text) }
    pub fn find(value: &Regex, text: &str) -> Option<Match> { value.inner.find(text).map(matched) }
    pub fn find_all(value: &Regex, text: &str) -> Vec<Match> { value.inner.find_iter(text).map(matched).collect() }
    pub fn captures(value: &Regex, text: &str) -> Option<Captures> { value.inner.captures(text).map(captured) }
    pub fn captures_all(value: &Regex, text: &str) -> Vec<Captures> { value.inner.captures_iter(text).map(captured).collect() }
    pub fn replace(value: &Regex, text: &str, replacement_template: &str) -> std::string::String {
        value.inner.replace(text, replacement_template).into_owned()
    }
    pub fn replace_all(value: &Regex, text: &str, replacement_template: &str) -> std::string::String {
        value.inner.replace_all(text, replacement_template).into_owned()
    }
    pub fn split(value: &Regex, text: &str) -> Vec<Match> {
        let mut start = 0;
        let mut output = Vec::new();
        for found in value.inner.find_iter(text) {
            output.push(Match { start, end: offset(found.start()) });
            start = offset(found.end());
        }
        output.push(Match { start, end: offset(text.len()) });
        output
    }
    pub fn string_match(text: &str, value: &Regex) -> Option<Captures> { captures(value, text) }
    pub fn string_match_all(text: &str, value: &Regex) -> Vec<Captures> { captures_all(value, text) }
    pub fn string_search(text: &str, value: &Regex) -> i64 { find(value, text).map_or(-1, |found| found.start) }
}
