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
    OwnedBooleanCallback,
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
            Self::OwnedBooleanCallback => Signature::Intrinsic(Intrinsic::OwnedBooleanCallback),
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
        if self.module=="std:crypto" && self.parameters.get(position).is_some_and(|parameter|parameter.ty==NativeType::Structured(&crate::native_signatures::Signature::Named(crate::native_signatures::NativeTypeId::Bytes)) && parameter.access==Mode::Read){return &[crate::native_signatures::NativeTypeId::ByteView,crate::native_signatures::NativeTypeId::ByteViewMut];}
        if self.module=="std:net" && self.name=="__sendTo" && position==1{return &[crate::native_signatures::NativeTypeId::ByteView,crate::native_signatures::NativeTypeId::ByteViewMut];}
        if self.module=="std:io" {
            use crate::native_signatures::NativeTypeId::*;
            match (self.name.trim_start_matches('_'),position) {
                ("reader",0)=>return &[BufferedReader,HttpBody,Bytes],
                ("writer",0)=>return &[BufferedWriter],
                ("close",0)=>return &[IoWriter,IoLineReader,CompressionReader,CompressionWriter],
                ("read"|"readInto"|"readExact"|"readAll"|"copy",0)=>return &[TcpStream,CompressionReader],
                ("write"|"writeAll"|"flush",0)=>return &[TcpStream,CompressionWriter],
                ("copy",1)=>return &[TcpStream,CompressionWriter],
                ("readInto"|"readExact",1)=>return &[ByteViewMut],
                ("write"|"writeAll",1)=>return &[ByteView,ByteViewMut],
                _=>{},
            }
        }
        if self.module=="std:compression"{use crate::native_signatures::NativeTypeId::*;match(self.name,position){("__readerReadInto"|"__readerReadExact",1)=>return &[ByteViewMut],("__writerWrite"|"__writerWriteAll",1)=>return &[ByteView,ByteViewMut],("__copy",0)=>return &[CompressionReader],("__copy",1)=>return &[CompressionWriter],_=>{}}}
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
        if self.module == "std:fs" && position == 1 && matches!(self.name, "write" | "writeAll" | "writeFile" | "appendFile") {
            return &[ByteView, ByteViewMut];
        }
        if self.module == "std:bytes" {
            if matches!(self.name,"concat"|"equals"|"compare"|"indexOf") && position<=1 {return &[ByteView,ByteViewMut];}
            if self.name=="copyInto" {return match position {0=>&[ByteViewMut],2=>&[ByteView,ByteViewMut],_=>&[]};}
        }
        if self.module != "std:bytes" || position != 0 {
            return &[];
        }
        match self.name {
            // A view of a view borrows the parent view for as long as it lives.
            "length" | "at" | "copy" | "toUtf8" | "view"
            | "toHex" | "toBase64" | "toBase64Url" | "toUtf8Lossy" | "toUtf16"
            | "readInt" | "readUint" | "readFloat32" | "readFloat64" => &[ByteView, ByteViewMut],
            "set" | "viewMut" | "copyWithin" | "writeInt" | "writeUint" | "writeFloat32" | "writeFloat64" => &[ByteViewMut],
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
    crate::stdlib::process::bindings::SPAWN,
    crate::stdlib::process::bindings::RUN,
    crate::stdlib::process::bindings::RUN_POLL,
    crate::stdlib::process::bindings::ID,
    crate::stdlib::process::bindings::TAKE_STDIN,
    crate::stdlib::process::bindings::TAKE_STDOUT,
    crate::stdlib::process::bindings::TAKE_STDERR,
    crate::stdlib::process::bindings::WAIT,
    crate::stdlib::process::bindings::TRY_WAIT,
    crate::stdlib::process::bindings::KILL,
 crate::stdlib::net::bindings::LISTEN,crate::stdlib::net::bindings::LISTEN_POLL,crate::stdlib::net::bindings::CONNECT,crate::stdlib::net::bindings::CONNECT_POLL,crate::stdlib::net::bindings::BIND_UDP,crate::stdlib::net::bindings::BIND_UDP_POLL,crate::stdlib::net::bindings::ACCEPT,crate::stdlib::net::bindings::LISTENER_ADDRESS,crate::stdlib::net::bindings::LISTENER_CLOSE,crate::stdlib::net::bindings::STREAM_LOCAL,crate::stdlib::net::bindings::STREAM_PEER,crate::stdlib::net::bindings::NO_DELAY,crate::stdlib::net::bindings::SHUTDOWN,crate::stdlib::net::bindings::SPLIT,crate::stdlib::net::bindings::STREAM_CLOSE,crate::stdlib::net::bindings::SEND_TO,crate::stdlib::net::bindings::RECEIVE_FROM,crate::stdlib::net::bindings::UDP_ADDRESS,crate::stdlib::net::bindings::UDP_CLOSE,
 crate::stdlib::net::bindings::PARSE_IP,crate::stdlib::net::bindings::FORMAT_IP,crate::stdlib::net::bindings::PARSE_ADDRESS,crate::stdlib::net::bindings::FORMAT_ADDRESS,crate::stdlib::net::bindings::IS_IPV4,crate::stdlib::net::bindings::IS_IPV6,crate::stdlib::net::bindings::LOOKUP,crate::stdlib::net::bindings::LOOKUP_POLL,
    crate::stdlib::compression::bindings::COMPRESSREADER,
    crate::stdlib::compression::bindings::POLL_COMPRESSREADER,
    crate::stdlib::compression::bindings::DECOMPRESSREADER,
    crate::stdlib::compression::bindings::POLL_DECOMPRESSREADER,
    crate::stdlib::compression::bindings::COMPRESSWRITER,
    crate::stdlib::compression::bindings::POLL_COMPRESSWRITER,
    crate::stdlib::compression::bindings::DECOMPRESSWRITER,
    crate::stdlib::compression::bindings::POLL_DECOMPRESSWRITER,
    crate::stdlib::compression::bindings::READER_READ,
    crate::stdlib::compression::bindings::READER_READINTO,
    crate::stdlib::compression::bindings::READER_READEXACT,
    crate::stdlib::compression::bindings::READER_READALL,
    crate::stdlib::compression::bindings::READER_FINISH,
    crate::stdlib::compression::bindings::READER_CLOSE,
    crate::stdlib::compression::bindings::WRITER_WRITE,
    crate::stdlib::compression::bindings::WRITER_WRITEALL,
    crate::stdlib::compression::bindings::WRITER_FLUSH,
    crate::stdlib::compression::bindings::WRITER_FINISH,
    crate::stdlib::compression::bindings::WRITER_CLOSE,
    crate::stdlib::compression::bindings::COPY,
    crate::stdlib::compression::bindings::GZIP,
    crate::stdlib::compression::bindings::POLL_GZIP,
    crate::stdlib::compression::bindings::GUNZIP,
    crate::stdlib::compression::bindings::POLL_GUNZIP,
    crate::stdlib::compression::bindings::DEFLATE,
    crate::stdlib::compression::bindings::POLL_DEFLATE,
    crate::stdlib::compression::bindings::INFLATE,
    crate::stdlib::compression::bindings::POLL_INFLATE,
    crate::stdlib::compression::bindings::DEFLATERAW,
    crate::stdlib::compression::bindings::POLL_DEFLATERAW,
    crate::stdlib::compression::bindings::INFLATERAW,
    crate::stdlib::compression::bindings::POLL_INFLATERAW,

    crate::stdlib::io::bindings::READER,
    crate::stdlib::io::bindings::WRITER,
    crate::stdlib::io::bindings::STDOUT,
    crate::stdlib::io::bindings::STDERR,
    crate::stdlib::io::bindings::STDIN,
    crate::stdlib::io::bindings::READ,
    crate::stdlib::io::bindings::READ_POLL,
    crate::stdlib::io::bindings::READ_INTO,
    crate::stdlib::io::bindings::READ_INTO_POLL,
    crate::stdlib::io::bindings::READ_EXACT,
    crate::stdlib::io::bindings::READ_EXACT_POLL,
    crate::stdlib::io::bindings::READ_ALL,
    crate::stdlib::io::bindings::READ_ALL_POLL,
    crate::stdlib::io::bindings::WRITE,
    crate::stdlib::io::bindings::WRITE_POLL,
    crate::stdlib::io::bindings::WRITE_ALL,
    crate::stdlib::io::bindings::WRITE_ALL_POLL,
    crate::stdlib::io::bindings::FLUSH,
    crate::stdlib::io::bindings::FLUSH_POLL,
    crate::stdlib::io::bindings::COPY,
    crate::stdlib::io::bindings::COPY_POLL,
    crate::stdlib::io::bindings::LINES,
    crate::stdlib::io::bindings::READ_LINE,
    crate::stdlib::io::bindings::READ_LINE_POLL,
    crate::stdlib::io::bindings::CLOSE,
    crate::stdlib::io::bindings::CLOSE_POLL,
    crate::stdlib::io::bindings::READ_STDIN,
    crate::stdlib::io::bindings::READ_LINE_STDIN,
    crate::stdlib::io::bindings::BODY,

    crate::stdlib::cache::SHARED_CREATE, crate::stdlib::cache::SHARED_GET, crate::stdlib::cache::SHARED_PEEK, crate::stdlib::cache::SHARED_SET, crate::stdlib::cache::SHARED_REMOVE, crate::stdlib::cache::SHARED_SIZE, crate::stdlib::cache::SHARED_CLEAR, crate::stdlib::cache::SHARE, crate::stdlib::cache::CREATE, crate::stdlib::cache::GET, crate::stdlib::cache::PEEK, crate::stdlib::cache::SET, crate::stdlib::cache::REMOVE, crate::stdlib::cache::SIZE, crate::stdlib::cache::CLEAR,
    crate::stdlib::sql::CONNECT,
    crate::stdlib::sql::CONNECT_POLL,
    crate::stdlib::sql::BEGIN,
    crate::stdlib::sql::CLOSE,
    crate::stdlib::sql::MIGRATE,
    crate::stdlib::sql::EXECUTE,
    crate::stdlib::sql::QUERY,
    crate::stdlib::sql::CURSOR,
    crate::stdlib::sql::CURSORNEXT,
    crate::stdlib::sql::CURSORCLOSE,
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
    crate::stdlib::http::GETCOOKIE,
    crate::stdlib::http::SETCOOKIE,
    crate::stdlib::http::DELETECOOKIE,
    crate::stdlib::http::PUBLIC_SETCOOKIE,
    crate::stdlib::http::PUBLIC_DELETECOOKIE,
    crate::stdlib::http::CONTEXTGET,
    crate::stdlib::http::CONTEXTSET,
    crate::stdlib::http::CONTEXTREQUESTTEXT,
    crate::stdlib::http::CONTEXTREQUESTBODY,
    crate::stdlib::http::CONTEXTREQUESTFORM,
    crate::stdlib::http::CONTEXTREQUESTMULTIPART,
    crate::stdlib::http::ROUTER,
    crate::stdlib::http::ROUTERCONFIGURE,
    crate::stdlib::http::ROUTERSTATIC,
    crate::stdlib::http::ROUTERROUTE,
    crate::stdlib::http::ROUTERUSE,
    crate::stdlib::http::ROUTERUSEPATH,
    crate::stdlib::http::ROUTERLISTEN,
    crate::stdlib::http::CONTEXTTEXT,
    crate::stdlib::http::CONTEXTNEXT,
    crate::stdlib::http::CONTEXTHTML,
    crate::stdlib::http::CONTEXTJSON,
    crate::stdlib::http::CONTEXTHEADER,
    crate::stdlib::http::CONTEXTSTATUS,
    crate::stdlib::http::APP,
    crate::stdlib::http::CONFIGURE,
    crate::stdlib::http::CLIENT,
    crate::stdlib::http::REQUEST,
    crate::stdlib::http::REQUEST_FORM,
    crate::stdlib::http::REQUEST_MULTIPART,
    crate::stdlib::http::REQUEST_BYTES,
    crate::stdlib::http::REQUEST_STREAM,
    crate::stdlib::http::REQUEST_CHANNEL,
    crate::stdlib::http::TEXT,
    crate::stdlib::http::HTML,
    crate::stdlib::http::JSON,
    crate::stdlib::http::STREAM,
    crate::stdlib::http::STREAMCHANNEL,
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
    crate::stdlib::http::FORM,
    crate::stdlib::http::MULTIPART,
    crate::stdlib::http::REQUESTTEXT,
    crate::stdlib::http::RESPONSEHEADER,
    crate::stdlib::http::RESPONSESETHEADER,
    crate::stdlib::http::RESPONSEBODY,
    crate::stdlib::http::RESPONSETEXT,
    crate::stdlib::http::METHOD,
    crate::stdlib::http::URL,
    crate::stdlib::http::PARAM,
    crate::stdlib::http::QUERY,
    crate::stdlib::http::QUERIES,
    crate::stdlib::http::STATUS,
    crate::stdlib::http::PUBLIC_TEXT,
    crate::stdlib::http::PUBLIC_HTML,
    crate::stdlib::http::PUBLIC_JSON,
    crate::stdlib::http::PUBLIC_REQUEST,
    crate::stdlib::http::CORS,crate::stdlib::http::PUBLIC_CORS,crate::stdlib::http::SECURE_HEADERS,crate::stdlib::http::COMPRESS,
    crate::stdlib::http::PUBLIC_REQUEST_FORM,
    crate::stdlib::http::PUBLIC_REQUEST_MULTIPART,
    crate::stdlib::http::PUBLIC_REQUEST_BYTES,
    crate::stdlib::http::PUBLIC_REQUEST_STREAM,
    crate::stdlib::http::PUBLIC_REQUEST_CHANNEL,
    crate::stdlib::http::PUBLIC_STREAM,
    crate::stdlib::http::PUBLIC_STREAMCHANNEL,
    crate::stdlib::futures::DELAY,
    crate::stdlib::futures::YIELD,
    crate::stdlib::futures::INTERVAL,
    crate::stdlib::futures::TICK,
    crate::stdlib::futures::TOKEN,
    crate::stdlib::futures::TOKEN_SHARE,
    crate::stdlib::futures::TOKEN_CHILD,
    crate::stdlib::futures::TOKEN_CANCEL,
    crate::stdlib::futures::TOKEN_IS_CANCELLED,
    crate::stdlib::futures::TOKEN_CANCELLED,
    crate::stdlib::futures::ALL,
    crate::stdlib::futures::ALL_POLL,
    crate::stdlib::futures::TRY_SEND,
    crate::stdlib::futures::TRY_RECEIVE,
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
    crate::stdlib::threads::POOL_SUBMISSION,
    crate::stdlib::threads::TASK_ASYNC,
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
    crate::stdlib::date::PARSE_TIME,
    crate::stdlib::date::FORMAT_TIME,
    crate::stdlib::date::COMPARE_TIMES,
    crate::stdlib::date::PARSE_ISO,
    crate::stdlib::date::FORMAT_ISO,
    crate::stdlib::date::NOW,
    crate::stdlib::date::START_OF_SECOND,
    crate::stdlib::date::FROM_UNIX_MILLISECONDS,
    crate::stdlib::date::TO_UNIX_MILLISECONDS,
    crate::stdlib::date::FROM_UNIX_SECONDS,
    crate::stdlib::date::TO_UNIX_SECONDS,
    crate::stdlib::date::COMPARE_INSTANTS,
    crate::stdlib::date::DURATION_FROM_NANOSECONDS,
    crate::stdlib::date::DURATION_FROM_MILLISECONDS,
    crate::stdlib::date::DURATION_FROM_SECONDS,
    crate::stdlib::date::DURATION_TO_NANOSECONDS,
    crate::stdlib::date::DURATION_TO_MILLISECONDS,
    crate::stdlib::date::ADD_DURATIONS,
    crate::stdlib::date::SUBTRACT_DURATIONS,
    crate::stdlib::date::ADD_DURATION,
    crate::stdlib::date::DIFFERENCE,
    crate::stdlib::date::PARSE_TIME_ZONE,
    crate::stdlib::date::FIXED_OFFSET,
    crate::stdlib::date::ZONE_NAME,
    crate::stdlib::date::OFFSET_SECONDS,
    crate::stdlib::date::LOCAL_DATE,
    crate::stdlib::date::LOCAL_TIME,
    crate::stdlib::date::RESOLVE_DATE_TIME,
    crate::stdlib::date::PARSE_DATE,
    crate::stdlib::date::FORMAT_DATE,
    crate::stdlib::date::FORMAT,
    crate::stdlib::date::ADD_DAYS,
    crate::stdlib::date::ADD_MONTHS,
    crate::stdlib::date::DIFFERENCE_IN_CALENDAR_DAYS,
    crate::stdlib::date::COMPARE_ASC,
    crate::stdlib::date::IS_LEAP_YEAR,
    crate::stdlib::date::GET_DAY,
    crate::stdlib::date::START_OF_YEAR,
    crate::stdlib::date::END_OF_YEAR,
    crate::stdlib::date::GET_HOURS,
    crate::stdlib::date::GET_MINUTES,
    crate::stdlib::date::GET_SECONDS,
    crate::stdlib::date::GET_NANOSECONDS,
    crate::stdlib::date::GET_YEAR,
    crate::stdlib::date::GET_MONTH,
    crate::stdlib::date::GET_DATE,
    crate::stdlib::date::START_OF_MONTH,
    crate::stdlib::date::END_OF_MONTH,
    crate::stdlib::ui::LOCATION,crate::stdlib::ui::NAVIGATE,crate::stdlib::ui::TRAVERSE,crate::stdlib::ui::LINK,
    crate::stdlib::ui::COMPUTED_NEW,crate::stdlib::ui::COMPUTED_GET,crate::stdlib::ui::SIGNAL_NEW,crate::stdlib::ui::HANDLE_NEW,crate::stdlib::ui::GET,crate::stdlib::ui::SET,crate::stdlib::ui::OWN_STATE,crate::stdlib::ui::PRESS_BUTTON,
    crate::stdlib::ui::READ_ONLY_BINDING,
    crate::stdlib::ui::SVG,crate::stdlib::ui::PATH,crate::stdlib::ui::PATH_BINDING,crate::stdlib::ui::VALIDATE_ROOT,
    crate::stdlib::ui::USE_ID,crate::stdlib::ui::SEMANTIC_BINDING,crate::stdlib::ui::CLASS_BINDING,crate::stdlib::ui::VARIANT_BINDING,crate::stdlib::ui::HIDDEN_BINDING,
    crate::stdlib::ui::TEXT_BINDING,
    crate::stdlib::ui::BRANCH,crate::stdlib::ui::ROUTE,
    crate::stdlib::ui::INPUT,crate::stdlib::ui::INPUT_CHANGE,
    crate::stdlib::ui::TEXT,crate::stdlib::ui::COPY_BUTTON,
    crate::stdlib::ui::VIEW,crate::stdlib::ui::VISIBILITY,
    crate::stdlib::ui::THEME,
    crate::stdlib::ui::THEME_MODE,
    crate::stdlib::ui::SEMANTICS,
    crate::stdlib::ui::DISABLED_BINDING,
    crate::stdlib::ui::BUTTON,
    crate::stdlib::ui::DOCUMENT,
    crate::stdlib::ui::RUN,
    crate::stdlib::signals::LISTEN,
    crate::stdlib::locale::PARSE,
    crate::stdlib::locale::FORMAT_LOCALE,
    crate::stdlib::locale::CREATE_COLLATOR,
    crate::stdlib::locale::COMPARE,
    crate::stdlib::locale::STRING_COMPARE,
    crate::stdlib::locale::CREATE_SEGMENTER,
    crate::stdlib::locale::SEGMENT,
    crate::stdlib::locale::CREATE_NUMBER,
    crate::stdlib::locale::FORMAT_NUMBER,
    crate::stdlib::regex::COMPILE,
    crate::stdlib::regex::ESCAPE,
    crate::stdlib::regex::IS_MATCH,
    crate::stdlib::regex::FIND,
    crate::stdlib::regex::FIND_ALL,
    crate::stdlib::regex::CAPTURES,
    crate::stdlib::regex::CAPTURES_ALL,
    crate::stdlib::regex::REPLACE,
    crate::stdlib::regex::REPLACE_ALL,
    crate::stdlib::regex::SPLIT,
    crate::stdlib::regex::STRING_MATCH,
    crate::stdlib::regex::STRING_MATCH_ALL,
    crate::stdlib::regex::STRING_SEARCH,
    crate::stdlib::url::PARSE,
    crate::stdlib::url::RESOLVE,
    crate::stdlib::url::FORMAT,
    crate::stdlib::url::HREF,
    crate::stdlib::url::SET_HREF,
    crate::stdlib::url::PROTOCOL,
    crate::stdlib::url::SET_PROTOCOL,
    crate::stdlib::url::USERNAME,
    crate::stdlib::url::SET_USERNAME,
    crate::stdlib::url::PASSWORD,
    crate::stdlib::url::SET_PASSWORD,
    crate::stdlib::url::HOSTNAME,
    crate::stdlib::url::SET_HOSTNAME,
    crate::stdlib::url::PORT,
    crate::stdlib::url::SET_PORT,
    crate::stdlib::url::PATHNAME,
    crate::stdlib::url::SET_PATHNAME,
    crate::stdlib::url::SEARCH,
    crate::stdlib::url::SET_SEARCH,
    crate::stdlib::url::HASH,
    crate::stdlib::url::SET_HASH,
    crate::stdlib::url::ORIGIN,
    crate::stdlib::url::PARSE_QUERY,
    crate::stdlib::url::FORMAT_QUERY,
    crate::stdlib::url::SEARCH_PARAMS,
    crate::stdlib::url::SET_SEARCH_PARAMS,
    crate::stdlib::url::GET,
    crate::stdlib::url::GET_ALL,
    crate::stdlib::url::HAS,
    crate::stdlib::url::APPEND,
    crate::stdlib::url::SET,
    crate::stdlib::url::DELETE,
    crate::stdlib::url::ENTRIES,
    crate::stdlib::url::SORT,
    crate::stdlib::url::PATH_TO_FILE_URL,
    crate::stdlib::url::FILE_URL_TO_PATH,
    crate::stdlib::url::ENCODE_COMPONENT,
    crate::stdlib::url::DECODE_COMPONENT,
    crate::stdlib::signals::RECEIVE,
    crate::stdlib::signals::POLL,
    crate::stdlib::crypto::RANDOM_BYTES,
    crate::stdlib::crypto::SHA256,
    crate::stdlib::crypto::SHA384,
    crate::stdlib::crypto::SHA512,
    crate::stdlib::crypto::CREATE_HASHER,
    crate::stdlib::crypto::UPDATE,
    crate::stdlib::crypto::FINALIZE,
    crate::stdlib::crypto::HMAC,
    crate::stdlib::crypto::VERIFY_HMAC,
    crate::stdlib::crypto::HKDF,
    crate::stdlib::crypto::RANDOM_INT,
    crate::stdlib::crypto::HASH_PASSWORD,
    crate::stdlib::crypto::POLL_HASH_PASSWORD,
    crate::stdlib::crypto::VERIFY_PASSWORD,
    crate::stdlib::crypto::POLL_VERIFY_PASSWORD,
    crate::stdlib::crypto::VERIFY_BOUNDED,
    crate::stdlib::crypto::POLL_VERIFY_BOUNDED,

    crate::stdlib::crypto::TIMING_SAFE_EQUAL,
    crate::stdlib::uuid::PARSE_UUID,
    crate::stdlib::uuid::FORMAT_UUID,
    crate::stdlib::uuid::FROM_BYTES,
    crate::stdlib::uuid::TO_BYTES,
    crate::stdlib::uuid::COMPARE,
    crate::stdlib::uuid::VERSION,
    crate::stdlib::uuid::RANDOM_UUID,
    crate::stdlib::decimal::PARSE_DECIMAL,
    crate::stdlib::decimal::FROM_INT,
    crate::stdlib::decimal::FORMAT,
    crate::stdlib::decimal::FORMAT_FIXED,
    crate::stdlib::decimal::ADD,
    crate::stdlib::decimal::SUBTRACT,
    crate::stdlib::decimal::MULTIPLY,
    crate::stdlib::decimal::DIVIDE_EXACT,
    crate::stdlib::decimal::COMPARE,
    crate::stdlib::decimal::TO_INT,
    crate::stdlib::decimal::TO_NUMBER,
    crate::stdlib::iterator::ARRAY_CURSOR,
    crate::stdlib::iterator::ARRAY_NEXT,
    crate::stdlib::numeric::TO_INT,
    crate::stdlib::numeric::TO_NUMBER,
    crate::stdlib::numeric::TO_FLOAT32,
    crate::stdlib::numeric::CHECK_SIGNED,
    crate::stdlib::numeric::CHECK_UNSIGNED,
    crate::stdlib::numeric::PARSE_INT,
    crate::stdlib::numeric::PARSE_NUMBER,
    crate::stdlib::numeric::CHECKED_ADD,
    crate::stdlib::numeric::CHECKED_SUBTRACT,
    crate::stdlib::numeric::CHECKED_MULTIPLY,
    crate::stdlib::numeric::CHECKED_DIVIDE,
    crate::stdlib::numeric::CHECKED_REMAINDER,

    crate::stdlib::env::bindings::GET,
    crate::stdlib::env::bindings::ARGS,
    crate::stdlib::env::bindings::VARS,
    crate::stdlib::env::bindings::CURRENT_DIR,
    crate::stdlib::env::bindings::CURRENT_EXE,
    crate::stdlib::env::bindings::TEMP_DIR,
    crate::stdlib::env::bindings::HOME_DIR,
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
    crate::stdlib::fs::bindings::OPEN_FILE,
    crate::stdlib::fs::bindings::STAT,
    crate::stdlib::fs::bindings::LSTAT,
    crate::stdlib::fs::bindings::FILE_STAT,
    crate::stdlib::fs::bindings::READ_DIR,
    crate::stdlib::fs::bindings::MKDIR,
    crate::stdlib::fs::bindings::MKDIR_ALL,
    crate::stdlib::fs::bindings::REMOVE_FILE,
    crate::stdlib::fs::bindings::REMOVE_DIR,
    crate::stdlib::fs::bindings::REMOVE_DIR_ALL,
    crate::stdlib::fs::bindings::RENAME,
    crate::stdlib::fs::bindings::COPY_FILE,
    crate::stdlib::fs::bindings::HARD_LINK,
    crate::stdlib::fs::bindings::CANONICALIZE,
    crate::stdlib::fs::bindings::READ_LINK,
    crate::stdlib::fs::bindings::APPEND_FILE,
    crate::stdlib::fs::bindings::READ_TEXT_FILE,
    crate::stdlib::fs::bindings::WRITE_TEXT_FILE,
    crate::stdlib::fs::bindings::TRUNCATE,
    crate::stdlib::fs::bindings::SYNC_ALL,
    crate::stdlib::fs::bindings::SYNC_DATA,
    crate::stdlib::fs::bindings::CREATE_TEMP_DIR,
    crate::stdlib::fs::bindings::SEEK,
    crate::stdlib::fs::bindings::POSITION,

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
    crate::stdlib::bytes::bindings::TO_HEX,
    crate::stdlib::bytes::bindings::FROM_HEX,
    crate::stdlib::bytes::bindings::TO_UTF8_LOSSY,
    crate::stdlib::bytes::bindings::TO_BASE64,
    crate::stdlib::bytes::bindings::FROM_BASE64,
    crate::stdlib::bytes::bindings::TO_BASE64_URL,
    crate::stdlib::bytes::bindings::FROM_BASE64_URL,
    crate::stdlib::bytes::bindings::FROM_UTF16,
    crate::stdlib::bytes::bindings::TO_UTF16,
    crate::stdlib::bytes::bindings::CONCAT,
    crate::stdlib::bytes::bindings::COPY_INTO,
    crate::stdlib::bytes::bindings::COPY_WITHIN,
    crate::stdlib::bytes::bindings::EQUALS,
    crate::stdlib::bytes::bindings::COMPARE,
    crate::stdlib::bytes::bindings::INDEX_OF,
    crate::stdlib::bytes::bindings::READ_INT,
    crate::stdlib::bytes::bindings::WRITE_INT,
    crate::stdlib::bytes::bindings::READ_UINT,
    crate::stdlib::bytes::bindings::WRITE_UINT,
    crate::stdlib::bytes::bindings::READ_FLOAT32,
    crate::stdlib::bytes::bindings::WRITE_FLOAT32,
    crate::stdlib::bytes::bindings::READ_FLOAT64,
    crate::stdlib::bytes::bindings::WRITE_FLOAT64,
    crate::stdlib::testing::bindings::CONTAINS_VALUE,
    crate::stdlib::testing::bindings::LENGTH_VALUE,
    crate::stdlib::testing::bindings::HAS_LENGTH,
    crate::stdlib::testing::bindings::NUMERIC_VALUE,
    crate::stdlib::testing::bindings::ASSERTION_DETAILS,
    crate::stdlib::testing::bindings::DEEP_EQUAL,
    crate::stdlib::testing::bindings::SHARE_CALLBACK,
    crate::stdlib::path::bindings::BASENAME,
    crate::stdlib::path::bindings::DIRNAME,
    crate::stdlib::path::bindings::EXTNAME,
    crate::stdlib::path::bindings::IS_ABSOLUTE,
    crate::stdlib::path::bindings::NORMALIZE,
    crate::stdlib::testing::resources::CREATE,
    crate::stdlib::testing::resources::ID,
    crate::stdlib::testing::resources::CLOSE,
    crate::stdlib::testing::resources::RELEASE_COUNT,
    crate::stdlib::testing::resources::RELEASE_ORDER,
    crate::stdlib::testing::resources::LIVE_COUNT,
    crate::stdlib::testing::bindings::CHECK_ASSERTION,
    crate::stdlib::testing::bindings::INVOKE_REGISTRATION,
    crate::stdlib::testing::bindings::INVOKE_SUITE,
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
    crate::stdlib::os::bindings::AVAILABLE_PARALLELISM,
    crate::stdlib::os::bindings::HOSTNAME,
    crate::stdlib::path::bindings::PARSE,
    crate::stdlib::path::bindings::FORMAT,
    crate::stdlib::path::bindings::RESOLVE,
    crate::stdlib::path::bindings::SEPARATOR,
    crate::stdlib::path::bindings::DELIMITER,
    crate::stdlib::path::bindings::JOIN,
    crate::stdlib::path::bindings::RELATIVE,
    crate::stdlib::bytes::bindings::VIEW,
    crate::stdlib::bytes::bindings::VIEW_MUT,
];
/// Checked query method metadata, shared with editor inference.
pub fn url_method(identity:crate::native_signatures::NativeTypeId,member:&str)->Option<&'static NativeBinding>{crate::stdlib::url::method(Some(identity),member)}
/// Computed URL properties have separate read and write contracts, not record fields.
pub fn url_property(identity:crate::native_signatures::NativeTypeId,member:&str)->Option<(&'static NativeBinding,Option<&'static NativeBinding>)>{
    if identity!=crate::native_signatures::NativeTypeId::Url{return None;}
    let property=crate::stdlib::url::property(member)?;Some((&property.getter,property.setter.as_ref()))
}
pub fn url_property_names()->impl Iterator<Item=&'static str>{crate::stdlib::url::PROPERTIES.iter().map(|property|property.name)}

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

