//! Native declarations describe public identity separately from callable exports.
//! Opaque declarations expose no source construction or implementation fields.
use crate::native_signatures::{NativeTypeId, Signature};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Representation {
    Opaque,
    Record,
    /// A scoped native value whose storage is borrowed, never independently owned.
    Borrowed(crate::native_return_borrows::BorrowAccess),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability { Inspection, StringConversion, PropertyReflection, ObjectSpread, Equality }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: &'static str,
    pub signature: Signature,
    pub optional: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Declaration {
    pub id: NativeTypeId,
    pub module: &'static str,
    pub name: &'static str,
    pub type_parameters: &'static [&'static str],
    pub representation: Representation,
    /// Explicit clone produces independent storage. This never permits implicit copies.
    pub independent_clone: bool,
    /// Eligibility for current scalar/enum collection keys, not object identity.
    pub collection_key: bool,
    /// The owned backend representation may cross a native thread boundary.
    /// This metadata does not itself expose a source thread API.
    pub thread_transfer: bool,
    pub fields: &'static [Field],
    pub capabilities: &'static [Capability],
}

impl Declaration {
    pub fn borrow_access(&self) -> Option<crate::native_return_borrows::BorrowAccess> {
        match self.representation {
            Representation::Borrowed(access) => Some(access),
            _ => None,
        }
    }
    pub fn source_constructible(&self) -> bool {
        self.representation == Representation::Record
    }
}

static THREAD: Declaration = Declaration {
    id: NativeTypeId::Thread, module: "std:threads", name: "Thread",
    type_parameters: &["T", "E"], representation: Representation::Opaque,
    independent_clone: false, collection_key: false, thread_transfer: false,
    fields: &[], capabilities: &[],
};
static IP_ADDRESS:Declaration=Declaration{id:NativeTypeId::IpAddress,module:"std:net",name:"IpAddress",type_parameters:&[],independent_clone:true,thread_transfer:true,..THREAD};
static SOCKET_ADDRESS:Declaration=Declaration{id:NativeTypeId::SocketAddress,name:"SocketAddress",..IP_ADDRESS};
static TCP_LISTENER:Declaration=Declaration{id:NativeTypeId::TcpListener,module:"std:net",name:"TcpListener",thread_transfer:true,..FILE};
static TCP_STREAM:Declaration=Declaration{id:NativeTypeId::TcpStream,name:"TcpStream",..TCP_LISTENER};
static UDP_SOCKET:Declaration=Declaration{id:NativeTypeId::UdpSocket,name:"UdpSocket",..TCP_LISTENER};
static DATAGRAM:Declaration=Declaration{id:NativeTypeId::Datagram,module:"std:net",name:"Datagram",fields:&[Field{name:"data",signature:Signature::Named(NativeTypeId::Bytes),optional:false},Field{name:"remote",signature:Signature::Named(NativeTypeId::SocketAddress),optional:false}],..SPAWN_ERROR};
static NET_ERROR:Declaration=Declaration{id:NativeTypeId::NetError,module:"std:net",name:"NetError",..FS_ERROR};
static URL: Declaration = Declaration {id:NativeTypeId::Url,module:"std:url",name:"Url",type_parameters:&[],independent_clone:true,thread_transfer:true,..THREAD};
static SEARCH_PARAMS: Declaration = Declaration {id:NativeTypeId::SearchParams,name:"SearchParams",..URL};
static URL_ERROR: Declaration = Declaration {id:NativeTypeId::UrlError,module:"std:url",name:"UrlError",fields:&[
 Field{name:"code",signature:Signature::String,optional:false},Field{name:"operation",signature:Signature::String,optional:false},Field{name:"message",signature:Signature::String,optional:false},
],..SPAWN_ERROR};
static QUERY_ENTRY: Declaration = Declaration {id:NativeTypeId::QueryEntry,module:"std:url",name:"QueryEntry",fields:&[
 Field{name:"name",signature:Signature::String,optional:false},Field{name:"value",signature:Signature::String,optional:false},
],..SPAWN_ERROR};
static LOCALE_LOCALE:Declaration=Declaration{id:NativeTypeId::Locale,module:"std:locale",name:"Locale",type_parameters:&[],thread_transfer:true,..THREAD};
static LOCALE_COLLATOR:Declaration=Declaration{id:NativeTypeId::Collator,module:"std:locale",name:"Collator",type_parameters:&[],thread_transfer:true,..THREAD};
static LOCALE_SEGMENTER:Declaration=Declaration{id:NativeTypeId::Segmenter,module:"std:locale",name:"Segmenter",type_parameters:&[],thread_transfer:true,..THREAD};
static LOCALE_NUMBERFORMAT:Declaration=Declaration{id:NativeTypeId::NumberFormat,module:"std:locale",name:"NumberFormat",type_parameters:&[],thread_transfer:true,..THREAD};
static LOCALE_SEGMENTRANGE:Declaration=Declaration{id:NativeTypeId::SegmentRange,module:"std:locale",name:"SegmentRange",fields:&[Field{name:"start",signature:Signature::Int,optional:false},Field{name:"end",signature:Signature::Int,optional:false}],..SPAWN_ERROR};
static LOCALE_LOCALEERROR:Declaration=Declaration{id:NativeTypeId::LocaleError,module:"std:locale",name:"LocaleError",fields:&[Field{name:"code",signature:Signature::String,optional:false},Field{name:"message",signature:Signature::String,optional:false}],..SPAWN_ERROR};
static LOCALE_COLLATOROPTIONS:Declaration=Declaration{id:NativeTypeId::CollatorOptions,module:"std:locale",name:"CollatorOptions",fields:&[Field{name:"strength",signature:Signature::Union(&[Signature::StringLiteral("primary"),Signature::StringLiteral("secondary"),Signature::StringLiteral("tertiary"),Signature::StringLiteral("quaternary"),Signature::StringLiteral("identical")]),optional:true},Field{name:"alternateHandling",signature:Signature::Union(&[Signature::StringLiteral("nonIgnorable"),Signature::StringLiteral("shifted")]),optional:true},Field{name:"maxVariable",signature:Signature::Union(&[Signature::StringLiteral("space"),Signature::StringLiteral("punctuation"),Signature::StringLiteral("symbol"),Signature::StringLiteral("currency")]),optional:true},Field{name:"caseLevel",signature:Signature::Boolean,optional:true}],..SPAWN_ERROR};
static LOCALE_NUMBERFORMATOPTIONS:Declaration=Declaration{id:NativeTypeId::NumberFormatOptions,module:"std:locale",name:"NumberFormatOptions",fields:&[Field{name:"grouping",signature:Signature::Union(&[Signature::StringLiteral("auto"),Signature::StringLiteral("never"),Signature::StringLiteral("always"),Signature::StringLiteral("min2")]),optional:true},Field{name:"fractionDigits",signature:Signature::Int,optional:true}],..SPAWN_ERROR};
static REGEX: Declaration = Declaration {id:NativeTypeId::Regex,module:"std:regex",name:"Regex",type_parameters:&[],thread_transfer:true,..THREAD};
static REGEX_MATCH: Declaration = Declaration {id:NativeTypeId::RegexMatch,module:"std:regex",name:"RegexMatch",independent_clone:true,fields:&[
 Field{name:"start",signature:Signature::Int,optional:false},Field{name:"end",signature:Signature::Int,optional:false},
],..SPAWN_ERROR};
static REGEX_CAPTURES: Declaration = Declaration {id:NativeTypeId::RegexCaptures,module:"std:regex",name:"RegexCaptures",fields:&[
 Field{name:"match",signature:Signature::Named(NativeTypeId::RegexMatch),optional:false},
 Field{name:"groups",signature:Signature::Array(&Signature::Optional(&Signature::Named(NativeTypeId::RegexMatch))),optional:false},
],..SPAWN_ERROR};
static REGEX_ERROR: Declaration = Declaration {id:NativeTypeId::RegexError,module:"std:regex",name:"RegexError",fields:&[
 Field{name:"code",signature:Signature::String,optional:false},Field{name:"message",signature:Signature::String,optional:false},
],..SPAWN_ERROR};
static REGEX_OPTIONS: Declaration = Declaration {id:NativeTypeId::RegexOptions,module:"std:regex",name:"RegexOptions",fields:&[
 Field{name:"sizeLimit",signature:Signature::Int,optional:true},Field{name:"dfaSizeLimit",signature:Signature::Int,optional:true},Field{name:"nestLimit",signature:Signature::Int,optional:true},
],..SPAWN_ERROR};
static UI_NODE: Declaration = Declaration {
    id: NativeTypeId::UiNode,
    module: "std:ui-runtime",
    name: "Node",
    type_parameters: &[],
    ..THREAD
};
static UI_SIGNAL: Declaration = Declaration { id:NativeTypeId::UiSignal,name:"Signal",type_parameters:&["T"],..UI_NODE };
static UI_COMPUTED:Declaration=Declaration{id:NativeTypeId::UiComputed,name:"Computed",..UI_SIGNAL};
static UI_TEXT_BINDING:Declaration=Declaration{id:NativeTypeId::UiTextBinding,name:"TextBinding",..UI_NODE};
static UI_ERROR: Declaration = Declaration {
    id: NativeTypeId::UiError,
    module: "std:ui-runtime",
    name: "UiError",
    ..ENV_ERROR
};
static LRU_CACHE: Declaration = Declaration {id:NativeTypeId::LruCache,module:"std:cache",name:"LruCache",type_parameters:&["K","V"],..THREAD};
static SHARED_LRU_CACHE: Declaration = Declaration {id:NativeTypeId::SharedLruCache,name:"SharedLruCache",thread_transfer:true,..LRU_CACHE};
static CACHE_ENTRY: Declaration = Declaration {id:NativeTypeId::CacheEntry,module:"std:cache",name:"CacheEntry",independent_clone:true,type_parameters:&["K","V"],fields:&[
 Field{name:"key",signature:Signature::Parameter(0),optional:false},Field{name:"value",signature:Signature::Parameter(1),optional:false},
],..SPAWN_ERROR};
static CACHE_INSERT_ERROR: Declaration = Declaration {id:NativeTypeId::CacheInsertError,name:"CacheInsertError",fields:&[
 Field{name:"code",signature:Signature::String,optional:false},Field{name:"key",signature:Signature::Parameter(0),optional:false},Field{name:"value",signature:Signature::Parameter(1),optional:false},
],..CACHE_ENTRY};
static SQLPOOL:Declaration=Declaration{id:NativeTypeId::SqlPool,module:"std:sql",name:"Pool",type_parameters:&[],thread_transfer:true,..THREAD};
static SQLTRANSACTION:Declaration=Declaration{representation:Representation::Borrowed(crate::native_return_borrows::BorrowAccess::ReadOnly),id:NativeTypeId::SqlTransaction,module:"std:sql",name:"Transaction",type_parameters:&[],thread_transfer:true,..THREAD};
static SQLCURSOR:Declaration=Declaration{id:NativeTypeId::SqlCursor,name:"Cursor",..SQLTRANSACTION};
static SQLROW:Declaration=Declaration{id:NativeTypeId::SqlRow,module:"std:sql",name:"Row",type_parameters:&[],thread_transfer:true,..THREAD};
static SQLERROR:Declaration=Declaration{id:NativeTypeId::SqlError,module:"std:sql",name:"SqlError",type_parameters:&[],thread_transfer:true,..SPAWN_ERROR};
static SQLITECONNECTION:Declaration=Declaration{id:NativeTypeId::SqliteConnection,module:"std:sqlite",name:"Connection",type_parameters:&[],..THREAD};
static SQLITESTATEMENT:Declaration=Declaration{id:NativeTypeId::SqliteStatement,module:"std:sqlite",name:"Statement",type_parameters:&[],representation:Representation::Borrowed(crate::native_return_borrows::BorrowAccess::ReadOnly),..THREAD};
static SQLITETRANSACTION:Declaration=Declaration{id:NativeTypeId::SqliteTransaction,module:"std:sqlite",name:"Transaction",type_parameters:&[],representation:Representation::Borrowed(crate::native_return_borrows::BorrowAccess::Exclusive),..THREAD};
static SQLITEROW:Declaration=Declaration{id:NativeTypeId::SqliteRow,module:"std:sqlite",name:"Row",type_parameters:&[],..THREAD};
static SQLITEERROR:Declaration=Declaration{id:NativeTypeId::SqliteError,module:"std:sqlite",name:"SqliteError",type_parameters:&[],..SPAWN_ERROR};

