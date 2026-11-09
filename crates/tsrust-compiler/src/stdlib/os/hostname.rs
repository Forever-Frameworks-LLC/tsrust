mod tsrust_native_hostname {
    #[derive(Debug,PartialEq,Eq)]
    pub struct Error {pub code:&'static str,pub message:std::string::String}
    pub fn get()->Result<std::string::String,Error> {
        hostname::get().map_err(|error|Error{code:"Io",message:error.to_string()})?
            .into_string().map_err(|_|Error{code:"NotUnicode",message:"Host name is not valid Unicode".into()})
    }
}
