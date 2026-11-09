mod tsrust_native_url {
    pub type Url = url::Url;
    #[derive(Debug)]
    pub struct Error { pub code: &'static str, pub operation: &'static str, pub message: String }
    fn error(code: &'static str, operation: &'static str, message: &str) -> Error {
        Error { code, operation, message: message.to_owned() }
    }
    fn parsed(error: url::ParseError, operation: &'static str) -> Error {
        self::error("parse", operation, &error.to_string())
    }
    pub fn parse_url(text: &str, base: Option<&Url>) -> Result<Url, Error> {
        Url::options().base_url(base).parse(text).map_err(|e| parsed(e,"parseUrl"))
    }
    pub fn resolve_url(base: &Url, reference: &str) -> Result<Url, Error> {
        base.join(reference).map_err(|e| parsed(e,"resolveUrl"))
    }
    pub fn format_url(value: &Url) -> &str { value.as_str() }
    pub fn href(value: &Url) -> &str { value.as_str() }
    pub fn protocol(value: &Url) -> &str { &value.as_str()[..value.scheme().len()+1] }
    pub fn username(value: &Url) -> &str { value.username() }
    pub fn password(value: &Url) -> Option<&str> { value.password() }
    pub fn hostname(value: &Url) -> Option<&str> { value.host_str() }
    pub fn port(value: &Url) -> Option<i64> { value.port().map(i64::from) }
    pub fn pathname(value: &Url) -> &str { value.path() }
    pub fn search(value: &Url) -> Option<&str> {
        value.query().map(|_| &value[url::Position::AfterPath..url::Position::AfterQuery])
    }
    pub fn hash(value: &Url) -> Option<&str> {
        value.fragment().map(|_| &value[url::Position::AfterQuery..])
    }
    pub fn origin(value: &Url) -> String { value.origin().ascii_serialization() }
    pub fn set_href(value: &mut Url, text: &str) -> Result<(), Error> {
        let replacement=Url::parse(text).map_err(|e|parsed(e,"href"))?;
        *value=replacement;Ok(())
    }
    pub fn set_protocol(value: &mut Url, text: &str) -> Result<(), Error> {
        value.set_scheme(text.strip_suffix(':').unwrap_or(text)).map_err(|_|error("scheme","protocol","Scheme cannot be assigned to this URL"))
    }
    pub fn set_username(value: &mut Url, text: &str) -> Result<(), Error> {
        value.set_username(text).map_err(|_|error("credentials","username","This URL cannot contain credentials"))
    }
    pub fn set_password(value: &mut Url, text: Option<&str>) -> Result<(), Error> {
        value.set_password(text).map_err(|_|error("credentials","password","This URL cannot contain credentials"))
    }
    pub fn set_hostname(value: &mut Url, text: Option<&str>) -> Result<(), Error> {
        // url#1174: empty host with credentials/port corrupts offsets; reject before mutation.
        if text==Some("") && (value.port().is_some() || !value.username().is_empty() || value.password().is_some()) {
            return Err(error("host","hostname","An empty host cannot retain credentials or a port"));
        }
        value.set_host(text).map_err(|e|parsed(e,"hostname"))
    }
    pub fn set_port(value: &mut Url, port: Option<i64>) -> Result<(), Error> {
        let port=port.map(|port|u16::try_from(port).map_err(|_|error("range","port","Port must be between 0 and 65535"))).transpose()?;
        value.set_port(port).map_err(|_|error("port","port","This URL cannot contain a port"))
    }
    pub fn set_pathname(value: &mut Url, text: &str) -> Result<(), Error> { value.set_path(text);Ok(()) }
    pub fn set_search(value: &mut Url, text: Option<&str>) -> Result<(), Error> {
        value.set_query(text.map(|text|text.strip_prefix('?').unwrap_or(text)));Ok(())
    }
    pub fn set_hash(value: &mut Url, text: Option<&str>) -> Result<(), Error> {
        value.set_fragment(text.map(|text|text.strip_prefix('#').unwrap_or(text)));Ok(())
    }
    #[derive(Clone, Debug)]
    pub struct SearchParams { entries: Vec<(String,String)> }
    pub fn parse_query(text: &str) -> SearchParams {
        SearchParams {entries:url::form_urlencoded::parse(text.strip_prefix('?').unwrap_or(text).as_bytes()).into_owned().collect()}
    }
    pub fn format_query(value: &SearchParams) -> String {
        url::form_urlencoded::Serializer::new(String::new()).extend_pairs(&value.entries).finish()
    }
    pub fn search_params(value: &Url) -> SearchParams {
        SearchParams {entries:value.query_pairs().into_owned().collect()}
    }
    pub fn set_search_params(value: &mut Url, params: &SearchParams) {
        let query=format_query(params);value.set_query((!query.is_empty()).then_some(query.as_str()));
    }
    impl SearchParams {
        pub fn get(&self, name: &str) -> Option<&str> {self.entries.iter().find(|(key,_)|key==name).map(|(_,value)|value.as_str())}
        pub fn get_all(&self, name: &str) -> Vec<String> {self.entries.iter().filter(|(key,_)|key==name).map(|(_,value)|value.clone()).collect()}
        pub fn has(&self, name: &str) -> bool {self.entries.iter().any(|(key,_)|key==name)}
        pub fn append(&mut self, name: &str, value: &str) {self.entries.push((name.to_owned(),value.to_owned()));}
        pub fn set(&mut self, name: &str, value: &str) {
            if let Some(first)=self.entries.iter().position(|(key,_)|key==name) {
                self.entries[first].1=value.to_owned();
                let mut index=0;self.entries.retain(|(key,_)|{let keep=index<=first || key!=name;index+=1;keep});
            } else {self.append(name,value);}
        }
        pub fn delete(&mut self, name: &str) {self.entries.retain(|(key,_)|key!=name);}
        pub fn entries(&self) -> Vec<(String,String)> {self.entries.clone()}
        pub fn sort(&mut self) {self.entries.sort_by(|left,right|left.0.cmp(&right.0));}
    }
    pub fn path_to_file_url(path: &str) -> Result<Url, Error> {
        Url::from_file_path(path).map_err(|_|error("path","pathToFileUrl","Expected an absolute native path"))
    }
    pub fn file_url_to_path(value: &Url) -> Result<String, Error> {
        let path=value.to_file_path().map_err(|_|error("path","fileUrlToPath","URL cannot represent a native file path"))?;
        path.into_os_string().into_string().map_err(|_|error("encoding","fileUrlToPath","File path is not valid Unicode"))
    }
    const COMPONENT: &percent_encoding::AsciiSet=&percent_encoding::NON_ALPHANUMERIC
        .remove(b'-').remove(b'_').remove(b'.').remove(b'!').remove(b'~').remove(b'*').remove(b'\'').remove(b'(').remove(b')');
    pub fn encode_uri_component(text: &str) -> String {percent_encoding::utf8_percent_encode(text,COMPONENT).to_string()}
    pub fn decode_uri_component(text: &str) -> Result<String, Error> {
        let bytes=text.as_bytes();let mut index=0;
        while index<bytes.len() {
            if bytes[index]==b'%' {
                if index+2>=bytes.len() || !bytes[index+1].is_ascii_hexdigit() || !bytes[index+2].is_ascii_hexdigit() {
                    return Err(error("percent","decodeURIComponent","Expected two hexadecimal digits after percent"));
                }
                index+=3;
            } else {index+=1;}
        }
        percent_encoding::percent_decode_str(text).decode_utf8().map(|text|text.into_owned()).map_err(|_|error("encoding","decodeURIComponent","Decoded component is not valid UTF-8"))
    }
}