static HTTPAPP: Declaration = Declaration {id:NativeTypeId::HttpApp,module:"std:http",name:"App",type_parameters:&[],thread_transfer:true,..THREAD};
static HTTPROUTER: Declaration = Declaration {id:NativeTypeId::HttpRouter,name:"Router",type_parameters:&["State","Variables"],..HTTPAPP};
static HTTPCONTEXT: Declaration = Declaration {id:NativeTypeId::HttpContext,name:"Context",type_parameters:&["State","Variables"],fields:&[Field{name:"req",signature:Signature::Named(NativeTypeId::HttpRequest),optional:false},Field{name:"env",signature:Signature::Parameter(0),optional:false},Field{name:"res",signature:Signature::Named(NativeTypeId::HttpResponse),optional:false}],..HTTPAPP};
static HTTPCONTEXTNEXT: Declaration = Declaration {id:NativeTypeId::HttpContextNext,name:"ContextNext",type_parameters:&["State"],..HTTPAPP};
static HTTPREQUEST: Declaration = Declaration {id:NativeTypeId::HttpRequest,module:"std:http",name:"Request",type_parameters:&[],thread_transfer:true,..THREAD};
static HTTPRESPONSE: Declaration = Declaration {id:NativeTypeId::HttpResponse,module:"std:http",name:"Response",type_parameters:&[],thread_transfer:true,..THREAD};
static HTTPBODY: Declaration = Declaration {id:NativeTypeId::HttpBody,module:"std:http",name:"Body",type_parameters:&[],thread_transfer:true,..THREAD};
static HTTPCLIENT: Declaration = Declaration {id:NativeTypeId::HttpClient,module:"std:http",name:"Client",type_parameters:&[],thread_transfer:true,..THREAD};
static HTTPSERVER: Declaration = Declaration {id:NativeTypeId::HttpServer,module:"std:http",name:"Server",type_parameters:&[],thread_transfer:true,..THREAD};
static HTTPNEXT: Declaration = Declaration {id:NativeTypeId::HttpNext,module:"std:http",name:"Next",type_parameters:&[],thread_transfer:true,..THREAD};
static HTTPERROR:Declaration=Declaration{id:NativeTypeId::HttpError,module:"std:http",name:"HttpError",independent_clone:true,..SPAWN_ERROR};

