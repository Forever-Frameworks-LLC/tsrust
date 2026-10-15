//! Compiler-owned standard-library binding metadata. Language import integration is staged separately.
use crate::Mode;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeType {
    Structured(&'static crate::native_signatures::Signature),
    String,
    Boolean,
    Int,
    Number,
    Void,
    /// Owned reusable callback with no arguments and a void result.
    OwnedVoidCallback,
    /// Private registry callback; program access is supplied at invocation.
    ProgramVoidCallback,
    ProgramReusableVoidCallback,
    /// Private polymorphic compiler intrinsic; never emitted as a native call.
    SharedCallback,
    /// Private read-only value observation, resolved by compiler checking.
    ObservedValue,
}
impl NativeType {
    /// Shared semantic signature projection for native checking and editor help.
    pub fn signature(self) -> crate::native_signatures::Signature {
        use crate::native_signatures::{Intrinsic, Signature};
        match self {
            Self::Structured(signature) => *signature,
            Self::String => Signature::String,
            Self::Boolean => Signature::Boolean,
            Self::Int => Signature::Int,
            Self::Number => Signature::Number,
            Self::Void => Signature::Void,
            Self::ProgramReusableVoidCallback => Signature::Intrinsic(Intrinsic::ProgramReusableVoidCallback),
            Self::ProgramVoidCallback => Signature::Intrinsic(Intrinsic::ProgramVoidCallback),
            Self::OwnedVoidCallback => Signature::Intrinsic(Intrinsic::OwnedVoidCallback),
            Self::SharedCallback => Signature::Intrinsic(Intrinsic::SharedCallback),
            Self::ObservedValue => Signature::Intrinsic(Intrinsic::ObservedValue),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeParameter {
    pub name: &'static str,
    pub ty: NativeType,
    pub access: Mode,
    pub optional: bool,
    /// Final homogeneous rest parameter; ty describes each argument, not a public array argument.
    pub rest: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeBackend {
    /// Bundled generated Rust code and compiler-owned dependencies; no user package graph or build-time network access.
    Bundled { symbol: &'static str },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeBinding {
    pub module: &'static str,
    pub name: &'static str,
    pub parameters: &'static [NativeParameter],
    pub result: NativeType,
    /// Source loan carried by a native return payload, when present.
    pub returned_borrow: Option<crate::native_return_borrows::ReturnedBorrow>,
    pub backend: NativeBackend,
    pub features: &'static [&'static str],
}
impl NativeBinding {
    /// Further native types that a parameter accepts besides its declared type.
    /// A byte observer reads any readable byte storage; a byte mutator writes any
    /// writable one. A stored function value keeps the declared parameter type.
    pub fn argument_alternatives(&self, position: usize) -> &'static [crate::native_signatures::NativeTypeId] {
        if self.module=="std:sqlite" && self.name=="__prepare" && position==0 {return &[crate::native_signatures::NativeTypeId::SqliteTransaction];}
        use crate::native_signatures::NativeTypeId::{ByteView, ByteViewMut, BufferedReader, BufferedWriter};
        if self.module == "std:json" && position == 0 && matches!(self.name,"asString"|"get"|"at"|"kind"|"asBoolean"|"asInt"|"asNumber"|"stringify") {
            return &[crate::native_signatures::NativeTypeId::JsonView];
        }
        if self.module == "std:json" && position == 0 && matches!(self.name,"readJsonl"|"closeJsonl") {return &[crate::native_signatures::NativeTypeId::JsonlReaderView];}
        if self.module == "std:json" && position == 0 && matches!(self.name,"writeJsonl"|"flushJsonl"|"closeJsonlWriter") {return &[crate::native_signatures::NativeTypeId::JsonlWriterView];}
        if self.module == "std:fs" && position == 0 {
            match self.name {
                "read" => return &[BufferedReader],
                "write" | "writeAll" => return &[BufferedWriter],
                "close" => return &[BufferedReader, BufferedWriter],
                _ => {},
            }
        }
        if self.module == "std:fs" && position == 1 && matches!(self.name, "write" | "writeAll" | "writeFile") {
            return &[ByteView, ByteViewMut];
        }
        if self.module != "std:bytes" || position != 0 {
            return &[];
        }
        match self.name {
            // A view of a view borrows the parent view for as long as it lives.
            "length" | "at" | "copy" | "toUtf8" | "view" => &[ByteView, ByteViewMut],
            "set" | "viewMut" => &[ByteViewMut],
            _ => &[],
        }
    }
    pub fn accepts_arity(&self, count: usize) -> bool {
        let required = self
            .parameters
            .iter()
            .take_while(|parameter| !parameter.optional && !parameter.rest)
            .count();
        count >= required && (self.parameters.last().is_some_and(|parameter| parameter.rest) || count <= self.parameters.len())
    }
}
pub const BINDINGS: &[NativeBinding] = &[
    crate::stdlib::sql::CONNECT,
    crate::stdlib::sql::CONNECT_POLL,
    crate::stdlib::sql::BEGIN,
    crate::stdlib::sql::CLOSE,
    crate::stdlib::sql::EXECUTE,
    crate::stdlib::sql::QUERY,
    crate::stdlib::sql::TXEXECUTE,
    crate::stdlib::sql::TXQUERY,
    crate::stdlib::sql::COMMIT,
    crate::stdlib::sql::ROLLBACK,
    crate::stdlib::sql::INT,
    crate::stdlib::sql::NUMBER,
    crate::stdlib::sql::BOOLEAN,
    crate::stdlib::sql::TEXT,
    crate::stdlib::sql::BLOB,
    crate::stdlib::sql::ISNULL,
    crate::stdlib::sqlite::OPEN,
    crate::stdlib::sqlite::PREPARE,
    crate::stdlib::sqlite::BEGIN,
    crate::stdlib::sqlite::CLOSE,
    crate::stdlib::sqlite::COMMIT,
    crate::stdlib::sqlite::ROLLBACK,
    crate::stdlib::sqlite::FINALIZE,
    crate::stdlib::sqlite::EXECUTE,
    crate::stdlib::sqlite::QUERY,
    crate::stdlib::sqlite::INT,
    crate::stdlib::sqlite::NUMBER,
    crate::stdlib::sqlite::TEXT,
    crate::stdlib::sqlite::BLOB,
    crate::stdlib::sqlite::ISNULL,
    crate::stdlib::http::APP,
    crate::stdlib::http::CLIENT,
    crate::stdlib::http::REQUEST,
    crate::stdlib::http::TEXT,
    crate::stdlib::http::HTML,
    crate::stdlib::http::JSON,
    crate::stdlib::http::STREAM,
    crate::stdlib::http::ROUTE,
    crate::stdlib::http::USE,
    crate::stdlib::http::FALLBACK,
    crate::stdlib::http::LISTEN,
    crate::stdlib::http::SHUTDOWN,
    crate::stdlib::http::WAIT,
    crate::stdlib::http::ADDRESS,
    crate::stdlib::http::NEXT,
    crate::stdlib::http::SEND,
    crate::stdlib::http::BODYNEXT,
    crate::stdlib::http::BODYTEXT,
    crate::stdlib::http::REQUESTHEADER,
    crate::stdlib::http::REQUESTSETHEADER,
    crate::stdlib::http::REQUESTBODY,
    crate::stdlib::http::REQUESTTEXT,
    crate::stdlib::http::RESPONSEHEADER,
    crate::stdlib::http::RESPONSESETHEADER,
    crate::stdlib::http::RESPONSEBODY,
    crate::stdlib::http::RESPONSETEXT,
    crate::stdlib::http::METHOD,
    crate::stdlib::http::URL,
    crate::stdlib::http::PARAM,
    crate::stdlib::http::STATUS,
    crate::stdlib::http::PUBLIC_TEXT,
    crate::stdlib::http::PUBLIC_HTML,
    crate::stdlib::http::PUBLIC_JSON,
    crate::stdlib::http::PUBLIC_REQUEST,
    crate::stdlib::http::PUBLIC_STREAM,
    crate::stdlib::futures::DELAY,
    crate::stdlib::futures::YIELD,
    crate::stdlib::futures::RACE,
    crate::stdlib::futures::SETTLED,
    crate::stdlib::futures::START,
    crate::stdlib::futures::CANCEL,
    crate::stdlib::futures::JOIN,
    crate::stdlib::futures::TIMEOUT,
    crate::stdlib::futures::SLEEP,
    crate::stdlib::futures::YIELD_POLL,
    crate::stdlib::futures::RACE_POLL,
    crate::stdlib::futures::SETTLED_POLL,
    crate::stdlib::futures::CHANNEL,
    crate::stdlib::futures::SENDER,
    crate::stdlib::futures::CLOSE,
    crate::stdlib::futures::SEND,
    crate::stdlib::futures::RECEIVE,

    crate::stdlib::threads::TASK_CANCELLED,
    crate::stdlib::threads::CHANNEL_CREATE,
    crate::stdlib::threads::CHANNEL_SENDER,
    crate::stdlib::threads::CHANNEL_RECEIVE,
    crate::stdlib::threads::CHANNEL_TRY_RECEIVE,
    crate::stdlib::threads::CHANNEL_CLOSE,
    crate::stdlib::threads::SENDER_SEND,
    crate::stdlib::threads::SENDER_TRY_SEND,
    crate::stdlib::threads::SENDER_CLOSE,

    crate::stdlib::threads::SPAWN,
    crate::stdlib::threads::CREATE_POOL,
    crate::stdlib::threads::OWNED_POOL_TASK,
    crate::stdlib::threads::OWNED_POOL_STATS,
    crate::stdlib::threads::OWNED_POOL_SHUTDOWN,
    crate::stdlib::threads::CANCEL,
    crate::stdlib::threads::CANCELLATION_REQUESTED,
    crate::stdlib::threads::SPAWN_SCOPED,
    crate::stdlib::result::UNWRAP,
    crate::stdlib::threads::JOIN,
    crate::stdlib::threads::DETACH,
    crate::stdlib::schema::namespace::STRING,
    crate::stdlib::schema::namespace::MIN_STRING,
    crate::stdlib::schema::namespace::GT_INT,
    crate::stdlib::schema::namespace::LT_INT,
    crate::stdlib::schema::namespace::MIN_NUMBER,
    crate::stdlib::schema::namespace::MAX_NUMBER,
    crate::stdlib::schema::namespace::GT_NUMBER,
    crate::stdlib::schema::namespace::LT_NUMBER,
    crate::stdlib::schema::namespace::MIN_INT,
    crate::stdlib::schema::namespace::MAX_INT,
    crate::stdlib::schema::namespace::MAX_STRING,
    crate::stdlib::schema::namespace::BOOLEAN,
    crate::stdlib::schema::namespace::NULL,
    crate::stdlib::schema::namespace::SAFE_PARSE_NULL,
    crate::stdlib::schema::namespace::SAFE_PARSE_NULL_VALUE,
    crate::stdlib::schema::namespace::INT,
    crate::stdlib::schema::namespace::NUMBER,
    crate::stdlib::schema::namespace::SAFE_PARSE_NUMBER,
    crate::stdlib::schema::namespace::SAFE_PARSE_NUMBER_VALUE,
    crate::stdlib::schema::namespace::SAFE_PARSE_NUMBER_INT,
    crate::stdlib::schema::namespace::SAFE_PARSE_INT,
    crate::stdlib::schema::namespace::SAFE_PARSE_INT_VALUE,
    crate::stdlib::schema::namespace::SAFE_PARSE_BOOLEAN,
    crate::stdlib::schema::namespace::SAFE_PARSE_BOOLEAN_VALUE,
    crate::stdlib::schema::namespace::SAFE_PARSE,
    crate::stdlib::schema::namespace::SAFE_PARSE_OWNED,
    crate::stdlib::json::bindings::PARSE,
    crate::stdlib::json::bindings::PARSE_JSONC,
    crate::stdlib::json::bindings::BORROW_RAW,
    crate::stdlib::json::bindings::CREATE_JSONL,
    crate::stdlib::json::bindings::BORROW_JSONL_WRITER,
    crate::stdlib::json::bindings::WRITE_JSONL,
    crate::stdlib::json::bindings::FLUSH_JSONL,
    crate::stdlib::json::bindings::CLOSE_JSONL_WRITER,
    crate::stdlib::json::bindings::OPEN_JSONL,
    crate::stdlib::json::bindings::BORROW_JSONL,
    crate::stdlib::json::bindings::READ_JSONL,
    crate::stdlib::json::bindings::CLOSE_JSONL,
    crate::stdlib::json::bindings::STRINGIFY,
    crate::stdlib::json::bindings::KIND,
    crate::stdlib::json::bindings::GET,
    crate::stdlib::json::bindings::AT,
    crate::stdlib::json::bindings::FROM_ARRAY,
    crate::stdlib::json::bindings::FROM_ENTRIES,
    crate::stdlib::json::bindings::NULL_VALUE,
    crate::stdlib::json::bindings::FROM_BOOLEAN,
    crate::stdlib::json::bindings::FROM_INT,
    crate::stdlib::json::bindings::FROM_NUMBER,
    crate::stdlib::json::bindings::FROM_STRING,
    crate::stdlib::json::bindings::AS_STRING,
    crate::stdlib::json::bindings::AS_BOOLEAN,
    crate::stdlib::json::bindings::AS_INT,
    crate::stdlib::json::bindings::AS_NUMBER,
    crate::stdlib::json::bindings::TAKE_STRING,
    crate::stdlib::env::bindings::GET,
    crate::stdlib::time::MONOTONIC_NOW,
    crate::stdlib::fs::bindings::BUFFER_READER,
    crate::stdlib::fs::bindings::BUFFER_WRITER,
    crate::stdlib::fs::bindings::FLUSH,
    crate::stdlib::fs::bindings::OPEN,
    crate::stdlib::fs::bindings::CREATE,
    crate::stdlib::fs::bindings::READ_FILE,
    crate::stdlib::fs::bindings::WRITE_FILE,
    crate::stdlib::fs::bindings::WRITE,
    crate::stdlib::fs::bindings::WRITE_ALL,
    crate::stdlib::fs::bindings::READ,
    crate::stdlib::fs::bindings::CLOSE,
    crate::stdlib::bytes::bindings::FROM_ARRAY,
    crate::stdlib::bytes::bindings::LENGTH,
    crate::stdlib::bytes::bindings::AT,
    crate::stdlib::bytes::bindings::SET,
    crate::stdlib::bytes::bindings::ALLOC,
    crate::stdlib::bytes::bindings::RESIZE,
    crate::stdlib::bytes::bindings::APPEND,
    crate::stdlib::bytes::bindings::COPY,
    crate::stdlib::bytes::bindings::FROM_UTF8,
    crate::stdlib::bytes::bindings::TO_UTF8,
    crate::stdlib::testing::bindings::CONTAINS_VALUE,
    crate::stdlib::testing::bindings::LENGTH_VALUE,
    crate::stdlib::testing::bindings::HAS_LENGTH,
    crate::stdlib::testing::bindings::NUMERIC_VALUE,
    crate::stdlib::testing::bindings::ASSERTION_DETAILS,
    crate::stdlib::testing::bindings::SHARE_CALLBACK,
    crate::stdlib::path::bindings::BASENAME,
    crate::stdlib::path::bindings::DIRNAME,
    crate::stdlib::path::bindings::EXTNAME,
    crate::stdlib::path::bindings::IS_ABSOLUTE,
    crate::stdlib::path::bindings::NORMALIZE,
    crate::stdlib::testing::bindings::CHECK_ASSERTION,
    crate::stdlib::testing::bindings::INVOKE_REGISTRATION,
    crate::stdlib::testing::bindings::CREATE_REGISTRY,
    crate::stdlib::testing::bindings::REGISTER_TEST,
    crate::stdlib::testing::bindings::RUN_TESTS,
    crate::stdlib::testing::bindings::FINISH_TESTS,
    crate::stdlib::testing::bindings::BEGIN_SUITE,
    crate::stdlib::testing::bindings::END_SUITE,
    crate::stdlib::testing::bindings::BEGIN_FILE,
    crate::stdlib::testing::bindings::REGISTER_HOOK,
    crate::stdlib::os::bindings::PLATFORM,
    crate::stdlib::os::bindings::ARCH,
    crate::stdlib::os::bindings::ENDIANNESS,
    crate::stdlib::path::bindings::JOIN,
    crate::stdlib::path::bindings::RELATIVE,
    crate::stdlib::bytes::bindings::VIEW,
    crate::stdlib::bytes::bindings::VIEW_MUT,
];
/// Checked SQLite member metadata, shared with editor inference.
pub fn sqlite_method(identity: crate::native_signatures::NativeTypeId, member: &str) -> Option<&'static NativeBinding> {
    lookup("std:sqlite",crate::stdlib::sqlite::method(Some(identity),member)?).ok()
}

pub fn lookup(module: &str, name: &str) -> Result<&'static NativeBinding, String> {
    if !BINDINGS.iter().any(|binding| binding.module == module) {
        return Err(format!("Unknown standard-library module {module:?}."));
    }
    BINDINGS
        .iter()
        .find(|binding| binding.module == module && binding.name == name)
        .ok_or_else(|| format!("Standard-library module {module:?} does not export {name:?}."))
        .and_then(|binding| {
            binding.validate_returned_borrow().map_err(|error| format!("Invalid native returned-borrow contract for {module}::{name}: {error:?}."))?;
            Ok(binding)
        })
}
/// Deterministic native feature selection from resolved imports. Unknown exports are errors.
pub fn selected_features<'a>(
    imports: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<BTreeSet<&'static str>, String> {
    let mut selected = BTreeSet::new();
    for (module, name) in imports {
        if let crate::native_exports::Export::Function(binding) = crate::native_exports::lookup(module, name)? {
            selected.extend(binding.features.iter().copied());
        }
    }
    Ok(selected)
}

/// Internal AST identity cannot be spelled as a source identifier.
pub fn by_symbol(symbol: &str) -> Option<(usize, &'static NativeBinding)> {
    let (module, name) = symbol.strip_prefix("@native:")?.split_once('#')?;
    BINDINGS
        .iter()
        .enumerate()
        .find(|(_, binding)| binding.module == module && binding.name == name)
}

pub fn sql_method(identity: crate::native_signatures::NativeTypeId, member: &str) -> Option<&'static NativeBinding> {
 lookup("std:sql",crate::stdlib::sql::method(Some(identity),member)?).ok()
}
pub fn sql_asynchronous(name:&str)->bool{crate::stdlib::sql::asynchronous(name)}
