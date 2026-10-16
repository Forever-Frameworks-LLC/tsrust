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
static LRU_CACHE: Declaration = Declaration {id:NativeTypeId::LruCache,module:"std:cache",name:"LruCache",type_parameters:&["K","V"],..THREAD};
static CACHE_ENTRY: Declaration = Declaration {id:NativeTypeId::CacheEntry,module:"std:cache",name:"CacheEntry",type_parameters:&["K","V"],fields:&[
 Field{name:"key",signature:Signature::Parameter(0),optional:false},Field{name:"value",signature:Signature::Parameter(1),optional:false},
],..SPAWN_ERROR};
static CACHE_INSERT_ERROR: Declaration = Declaration {id:NativeTypeId::CacheInsertError,name:"CacheInsertError",fields:&[
 Field{name:"code",signature:Signature::String,optional:false},Field{name:"key",signature:Signature::Parameter(0),optional:false},Field{name:"value",signature:Signature::Parameter(1),optional:false},
],..CACHE_ENTRY};
static SQLPOOL:Declaration=Declaration{id:NativeTypeId::SqlPool,module:"std:sql",name:"Pool",type_parameters:&[],thread_transfer:true,..THREAD};
static SQLTRANSACTION:Declaration=Declaration{representation:Representation::Borrowed(crate::native_return_borrows::BorrowAccess::ReadOnly),id:NativeTypeId::SqlTransaction,module:"std:sql",name:"Transaction",type_parameters:&[],thread_transfer:true,..THREAD};
static SQLROW:Declaration=Declaration{id:NativeTypeId::SqlRow,module:"std:sql",name:"Row",type_parameters:&[],thread_transfer:true,..THREAD};
static SQLERROR:Declaration=Declaration{id:NativeTypeId::SqlError,module:"std:sql",name:"SqlError",type_parameters:&[],thread_transfer:true,..SPAWN_ERROR};
static SQLITECONNECTION:Declaration=Declaration{id:NativeTypeId::SqliteConnection,module:"std:sqlite",name:"Connection",type_parameters:&[],..THREAD};
static SQLITESTATEMENT:Declaration=Declaration{id:NativeTypeId::SqliteStatement,module:"std:sqlite",name:"Statement",type_parameters:&[],representation:Representation::Borrowed(crate::native_return_borrows::BorrowAccess::ReadOnly),..THREAD};
static SQLITETRANSACTION:Declaration=Declaration{id:NativeTypeId::SqliteTransaction,module:"std:sqlite",name:"Transaction",type_parameters:&[],representation:Representation::Borrowed(crate::native_return_borrows::BorrowAccess::Exclusive),..THREAD};
static SQLITEROW:Declaration=Declaration{id:NativeTypeId::SqliteRow,module:"std:sqlite",name:"Row",type_parameters:&[],..THREAD};
static SQLITEERROR:Declaration=Declaration{id:NativeTypeId::SqliteError,module:"std:sqlite",name:"SqliteError",type_parameters:&[],..SPAWN_ERROR};

static HTTPAPP: Declaration = Declaration {id:NativeTypeId::HttpApp,module:"std:http",name:"App",type_parameters:&[],thread_transfer:true,..THREAD};
static HTTPREQUEST: Declaration = Declaration {id:NativeTypeId::HttpRequest,module:"std:http",name:"Request",type_parameters:&[],thread_transfer:true,..THREAD};
static HTTPRESPONSE: Declaration = Declaration {id:NativeTypeId::HttpResponse,module:"std:http",name:"Response",type_parameters:&[],thread_transfer:true,..THREAD};
static HTTPBODY: Declaration = Declaration {id:NativeTypeId::HttpBody,module:"std:http",name:"Body",type_parameters:&[],thread_transfer:true,..THREAD};
static HTTPCLIENT: Declaration = Declaration {id:NativeTypeId::HttpClient,module:"std:http",name:"Client",type_parameters:&[],thread_transfer:true,..THREAD};
static HTTPSERVER: Declaration = Declaration {id:NativeTypeId::HttpServer,module:"std:http",name:"Server",type_parameters:&[],thread_transfer:true,..THREAD};
static HTTPNEXT: Declaration = Declaration {id:NativeTypeId::HttpNext,module:"std:http",name:"Next",type_parameters:&[],thread_transfer:true,..THREAD};
static HTTPERROR:Declaration=Declaration{id:NativeTypeId::HttpError,module:"std:http",name:"HttpError",..SPAWN_ERROR};