static HTTPTLSOPTIONS: Declaration = Declaration {id:NativeTypeId::HttpTlsOptions,module:"std:http",name:"TlsOptions",fields:&[
 Field{name:"cert",signature:Signature::String,optional:false},Field{name:"key",signature:Signature::String,optional:false},
],..SPAWN_ERROR};
static HTTPFORMFIELD: Declaration = Declaration {id:NativeTypeId::HttpFormField,module:"std:http",name:"FormField",fields:&[
 Field{name:"name",signature:Signature::String,optional:false},Field{name:"value",signature:Signature::String,optional:false},
],..SPAWN_ERROR};
static HTTPMULTIPARTPART: Declaration = Declaration {id:NativeTypeId::HttpMultipartPart,module:"std:http",name:"MultipartPart",fields:&[
 Field{name:"name",signature:Signature::String,optional:false},Field{name:"fileName",signature:Signature::String,optional:true},Field{name:"contentType",signature:Signature::String,optional:true},Field{name:"data",signature:Signature::Named(NativeTypeId::Bytes),optional:false},
],..SPAWN_ERROR};
static HTTPCOOKIEOPTIONS: Declaration = Declaration {id:NativeTypeId::HttpCookieOptions,module:"std:http",name:"CookieOptions",fields:&[
 Field{name:"path",signature:Signature::String,optional:true},Field{name:"domain",signature:Signature::String,optional:true},Field{name:"maxAge",signature:Signature::Int,optional:true},Field{name:"secure",signature:Signature::Boolean,optional:true},Field{name:"httpOnly",signature:Signature::Boolean,optional:true},Field{name:"sameSite",signature:Signature::String,optional:true},Field{name:"partitioned",signature:Signature::Boolean,optional:true},
],..SPAWN_ERROR};
static HTTPSERVEROPTIONS: Declaration = Declaration {id:NativeTypeId::HttpServerOptions,module:"std:http",name:"ServerOptions",fields:&[
 Field{name:"bodyLimit",signature:Signature::Int,optional:false},
 Field{name:"maxConnections",signature:Signature::Int,optional:false},
 Field{name:"maxRequests",signature:Signature::Int,optional:false},
 Field{name:"requestTimeoutMs",signature:Signature::Int,optional:false},
 Field{name:"shutdownTimeoutMs",signature:Signature::Int,optional:false},
 Field{name:"handshakeTimeoutMs",signature:Signature::Int,optional:false},
 Field{name:"requestsPerSecond",signature:Signature::Int,optional:false},
],..SPAWN_ERROR};
static HTTPCLIENTOPTIONS: Declaration = Declaration {id:NativeTypeId::HttpClientOptions,module:"std:http",name:"ClientOptions",fields:&[
 Field{name:"ca",signature:Signature::String,optional:false},
],..SPAWN_ERROR};

static JOB: Declaration = Declaration {id:NativeTypeId::Job,module:"std:futures",name:"Job",..THREAD};
static ASYNC_ERROR: Declaration = Declaration {id:NativeTypeId::AsyncError,module:"std:futures",name:"AsyncError",independent_clone:true,type_parameters:&["E"],fields:&[
    Field{name:"code",signature:Signature::String,optional:false},
    Field{name:"cause",signature:Signature::Parameter(0),optional:true},
],..SPAWN_ERROR};
static ASYNC_CHANNEL: Declaration = Declaration {id:NativeTypeId::AsyncChannel,module:"std:futures",name:"Channel",..CHANNEL};
static ASYNC_SENDER: Declaration = Declaration {id:NativeTypeId::AsyncSender,module:"std:futures",name:"Sender",..SENDER};
static INTERVAL: Declaration = Declaration {id:NativeTypeId::Interval,module:"std:futures",name:"Interval",type_parameters:&[],..THREAD};
static CANCELLATION_TOKEN: Declaration = Declaration {id:NativeTypeId::CancellationToken,module:"std:futures",name:"CancellationToken",type_parameters:&[],thread_transfer:true,..THREAD};
static COMPRESSION_READER:Declaration=Declaration{id:NativeTypeId::CompressionReader,module:"std:compression",name:"CompressionReader",thread_transfer:true,..FILE};
static COMPRESSION_WRITER:Declaration=Declaration{id:NativeTypeId::CompressionWriter,module:"std:compression",name:"CompressionWriter",thread_transfer:true,..FILE};
static COMPRESSION_OPTIONS:Declaration=Declaration{id:NativeTypeId::CompressionOptions,module:"std:compression",name:"CompressionOptions",fields:&[Field{name:"level",signature:Signature::Int,optional:true}],..SPAWN_ERROR};
static DECOMPRESSION_OPTIONS:Declaration=Declaration{id:NativeTypeId::DecompressionOptions,module:"std:compression",name:"DecompressionOptions",fields:&[Field{name:"maxOutputBytes",signature:Signature::Int,optional:true}],..SPAWN_ERROR};
static COMPRESSION_ERROR:Declaration=Declaration{id:NativeTypeId::CompressionError,module:"std:compression",name:"CompressionError",fields:&[Field{name:"code",signature:Signature::Union(&[Signature::StringLiteral("range"),Signature::StringLiteral("format"),Signature::StringLiteral("codec"),Signature::StringLiteral("truncated"),Signature::StringLiteral("trailing-data"),Signature::StringLiteral("output-limit"),Signature::StringLiteral("allocation"),Signature::StringLiteral("closed")]),optional:false},Field{name:"operation",signature:Signature::String,optional:false},Field{name:"message",signature:Signature::String,optional:false}],..SPAWN_ERROR};
static POOL_SUBMISSION: Declaration = Declaration {id:NativeTypeId::PoolSubmission,name:"__PoolSubmission",type_parameters:&[],thread_transfer:true,..THREAD};
static POOL: Declaration = Declaration { id: NativeTypeId::ThreadPool, name: "ThreadPool", type_parameters: &[], ..THREAD };
static TASK_CONTEXT: Declaration = Declaration { id: NativeTypeId::TaskContext, name: "TaskContext", type_parameters: &[], ..THREAD };
static CHANNEL: Declaration = Declaration { id: NativeTypeId::Channel, name: "Channel", type_parameters: &["T"], thread_transfer: true, ..THREAD };
static SENDER: Declaration = Declaration { id: NativeTypeId::Sender, name: "Sender", ..CHANNEL };
static CHANNEL_ERROR: Declaration = Declaration { id: NativeTypeId::ChannelError, name: "ChannelError", fields: &[Field { name: "code", signature: Signature::String, optional: false }], ..SPAWN_ERROR };
static SEND_ERROR: Declaration = Declaration { id: NativeTypeId::SendError, name: "SendError", independent_clone: true, type_parameters: &["T"], fields: &[
    Field { name: "code", signature: Signature::String, optional: false },
    Field { name: "value", signature: Signature::Parameter(0), optional: false },
], ..SPAWN_ERROR };
static SCOPED_THREAD: Declaration = Declaration {
    id: NativeTypeId::ScopedThread, name: "ScopedThread", ..THREAD
};
static SPAWN_ERROR: Declaration = Declaration {
    id: NativeTypeId::SpawnError, module: "std:threads", name: "SpawnError",
    type_parameters: &[], representation: Representation::Record,
    independent_clone: false, collection_key: false, thread_transfer: true,
    fields: &[
        Field { name: "code", signature: Signature::String, optional: false },
        Field { name: "message", signature: Signature::String, optional: false },
    ], capabilities: &[],
};
static POOL_OPTIONS: Declaration = Declaration {
    id: NativeTypeId::ThreadPoolOptions, name: "ThreadPoolOptions",
    fields: &[
        Field { name: "workers", signature: Signature::Int, optional: false },
        Field { name: "capacity", signature: Signature::Int, optional: false },
    ], ..SPAWN_ERROR
};
static POOL_STATS: Declaration = Declaration {
    id: NativeTypeId::ThreadPoolStats, name: "ThreadPoolStats",
    fields: &[
        Field { name: "workers", signature: Signature::Int, optional: false },
        Field { name: "capacity", signature: Signature::Int, optional: false },
        Field { name: "queued", signature: Signature::Int, optional: false },
        Field { name: "running", signature: Signature::Int, optional: false },
        Field { name: "completed", signature: Signature::Int, optional: false },
        Field { name: "outstanding", signature: Signature::Int, optional: false },
    ], ..SPAWN_ERROR
};
static THREAD_ERROR: Declaration = Declaration {
    id: NativeTypeId::ThreadError, name: "ThreadError", type_parameters: &["E"],
    fields: &[
        Field { name: "code", signature: Signature::String, optional: false },
        Field { name: "error", signature: Signature::Parameter(0), optional: true },
    ], ..SPAWN_ERROR
};

