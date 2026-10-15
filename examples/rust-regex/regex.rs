pub struct Search {
    pub pattern: String,
    pub text: String,
}
pub struct Hit {
    pub text: String,
    pub start: i64,
    pub end: i64,
}
pub enum SearchResult {
    Found(Hit),
    Missing,
}
/// Find the first match. Invalid patterns return an error string.
/// start/end are UTF-8 byte offsets; the search request is consumed.
pub fn search(request: Search) -> Result<SearchResult, String> {
    let regex = regex::Regex::new(&request.pattern).map_err(|error| error.to_string())?;
    Ok(match regex.find(&request.text) {
        Some(found) => SearchResult::Found(Hit {
            text: found.as_str().to_owned(),
            start: found.start() as i64,
            end: found.end() as i64,
        }),
        None => SearchResult::Missing,
    })
}