static HTTPTLSOPTIONS: Declaration = Declaration {id:NativeTypeId::HttpTlsOptions,module:"std:http",name:"TlsOptions",fields:&[
 Field{name:"cert",signature:Signature::String,optional:false},Field{name:"key",signature:Signature::String,optional:false},
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
static POOL: Declaration = Declaration { id: NativeTypeId::ThreadPool, name: "ThreadPool", type_parameters: &[], ..THREAD };
static TASK_CONTEXT: Declaration = Declaration { id: NativeTypeId::TaskContext, name: "TaskContext", type_parameters: &[], ..THREAD };
static CHANNEL: Declaration = Declaration { id: NativeTypeId::Channel, name: "Channel", type_parameters: &["T"], thread_transfer: true, ..THREAD };
static SENDER: Declaration = Declaration { id: NativeTypeId::Sender, name: "Sender", ..CHANNEL };
static CHANNEL_ERROR: Declaration = Declaration { id: NativeTypeId::ChannelError, name: "ChannelError", fields: &[Field { name: "code", signature: Signature::String, optional: false }], ..SPAWN_ERROR };
static SEND_ERROR: Declaration = Declaration { id: NativeTypeId::SendError, name: "SendError", type_parameters: &["T"], fields: &[
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
static ENV_ERROR: Declaration = Declaration {
    id: NativeTypeId::EnvError, module: "std:env", name: "EnvError",
    fields: &[
        Field { name: "code", signature: Signature::String, optional: false },
        Field { name: "message", signature: Signature::String, optional: false },
    ], ..FS_ERROR
};
static TIMEOFDAY: Declaration = Declaration {id:NativeTypeId::TimeOfDay,module:"std:date",name:"Time",..BYTES};
static UTCINSTANT: Declaration = Declaration {id:NativeTypeId::UtcInstant,module:"std:date",name:"Instant",..BYTES};
static TIMEDURATION: Declaration = Declaration {id:NativeTypeId::TimeDuration,module:"std:date",name:"Duration",..BYTES};
static TIMEZONE: Declaration = Declaration {id:NativeTypeId::TimeZone,module:"std:date",name:"TimeZone",..BYTES};
static CALENDAR_DATE: Declaration = Declaration {id:NativeTypeId::CalendarDate,module:"std:date",name:"Date",..BYTES};
static DATE_ERROR: Declaration = Declaration {id:NativeTypeId::DateError,module:"std:date",name:"DateError",..ENV_ERROR};
static UUID: Declaration = Declaration {id:NativeTypeId::Uuid,module:"std:uuid",name:"Uuid",..BYTES};
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

/// Exhaustive identity lookup prevents missing declarations as IDs are added.
pub fn declaration(id: NativeTypeId) -> &'static Declaration {
    match id {
        NativeTypeId::LruCache => &LRU_CACHE, NativeTypeId::CacheEntry => &CACHE_ENTRY, NativeTypeId::CacheInsertError => &CACHE_INSERT_ERROR,
        NativeTypeId::HttpTlsOptions => &HTTPTLSOPTIONS,
        NativeTypeId::HttpClientOptions => &HTTPCLIENTOPTIONS,
        NativeTypeId::SqlPool => &SQLPOOL,
NativeTypeId::SqlTransaction => &SQLTRANSACTION,
NativeTypeId::SqlRow => &SQLROW,
NativeTypeId::SqlError => &SQLERROR,
NativeTypeId::SqliteConnection => &SQLITECONNECTION,
NativeTypeId::SqliteStatement => &SQLITESTATEMENT,
NativeTypeId::SqliteTransaction => &SQLITETRANSACTION,
NativeTypeId::SqliteRow => &SQLITEROW,
NativeTypeId::SqliteError => &SQLITEERROR,
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
        NativeTypeId::Thread => &THREAD,
        NativeTypeId::ScopedThread => &SCOPED_THREAD,
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
        NativeTypeId::FsError => &FS_ERROR,
        NativeTypeId::EnvError => &ENV_ERROR,
        NativeTypeId::NumericError => &NUMERIC_ERROR,
        NativeTypeId::TimeOfDay => &TIMEOFDAY,
        NativeTypeId::UtcInstant => &UTCINSTANT,
        NativeTypeId::TimeDuration => &TIMEDURATION,
        NativeTypeId::TimeZone => &TIMEZONE,
        NativeTypeId::CalendarDate => &CALENDAR_DATE,
        NativeTypeId::DateError => &DATE_ERROR,
        NativeTypeId::Uuid => &UUID,
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
    }
}

/// Exact module/name lookup does not grant aliases a new declaration identity.
/// This registry alone does not publish types through the source module loader.
pub fn lookup(module: &str, name: &str) -> Option<&'static Declaration> {
    [&LRU_CACHE,&CACHE_ENTRY,&CACHE_INSERT_ERROR,&UUID,&UUID_ERROR,&TIMEOFDAY,&UTCINSTANT,&TIMEDURATION,&TIMEZONE,&CALENDAR_DATE,&DATE_ERROR,&DECIMAL,&DECIMAL_ERROR,&NUMERIC_ERROR,&TRACKED_RESOURCE,&SQLPOOL,&SQLTRANSACTION,&SQLROW,&SQLERROR,&SQLITECONNECTION,&SQLITESTATEMENT,&SQLITETRANSACTION,&SQLITEROW,&SQLITEERROR,&HTTPTLSOPTIONS,&HTTPCLIENTOPTIONS,&HTTPAPP,&HTTPREQUEST,&HTTPRESPONSE,&HTTPBODY,&HTTPCLIENT,&HTTPSERVER,&HTTPNEXT,&HTTPERROR,&JOB,&ASYNC_ERROR,&ASYNC_CHANNEL,&ASYNC_SENDER,&TASK_CONTEXT, &CHANNEL, &SENDER, &CHANNEL_ERROR, &SEND_ERROR, &POOL_OPTIONS, &POOL, &POOL_STATS, &THREAD, &SCOPED_THREAD, &THREAD_ERROR, &SPAWN_ERROR, &SCHEMA_EXPORT_ERROR, &VALIDATION_CONTEXT, &SCHEMA_CONSTRUCTION_ERROR, &VALIDATION_ERROR, &SCHEMA_ISSUE, &SCHEMA_PATH, &SCHEMA, &JSON_RAW, &JSONL_WRITER_VIEW, &JSONL_READER_VIEW, &JSONL_WRITER, &JSON_BUILD_LIMITS, &JSON_ENTRY, &STRING_VIEW, &JSONL_READER, &JSONL_ERROR, &JSON_VALUE, &JSON_VIEW, &JSON_ERROR, &JSON_LIMITS, &ENV_ERROR, &BYTES, &BYTE_ERROR, &BYTE_VIEW, &BYTE_VIEW_MUT, &FILE, &BUFFERED_READER, &BUFFERED_WRITER, &FS_ERROR, &DEQUE, &HEAP, &ORDERED_SET, &ORDERED_MAP].into_iter().find(|item| item.module == module && item.name == name)
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