static TRACKED_RESOURCE: Declaration = Declaration { id: NativeTypeId::TrackedResource, module: "std:testing-runtime", name: "TrackedResource", ..TEST_REGISTRY };

static TEST_REGISTRY: Declaration = Declaration {
    id: NativeTypeId::TestRegistry, module: "std:testing-runtime", name: "TestRegistry",
    type_parameters: &[], representation: Representation::Opaque,
    independent_clone: false, collection_key: false, thread_transfer: false,
    fields: &[], capabilities: &[],
};

static SCHEMA_PATH: Declaration = Declaration {
    id: NativeTypeId::SchemaPath, module: "std:schema", name: "SchemaPath",
    fields: &[
        Field { name: "field", signature: Signature::String, optional: true },
        Field { name: "index", signature: Signature::Int, optional: true },
    ], ..FS_ERROR
};
static SCHEMA_ISSUE: Declaration = Declaration {
    id: NativeTypeId::SchemaIssue, module: "std:schema", name: "SchemaIssue",
    fields: &[
        Field { name: "code", signature: Signature::String, optional: false },
        Field { name: "path", signature: Signature::Array(&Signature::Named(NativeTypeId::SchemaPath)), optional: false },
        Field { name: "pathTruncated", signature: Signature::Boolean, optional: false },
        Field { name: "expected", signature: Signature::String, optional: false },
        Field { name: "observed", signature: Signature::String, optional: false },
        Field { name: "message", signature: Signature::String, optional: false },
    ], ..FS_ERROR
};
static SCHEMA_EXPORT_ERROR: Declaration = Declaration {
    id: NativeTypeId::SchemaExportError, module: "std:schema", name: "SchemaExportError",
    fields: &[
        Field { name: "code", signature: Signature::String, optional: false },
        Field { name: "path", signature: Signature::String, optional: false },
        Field { name: "reason", signature: Signature::String, optional: false },
    ], ..FS_ERROR
};
static SCHEMA_CONSTRUCTION_ERROR: Declaration = Declaration {
    id: NativeTypeId::SchemaConstructionError, module: "std:schema", name: "SchemaConstructionError",
    fields: &[
        Field { name: "code", signature: Signature::String, optional: false },
        Field { name: "operation", signature: Signature::String, optional: false },
        Field { name: "limit", signature: Signature::Union(&[Signature::Int,Signature::Number]), optional: false },
    ], ..FS_ERROR
};
static VALIDATION_ERROR: Declaration = Declaration {
    id: NativeTypeId::ValidationError, module: "std:schema", name: "ValidationError",
    fields: &[
        Field { name: "issues", signature: Signature::Array(&Signature::Named(NativeTypeId::SchemaIssue)), optional: false },
        Field { name: "truncated", signature: Signature::Boolean, optional: false },
    ], ..FS_ERROR
};

// The callback receives a mutable borrow of this collector. Its generic
// argument is the validated output domain used for checked issue paths.
static VALIDATION_CONTEXT: Declaration = Declaration {
    id: NativeTypeId::ValidationContext, module: "std:schema", name: "ValidationContext",
    type_parameters: &["T"], representation: Representation::Opaque,
    independent_clone: false, collection_key: false, thread_transfer: false,
    fields: &[], capabilities: &[],
};

static SCHEMA: Declaration = Declaration {
    id: NativeTypeId::Schema, module: "std:schema", name: "Schema",
    type_parameters: &["T"], representation: Representation::Opaque,
    independent_clone: false, collection_key: false, thread_transfer: false,
    fields: &[], capabilities: &[],
};