/// Public HTTP factories unwrap their checked backend Result during lowering.
pub fn http_unwrapped(name:&str)->bool{crate::stdlib::http::public_unwrapped(name)}

/// State method signatures shared by checking and editor inference.
pub fn ui_method(identity: crate::native_signatures::NativeTypeId, member: &str) -> Option<&'static NativeBinding> {
    if identity==crate::native_signatures::NativeTypeId::UiComputed{return (member=="get").then(||lookup("std:ui-runtime","computedGet").ok()).flatten();}
    if identity!=crate::native_signatures::NativeTypeId::UiSignal || !matches!(member,"get"|"set") {return None;}
    lookup("std:ui-runtime",member).ok()
}

pub fn locale_method(identity:crate::native_signatures::NativeTypeId,member:&str)->Option<&'static NativeBinding>{crate::stdlib::locale::method(Some(identity),member)}
pub fn regex_method(identity:crate::native_signatures::NativeTypeId,member:&str)->Option<&'static NativeBinding>{crate::stdlib::regex::method(Some(identity),member)}

pub fn io_asynchronous(name:&str)->bool{crate::stdlib::io::asynchronous(name)}

pub fn process_method(identity:crate::native_signatures::NativeTypeId,member:&str)->Option<&'static NativeBinding>{crate::stdlib::process::method(Some(identity),member)}

pub fn process_asynchronous(name:&str)->bool{crate::stdlib::process::asynchronous(name)}
pub fn network_method(identity:crate::native_signatures::NativeTypeId,name:&str)->Option<&'static NativeBinding>{crate::stdlib::net::method(Some(identity),name)}
pub fn network_asynchronous(name:&str)->bool{crate::stdlib::net::asynchronous(name)}
pub fn compression_asynchronous(name:&str)->bool{!crate::stdlib::compression::constructor(name.trim_start_matches('_'))}