static BYTES: Declaration = Declaration {
    id: NativeTypeId::Bytes,
    module: "std:bytes",
    name: "Bytes",
    type_parameters: &[],
    representation: Representation::Opaque,
    independent_clone: true,
    collection_key: false,
    thread_transfer: true,
    fields: &[],
    capabilities: &[],
};
static FILE: Declaration = Declaration {
    id: NativeTypeId::File, module: "std:fs", name: "File",
    type_parameters: &[],
    representation: Representation::Opaque, independent_clone: false,
    collection_key: false, thread_transfer: false, fields: &[], capabilities: &[],
};
static BUFFERED_READER: Declaration = Declaration { id: NativeTypeId::BufferedReader, name: "BufferedReader", ..FILE };
static BUFFERED_WRITER: Declaration = Declaration { id: NativeTypeId::BufferedWriter, name: "BufferedWriter", ..FILE };
static FS_ERROR: Declaration = Declaration {
    id: NativeTypeId::FsError, module: "std:fs", name: "FsError",
    type_parameters: &[],
    representation: Representation::Record, independent_clone: true,
    collection_key: false, thread_transfer: true,
    capabilities: &[Capability::Inspection, Capability::StringConversion, Capability::PropertyReflection, Capability::ObjectSpread, Capability::Equality],
    fields: &[
        Field { name: "code", signature: Signature::String, optional: false },
        Field { name: "operation", signature: Signature::String, optional: false },
        Field { name: "message", signature: Signature::String, optional: false },
        Field { name: "osCode", signature: Signature::Int, optional: true },
    ],
};
static PROCESS_CHILD:Declaration=Declaration{id:NativeTypeId::ProcessChild,module:"std:process",name:"Child",thread_transfer:true,..FILE};
static PROCESS_ERROR:Declaration=Declaration{id:NativeTypeId::ProcessError,module:"std:process",name:"ProcessError",..FS_ERROR};
static PROCESS_STATUS:Declaration=Declaration{id:NativeTypeId::ProcessExitStatus,module:"std:process",name:"ExitStatus",fields:&[Field{name:"success",signature:Signature::Boolean,optional:false},Field{name:"code",signature:Signature::Int,optional:true},Field{name:"signal",signature:Signature::Int,optional:true}],..SPAWN_ERROR};
static PROCESS_OUTPUT:Declaration=Declaration{id:NativeTypeId::ProcessOutput,module:"std:process",name:"Output",fields:&[Field{name:"status",signature:Signature::Named(NativeTypeId::ProcessExitStatus),optional:false},Field{name:"stdout",signature:Signature::Named(NativeTypeId::Bytes),optional:false},Field{name:"stderr",signature:Signature::Named(NativeTypeId::Bytes),optional:false}],..SPAWN_ERROR};
static PROCESS_OPTIONS:Declaration=Declaration{id:NativeTypeId::ProcessCommandOptions,module:"std:process",name:"CommandOptions",fields:&[
 Field{name:"cwd",signature:Signature::String,optional:true},Field{name:"env",signature:Signature::Array(&Signature::Named(NativeTypeId::EnvVar)),optional:true},Field{name:"clearEnv",signature:Signature::Boolean,optional:true},
 Field{name:"stdin",signature:Signature::Union(&[Signature::StringLiteral("inherit"),Signature::StringLiteral("null"),Signature::StringLiteral("pipe")]),optional:true},Field{name:"stdout",signature:Signature::Union(&[Signature::StringLiteral("inherit"),Signature::StringLiteral("null"),Signature::StringLiteral("pipe")]),optional:true},Field{name:"stderr",signature:Signature::Union(&[Signature::StringLiteral("inherit"),Signature::StringLiteral("null"),Signature::StringLiteral("pipe")]),optional:true},
 Field{name:"input",signature:Signature::Union(&[Signature::Named(NativeTypeId::Bytes),Signature::Named(NativeTypeId::IoReader)]),optional:true},Field{name:"deadlineMs",signature:Signature::Int,optional:true},Field{name:"maxOutputBytes",signature:Signature::Int,optional:true},
],..SPAWN_ERROR};
static IO_READER: Declaration = Declaration {id:NativeTypeId::IoReader,module:"std:io",name:"Reader",thread_transfer:true,..FILE};
static IO_WRITER: Declaration = Declaration {id:NativeTypeId::IoWriter,module:"std:io",name:"Writer",thread_transfer:true,..FILE};
static IO_LINE_READER: Declaration = Declaration {id:NativeTypeId::IoLineReader,module:"std:io",name:"LineReader",thread_transfer:true,..FILE};
static IO_STDIN: Declaration = Declaration {id:NativeTypeId::IoStdin,module:"std:io",name:"Stdin",..FILE};
static IO_ERROR: Declaration = Declaration {id:NativeTypeId::IoError,module:"std:io",name:"IoError",..FS_ERROR};
static FS_OPEN_OPTIONS: Declaration = Declaration { id: NativeTypeId::FsOpenOptions, name: "OpenOptions", fields: &[
    Field { name: "read", signature: Signature::Boolean, optional: true },
    Field { name: "write", signature: Signature::Boolean, optional: true },
    Field { name: "append", signature: Signature::Boolean, optional: true },
    Field { name: "create", signature: Signature::Boolean, optional: true },
    Field { name: "createNew", signature: Signature::Boolean, optional: true },
    Field { name: "truncate", signature: Signature::Boolean, optional: true },
], ..FS_ERROR };
static FS_FILE_TIME: Declaration = Declaration { id: NativeTypeId::FsFileTime, name: "FileTime", fields: &[
    Field { name: "unixSeconds", signature: Signature::Int, optional: false },
    Field { name: "nanoseconds", signature: Signature::Int, optional: false },
], ..FS_ERROR };
static FS_METADATA: Declaration = Declaration { id: NativeTypeId::FsMetadata, name: "Metadata", fields: &[
    Field { name: "isFile", signature: Signature::Boolean, optional: false },
    Field { name: "isDirectory", signature: Signature::Boolean, optional: false },
    Field { name: "isSymlink", signature: Signature::Boolean, optional: false },
    Field { name: "size", signature: Signature::Int, optional: false },
    Field { name: "readOnly", signature: Signature::Boolean, optional: false },
    Field { name: "modified", signature: Signature::Named(NativeTypeId::FsFileTime), optional: true },
    Field { name: "accessed", signature: Signature::Named(NativeTypeId::FsFileTime), optional: true },
    Field { name: "created", signature: Signature::Named(NativeTypeId::FsFileTime), optional: true },
], ..FS_ERROR };
static FS_DIR_ENTRY: Declaration = Declaration { id: NativeTypeId::FsDirEntry, name: "DirEntry", fields: &[
    Field { name: "name", signature: Signature::String, optional: false },
    Field { name: "path", signature: Signature::String, optional: false },
    Field { name: "isFile", signature: Signature::Boolean, optional: false },
    Field { name: "isDirectory", signature: Signature::Boolean, optional: false },
    Field { name: "isSymlink", signature: Signature::Boolean, optional: false },
], ..FS_ERROR };
static ENV_ERROR: Declaration = Declaration {
    id: NativeTypeId::EnvError, module: "std:env", name: "EnvError",
    fields: &[
        Field { name: "code", signature: Signature::String, optional: false },
        Field { name: "message", signature: Signature::String, optional: false },
    ], ..FS_ERROR
};
static ENV_VAR:Declaration=Declaration{id:NativeTypeId::EnvVar,module:"std:env",name:"EnvVar",fields:&[
    Field{name:"name",signature:Signature::String,optional:false},Field{name:"value",signature:Signature::String,optional:false},
],..ENV_ERROR};
static PATH_PARTS:Declaration=Declaration{id:NativeTypeId::PathParts,module:"std:path",name:"PathParts",fields:&[
    Field{name:"root",signature:Signature::String,optional:false},Field{name:"dir",signature:Signature::String,optional:false},Field{name:"base",signature:Signature::String,optional:false},Field{name:"ext",signature:Signature::String,optional:false},Field{name:"name",signature:Signature::String,optional:false},
],..ENV_ERROR};
static PATH_FORMAT_OPTIONS:Declaration=Declaration{id:NativeTypeId::PathFormatOptions,module:"std:path",name:"PathFormatOptions",fields:&[
    Field{name:"root",signature:Signature::String,optional:true},Field{name:"dir",signature:Signature::String,optional:true},Field{name:"base",signature:Signature::String,optional:true},Field{name:"ext",signature:Signature::String,optional:true},Field{name:"name",signature:Signature::String,optional:true},
],..ENV_ERROR};
static PATH_ERROR:Declaration=Declaration{id:NativeTypeId::PathError,module:"std:path",name:"PathError",..ENV_ERROR};
static OS_ERROR:Declaration=Declaration{id:NativeTypeId::OsError,module:"std:os",name:"OsError",..ENV_ERROR};
static TIMEOFDAY: Declaration = Declaration {id:NativeTypeId::TimeOfDay,module:"std:date",name:"Time",..BYTES};
static UTCINSTANT: Declaration = Declaration {id:NativeTypeId::UtcInstant,module:"std:date",name:"Instant",..BYTES};
static TIMEDURATION: Declaration = Declaration {id:NativeTypeId::TimeDuration,module:"std:date",name:"Duration",..BYTES};
static TIMEZONE: Declaration = Declaration {id:NativeTypeId::TimeZone,module:"std:date",name:"TimeZone",..BYTES};
static CALENDAR_DATE: Declaration = Declaration {id:NativeTypeId::CalendarDate,module:"std:date",name:"Date",..BYTES};
static DATE_ERROR: Declaration = Declaration {id:NativeTypeId::DateError,module:"std:date",name:"DateError",..ENV_ERROR};
static UUID: Declaration = Declaration {id:NativeTypeId::Uuid,module:"std:uuid",name:"Uuid",..BYTES};
static SIGNAL_LISTENER: Declaration = Declaration {id:NativeTypeId::SignalListener,module:"std:signals",name:"SignalListener",type_parameters:&[],thread_transfer:true,..THREAD};
static SIGNAL_ERROR: Declaration = Declaration {id:NativeTypeId::SignalError,module:"std:signals",name:"SignalError",..ENV_ERROR};
static CRYPTO_ERROR: Declaration = Declaration {id:NativeTypeId::CryptoError,module:"std:crypto",name:"CryptoError",..ENV_ERROR};
static UUID_ERROR: Declaration = Declaration {id:NativeTypeId::UuidError,module:"std:uuid",name:"UuidError",..ENV_ERROR};
static DECIMAL: Declaration = Declaration {id:NativeTypeId::Decimal,module:"std:decimal",name:"Decimal",..BYTES};
static DECIMAL_ERROR: Declaration = Declaration {id:NativeTypeId::DecimalError,module:"std:decimal",name:"DecimalError",..ENV_ERROR};
static NUMERIC_ERROR: Declaration = Declaration {id:NativeTypeId::NumericError,module:"std:numeric",name:"NumericError",..ENV_ERROR};
static JSON_VALUE: Declaration = Declaration {id:NativeTypeId::JsonValue,module:"std:json",name:"JsonValue",..BYTES};
static STRING_VIEW: Declaration = Declaration {id:NativeTypeId::StringView,module:"std:json",name:"StringView",capabilities:&[Capability::Inspection,Capability::StringConversion],..BYTE_VIEW};
static JSON_RAW: Declaration = Declaration {id:NativeTypeId::JsonRaw,module:"std:json",name:"JsonRaw",..BYTE_VIEW};
static JSON_VIEW: Declaration = Declaration {id:NativeTypeId::JsonView,module:"std:json",name:"JsonView",..BYTE_VIEW};
static JSON_ERROR: Declaration = Declaration {id:NativeTypeId::JsonError,module:"std:json",name:"JsonError",fields:&[
    Field{name:"code",signature:Signature::String,optional:false},
    Field{name:"message",signature:Signature::String,optional:false},
    Field{name:"offset",signature:Signature::Int,optional:false},
],..FS_ERROR};
static JSONL_WRITER_VIEW: Declaration = Declaration {id:NativeTypeId::JsonlWriterView,module:"std:json",name:"JsonlWriterView",..BYTE_VIEW_MUT};
static JSONL_WRITER: Declaration = Declaration {id:NativeTypeId::JsonlWriter,module:"std:json",name:"JsonlWriter",..FILE};
static JSONL_READER_VIEW: Declaration = Declaration {id:NativeTypeId::JsonlReaderView,module:"std:json",name:"JsonlReaderView",..BYTE_VIEW_MUT};
static JSONL_READER: Declaration = Declaration {id:NativeTypeId::JsonlReader,module:"std:json",name:"JsonlReader",..FILE};
static JSONL_ERROR: Declaration = Declaration {id:NativeTypeId::JsonlError,module:"std:json",name:"JsonlError",fields:&[
    Field{name:"code",signature:Signature::String,optional:false},
    Field{name:"message",signature:Signature::String,optional:false},
    Field{name:"operation",signature:Signature::String,optional:false},
    Field{name:"line",signature:Signature::Int,optional:false},
    Field{name:"offset",signature:Signature::Int,optional:false},
    Field{name:"osCode",signature:Signature::Int,optional:true},
],..FS_ERROR};
static JSON_BUILD_LIMITS: Declaration = Declaration {id:NativeTypeId::JsonBuildLimits,module:"std:json",name:"JsonBuildLimits",fields:&[
 Field{name:"maxDepth",signature:Signature::Int,optional:false},
 Field{name:"maxEntries",signature:Signature::Int,optional:false},
 Field{name:"maxNodes",signature:Signature::Int,optional:false},
],..FS_ERROR};
static JSON_ENTRY: Declaration = Declaration {id:NativeTypeId::JsonEntry,module:"std:json",name:"JsonEntry",fields:&[
 Field{name:"key",signature:Signature::String,optional:false},
 Field{name:"value",signature:Signature::Named(NativeTypeId::JsonValue),optional:false},
],..FS_ERROR};
static JSON_LIMITS: Declaration = Declaration {id:NativeTypeId::JsonLimits,module:"std:json",name:"JsonLimits",fields:&[
    Field{name:"maxBytes",signature:Signature::Int,optional:false},
    Field{name:"maxDepth",signature:Signature::Int,optional:false},
    Field{name:"maxEntries",signature:Signature::Int,optional:false},
    Field{name:"maxNodes",signature:Signature::Int,optional:false},
],..FS_ERROR};
static BYTE_ERROR: Declaration = Declaration {
    id: NativeTypeId::ByteError,
    module: "std:bytes",
    name: "ByteError",
    type_parameters: &[],
    representation: Representation::Record,
    independent_clone: true,
    collection_key: false,
    thread_transfer: true,
    capabilities: &[Capability::Inspection, Capability::StringConversion, Capability::PropertyReflection, Capability::ObjectSpread, Capability::Equality],
    fields: &[
        Field { name: "code", signature: Signature::String, optional: false },
        Field { name: "message", signature: Signature::String, optional: false },
        Field { name: "offset", signature: Signature::Int, optional: true },
    ],
};

static BYTE_VIEW: Declaration = Declaration {
    id: NativeTypeId::ByteView,
    module: "std:bytes",
    name: "ByteView",
    type_parameters: &[],
    representation: Representation::Borrowed(crate::native_return_borrows::BorrowAccess::ReadOnly),
    independent_clone: false,
    collection_key: false,
    thread_transfer: false,
    fields: &[],
    capabilities: &[],
};
static BYTE_VIEW_MUT: Declaration = Declaration {
    id: NativeTypeId::ByteViewMut,
    module: "std:bytes",
    name: "ByteViewMut",
    type_parameters: &[],
    representation: Representation::Borrowed(crate::native_return_borrows::BorrowAccess::Exclusive),
    independent_clone: false,
    collection_key: false,
    thread_transfer: false,
    fields: &[],
    capabilities: &[],
};

// Opaque collection identity; operations validate the specialized element type.
static DEQUE: Declaration = Declaration {
    id: NativeTypeId::Deque, module: "std:collections", name: "Deque",
    type_parameters: &["T"], representation: Representation::Opaque,
    // Cloneability also requires every specialized element to support cloning.
    independent_clone: true, collection_key: false, thread_transfer: false,
    fields: &[], capabilities: &[],
};

static ORDERED_MAP: Declaration = Declaration {
    id: NativeTypeId::OrderedMap, module: "std:collections", name: "OrderedMap",
    type_parameters: &["K", "V"], representation: Representation::Opaque,
    // Payload cloneability is checked transitively through the map definition.
    independent_clone: true, collection_key: false, thread_transfer: false,
    fields: &[], capabilities: &[],
};

static ORDERED_SET: Declaration = Declaration {
    id: NativeTypeId::OrderedSet, module: "std:collections", name: "OrderedSet",
    type_parameters: &["T"], representation: Representation::Opaque,
    // Scalar keys support independent cloning; thread transfer remains pending.
    independent_clone: true, collection_key: false, thread_transfer: false,
    fields: &[], capabilities: &[],
};

static HEAP: Declaration = Declaration {
    id: NativeTypeId::Heap, module: "std:collections", name: "Heap",
    type_parameters: &["T"], representation: Representation::Opaque,
    // Scalar keys support independent cloning; thread transfer remains pending.
    independent_clone: true, collection_key: false, thread_transfer: false,
    fields: &[], capabilities: &[],
};

static OWNED_ITERATOR:Declaration=Declaration{id:NativeTypeId::OwnedIterator,module:"std:iterator",name:"Iterator",type_parameters:&["T"],..THREAD};
static ITERATOR_CURSOR:Declaration=Declaration{id:NativeTypeId::IteratorCursor,module:"std:iterator-runtime",name:"ArrayCursor",type_parameters:&["T"],fields:&[Field{name:"_items",signature:Signature::Array(&Signature::Parameter(0)),optional:false}],..THREAD};

/// Exhaustive identity lookup prevents missing declarations as IDs are added.
pub fn declaration(id: NativeTypeId) -> &'static Declaration {
    match id {
        NativeTypeId::Locale=>&LOCALE_LOCALE,NativeTypeId::Collator=>&LOCALE_COLLATOR,NativeTypeId::Segmenter=>&LOCALE_SEGMENTER,NativeTypeId::NumberFormat=>&LOCALE_NUMBERFORMAT,NativeTypeId::SegmentRange=>&LOCALE_SEGMENTRANGE,NativeTypeId::LocaleError=>&LOCALE_LOCALEERROR,NativeTypeId::CollatorOptions=>&LOCALE_COLLATOROPTIONS,NativeTypeId::NumberFormatOptions=>&LOCALE_NUMBERFORMATOPTIONS,
        NativeTypeId::Regex=>&REGEX,NativeTypeId::RegexMatch=>&REGEX_MATCH,NativeTypeId::RegexCaptures=>&REGEX_CAPTURES,NativeTypeId::RegexError=>&REGEX_ERROR,NativeTypeId::RegexOptions=>&REGEX_OPTIONS,
        NativeTypeId::TcpListener=>&TCP_LISTENER,NativeTypeId::TcpStream=>&TCP_STREAM,NativeTypeId::UdpSocket=>&UDP_SOCKET,NativeTypeId::Datagram=>&DATAGRAM,NativeTypeId::IpAddress=>&IP_ADDRESS,NativeTypeId::SocketAddress=>&SOCKET_ADDRESS,NativeTypeId::NetError=>&NET_ERROR,
        NativeTypeId::Url=>&URL,NativeTypeId::SearchParams=>&SEARCH_PARAMS,NativeTypeId::UrlError=>&URL_ERROR,NativeTypeId::QueryEntry=>&QUERY_ENTRY,
        NativeTypeId::LruCache => &LRU_CACHE, NativeTypeId::SharedLruCache => &SHARED_LRU_CACHE, NativeTypeId::CacheEntry => &CACHE_ENTRY, NativeTypeId::CacheInsertError => &CACHE_INSERT_ERROR,
        NativeTypeId::HttpTlsOptions => &HTTPTLSOPTIONS,
        NativeTypeId::HttpClientOptions => &HTTPCLIENTOPTIONS,
        NativeTypeId::HttpFormField => &HTTPFORMFIELD,
        NativeTypeId::HttpMultipartPart => &HTTPMULTIPARTPART,
        NativeTypeId::HttpCookieOptions => &HTTPCOOKIEOPTIONS,
        NativeTypeId::HttpServerOptions => &HTTPSERVEROPTIONS,
        NativeTypeId::SqlPool => &SQLPOOL,
NativeTypeId::SqlTransaction => &SQLTRANSACTION,
NativeTypeId::SqlCursor => &SQLCURSOR,
NativeTypeId::SqlRow => &SQLROW,
NativeTypeId::SqlError => &SQLERROR,
NativeTypeId::SqliteConnection => &SQLITECONNECTION,
NativeTypeId::SqliteStatement => &SQLITESTATEMENT,
NativeTypeId::SqliteTransaction => &SQLITETRANSACTION,
NativeTypeId::SqliteRow => &SQLITEROW,
NativeTypeId::SqliteError => &SQLITEERROR,
NativeTypeId::HttpRouter => &HTTPROUTER,NativeTypeId::HttpContext => &HTTPCONTEXT,NativeTypeId::HttpContextNext => &HTTPCONTEXTNEXT,
NativeTypeId::HttpApp => &HTTPAPP,
        NativeTypeId::HttpRequest => &HTTPREQUEST,
        NativeTypeId::HttpResponse => &HTTPRESPONSE,
        NativeTypeId::HttpBody => &HTTPBODY,
        NativeTypeId::HttpClient => &HTTPCLIENT,
        NativeTypeId::HttpServer => &HTTPSERVER,
        NativeTypeId::HttpNext => &HTTPNEXT,
        NativeTypeId::HttpError => &HTTPERROR,
        NativeTypeId::Job => &JOB,
        NativeTypeId::AsyncError => &ASYNC_ERROR,
        NativeTypeId::AsyncChannel => &ASYNC_CHANNEL,
        NativeTypeId::AsyncSender => &ASYNC_SENDER,
        NativeTypeId::Interval => &INTERVAL,
        NativeTypeId::CancellationToken => &CANCELLATION_TOKEN,
        NativeTypeId::PoolSubmission => &POOL_SUBMISSION,
        NativeTypeId::Thread => &THREAD,
        NativeTypeId::ScopedThread => &SCOPED_THREAD,
        NativeTypeId::CompressionReader=>&COMPRESSION_READER,NativeTypeId::CompressionWriter=>&COMPRESSION_WRITER,NativeTypeId::CompressionOptions=>&COMPRESSION_OPTIONS,NativeTypeId::DecompressionOptions=>&DECOMPRESSION_OPTIONS,NativeTypeId::CompressionError=>&COMPRESSION_ERROR,
        NativeTypeId::ThreadError => &THREAD_ERROR,
        NativeTypeId::SpawnError => &SPAWN_ERROR,
        NativeTypeId::ThreadPoolStats => &POOL_STATS,
        NativeTypeId::ThreadPoolOptions => &POOL_OPTIONS,
        NativeTypeId::ThreadPool => &POOL,
        NativeTypeId::TaskContext => &TASK_CONTEXT,
        NativeTypeId::Channel => &CHANNEL,
        NativeTypeId::Sender => &SENDER,
        NativeTypeId::ChannelError => &CHANNEL_ERROR,
        NativeTypeId::SendError => &SEND_ERROR,
        NativeTypeId::TrackedResource => &TRACKED_RESOURCE,
        NativeTypeId::TestRegistry => &TEST_REGISTRY,
        NativeTypeId::Schema => &SCHEMA,
        NativeTypeId::ValidationContext => &VALIDATION_CONTEXT,
        NativeTypeId::ValidationError => &VALIDATION_ERROR,
        NativeTypeId::SchemaExportError => &SCHEMA_EXPORT_ERROR,
        NativeTypeId::SchemaConstructionError => &SCHEMA_CONSTRUCTION_ERROR,
        NativeTypeId::SchemaIssue => &SCHEMA_ISSUE,
        NativeTypeId::SchemaPath => &SCHEMA_PATH,
        NativeTypeId::Bytes => &BYTES,
        NativeTypeId::ByteError => &BYTE_ERROR,
        NativeTypeId::ByteView => &BYTE_VIEW,
        NativeTypeId::ByteViewMut => &BYTE_VIEW_MUT,
        NativeTypeId::File => &FILE,
        NativeTypeId::BufferedReader => &BUFFERED_READER,
        NativeTypeId::BufferedWriter => &BUFFERED_WRITER,
        NativeTypeId::FsOpenOptions => &FS_OPEN_OPTIONS,
        NativeTypeId::FsFileTime => &FS_FILE_TIME,
        NativeTypeId::FsMetadata => &FS_METADATA,
        NativeTypeId::FsDirEntry => &FS_DIR_ENTRY,
        NativeTypeId::FsError => &FS_ERROR,
        NativeTypeId::ProcessChild=>&PROCESS_CHILD,NativeTypeId::ProcessCommandOptions=>&PROCESS_OPTIONS,NativeTypeId::ProcessExitStatus=>&PROCESS_STATUS,NativeTypeId::ProcessOutput=>&PROCESS_OUTPUT,NativeTypeId::ProcessError=>&PROCESS_ERROR,
        NativeTypeId::IoReader=>&IO_READER,NativeTypeId::IoWriter=>&IO_WRITER,NativeTypeId::IoLineReader=>&IO_LINE_READER,NativeTypeId::IoStdin=>&IO_STDIN,NativeTypeId::IoError=>&IO_ERROR,
        NativeTypeId::EnvError => &ENV_ERROR,
        NativeTypeId::EnvVar => &ENV_VAR,
        NativeTypeId::PathParts => &PATH_PARTS,
        NativeTypeId::PathFormatOptions => &PATH_FORMAT_OPTIONS,
        NativeTypeId::PathError => &PATH_ERROR,
        NativeTypeId::OsError => &OS_ERROR,
        NativeTypeId::UiSignal => &UI_SIGNAL,
        NativeTypeId::UiComputed => &UI_COMPUTED,
        NativeTypeId::UiNode => &UI_NODE,
        NativeTypeId::UiTextBinding => &UI_TEXT_BINDING,
        NativeTypeId::UiError => &UI_ERROR,
        NativeTypeId::NumericError => &NUMERIC_ERROR,
        NativeTypeId::TimeOfDay => &TIMEOFDAY,
        NativeTypeId::UtcInstant => &UTCINSTANT,
        NativeTypeId::TimeDuration => &TIMEDURATION,
        NativeTypeId::TimeZone => &TIMEZONE,
        NativeTypeId::CalendarDate => &CALENDAR_DATE,
        NativeTypeId::DateError => &DATE_ERROR,
        NativeTypeId::Uuid => &UUID,
        NativeTypeId::SignalListener => &SIGNAL_LISTENER,
        NativeTypeId::SignalError => &SIGNAL_ERROR,
        NativeTypeId::CryptoError => &CRYPTO_ERROR,
        NativeTypeId::UuidError => &UUID_ERROR,
        NativeTypeId::Decimal => &DECIMAL,
        NativeTypeId::DecimalError => &DECIMAL_ERROR,
        NativeTypeId::JsonValue => &JSON_VALUE,
        NativeTypeId::JsonView => &JSON_VIEW,
        NativeTypeId::JsonRaw => &JSON_RAW,
        NativeTypeId::StringView => &STRING_VIEW,
        NativeTypeId::JsonError => &JSON_ERROR,
        NativeTypeId::JsonLimits => &JSON_LIMITS,
        NativeTypeId::JsonBuildLimits => &JSON_BUILD_LIMITS,
        NativeTypeId::JsonEntry => &JSON_ENTRY,
        NativeTypeId::JsonlReader => &JSONL_READER,
        NativeTypeId::JsonlReaderView => &JSONL_READER_VIEW,
        NativeTypeId::JsonlWriter => &JSONL_WRITER,
        NativeTypeId::JsonlWriterView => &JSONL_WRITER_VIEW,
        NativeTypeId::JsonlError => &JSONL_ERROR,
        NativeTypeId::Deque => &DEQUE,
        NativeTypeId::OrderedMap => &ORDERED_MAP,
        NativeTypeId::OrderedSet => &ORDERED_SET,
        NativeTypeId::Heap => &HEAP,
        NativeTypeId::OwnedIterator=>&OWNED_ITERATOR,NativeTypeId::IteratorCursor=>&ITERATOR_CURSOR,
    }
}

/// Exact module/name lookup does not grant aliases a new declaration identity.
/// This registry alone does not publish types through the source module loader.
pub fn lookup(module: &str, name: &str) -> Option<&'static Declaration> {
    if module=="std:futures" {match name {"SendError"=>return Some(&SEND_ERROR),"ChannelError"=>return Some(&CHANNEL_ERROR),_=>{}}}
    [&OWNED_ITERATOR,&ITERATOR_CURSOR,&TCP_LISTENER,&TCP_STREAM,&UDP_SOCKET,&DATAGRAM,&IP_ADDRESS,&SOCKET_ADDRESS,&NET_ERROR,&PROCESS_CHILD,&PROCESS_OPTIONS,&PROCESS_STATUS,&PROCESS_OUTPUT,&PROCESS_ERROR,&IO_READER,&IO_WRITER,&IO_LINE_READER,&IO_STDIN,&IO_ERROR,&LOCALE_LOCALE,&LOCALE_COLLATOR,&LOCALE_SEGMENTER,&LOCALE_NUMBERFORMAT,&LOCALE_SEGMENTRANGE,&LOCALE_LOCALEERROR,&LOCALE_COLLATOROPTIONS,&LOCALE_NUMBERFORMATOPTIONS,&REGEX,&REGEX_MATCH,&REGEX_CAPTURES,&REGEX_ERROR,&REGEX_OPTIONS,&URL,&SEARCH_PARAMS,&URL_ERROR,&QUERY_ENTRY,&UI_SIGNAL,&UI_COMPUTED,&UI_NODE,&UI_TEXT_BINDING,
        &UI_ERROR,
        &COMPRESSION_READER,&COMPRESSION_WRITER,&COMPRESSION_OPTIONS,&DECOMPRESSION_OPTIONS,&COMPRESSION_ERROR,&SIGNAL_LISTENER,&SIGNAL_ERROR,&CRYPTO_ERROR,&LRU_CACHE,&SHARED_LRU_CACHE,&CACHE_ENTRY,&CACHE_INSERT_ERROR,&UUID,&UUID_ERROR,&TIMEOFDAY,&UTCINSTANT,&TIMEDURATION,&TIMEZONE,&CALENDAR_DATE,&DATE_ERROR,&DECIMAL,&DECIMAL_ERROR,&NUMERIC_ERROR,&TRACKED_RESOURCE,&SQLPOOL,&SQLTRANSACTION,&SQLCURSOR,&SQLROW,&SQLERROR,&SQLITECONNECTION,&SQLITESTATEMENT,&SQLITETRANSACTION,&SQLITEROW,&SQLITEERROR,&HTTPTLSOPTIONS,&HTTPFORMFIELD,&HTTPMULTIPARTPART,&HTTPCOOKIEOPTIONS,&HTTPSERVEROPTIONS,&HTTPCLIENTOPTIONS,&HTTPROUTER,&HTTPCONTEXT,&HTTPCONTEXTNEXT,&HTTPAPP,&HTTPREQUEST,&HTTPRESPONSE,&HTTPBODY,&HTTPCLIENT,&HTTPSERVER,&HTTPNEXT,&HTTPERROR,&JOB,&ASYNC_ERROR,&ASYNC_CHANNEL,&ASYNC_SENDER,&INTERVAL,&CANCELLATION_TOKEN,&POOL_SUBMISSION,&TASK_CONTEXT, &CHANNEL, &SENDER, &CHANNEL_ERROR, &SEND_ERROR, &POOL_OPTIONS, &POOL, &POOL_STATS, &THREAD, &SCOPED_THREAD, &THREAD_ERROR, &SPAWN_ERROR, &SCHEMA_EXPORT_ERROR, &VALIDATION_CONTEXT, &SCHEMA_CONSTRUCTION_ERROR, &VALIDATION_ERROR, &SCHEMA_ISSUE, &SCHEMA_PATH, &SCHEMA, &JSON_RAW, &JSONL_WRITER_VIEW, &JSONL_READER_VIEW, &JSONL_WRITER, &JSON_BUILD_LIMITS, &JSON_ENTRY, &STRING_VIEW, &JSONL_READER, &JSONL_ERROR, &JSON_VALUE, &JSON_VIEW, &JSON_ERROR, &JSON_LIMITS, &ENV_ERROR,&ENV_VAR,&PATH_PARTS,&PATH_FORMAT_OPTIONS,&PATH_ERROR,&OS_ERROR, &BYTES, &BYTE_ERROR, &BYTE_VIEW, &BYTE_VIEW_MUT, &FILE, &BUFFERED_READER, &BUFFERED_WRITER, &FS_OPEN_OPTIONS, &FS_FILE_TIME, &FS_METADATA, &FS_DIR_ENTRY, &FS_ERROR, &DEQUE, &HEAP, &ORDERED_SET, &ORDERED_MAP,
    ].into_iter().find(|item| item.module == module && item.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_return_borrows::BorrowAccess;

    #[test]
    fn validation_context_is_opaque_nonclonable_and_not_transferable() {
        let context=declaration(NativeTypeId::ValidationContext);
        assert_eq!(context.type_parameters, &["T"]);
        assert!(!context.source_constructible());
        assert!(!context.independent_clone);
        assert!(!context.collection_key);
        assert!(!context.thread_transfer);
        assert!(context.fields.is_empty());
        assert_eq!(lookup("std:schema","ValidationContext").unwrap().id,context.id);
    }

    #[test]
    fn byte_view_identities_are_nonconstructible_borrowed_storage() {
        for (id, access) in [
            (NativeTypeId::ByteView, BorrowAccess::ReadOnly),
            (NativeTypeId::ByteViewMut, BorrowAccess::Exclusive),
        ] {
            let declaration = declaration(id);
            assert_eq!(declaration.borrow_access(), Some(access));
            assert!(!declaration.source_constructible());
            assert!(!declaration.independent_clone);
            assert!(!declaration.collection_key);
            assert!(!declaration.thread_transfer);
            assert!(declaration.fields.is_empty());
        }
        assert_eq!(declaration(NativeTypeId::Bytes).borrow_access(), None);
        assert_eq!(declaration(NativeTypeId::ByteError).borrow_access(), None);
    }
}
