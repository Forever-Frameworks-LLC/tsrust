use crate::{Mode, native_bindings::{NativeBackend, NativeBinding, NativeParameter, NativeType}, native_signatures::{NativeTypeId, Signature}};

pub const OPEN: NativeBinding = NativeBinding {
    module: "std:fs", name: "open", returned_borrow: None,
    parameters: &[NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false }],
    result: NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::File), &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_open" }, features: &["fs"],
};
pub const READ: NativeBinding = NativeBinding {
    module: "std:fs", name: "read", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "file", ty: NativeType::Structured(&Signature::Named(NativeTypeId::File)), access: Mode::Mutate, optional: false, rest: false },
        NativeParameter { name: "length", ty: NativeType::Int, access: Mode::Copy, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::Bytes), &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_read" }, features: &["fs", "bytes"],
};
pub const CLOSE: NativeBinding = NativeBinding {
    module: "std:fs", name: "close", returned_borrow: None,
    parameters: &[NativeParameter { name: "file", ty: NativeType::Structured(&Signature::Named(NativeTypeId::File)), access: Mode::Consume, optional: false, rest: false }],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_close" }, features: &["fs"],
};

pub const CREATE: NativeBinding = NativeBinding {
    name: "create", backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_create" }, ..OPEN
};
pub const WRITE: NativeBinding = NativeBinding {
    module: "std:fs", name: "write", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "file", ty: NativeType::Structured(&Signature::Named(NativeTypeId::File)), access: Mode::Mutate, optional: false, rest: false },
        NativeParameter { name: "bytes", ty: NativeType::Structured(&Signature::Named(NativeTypeId::Bytes)), access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Int, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_write" }, features: &["fs", "bytes"],
};
pub const WRITE_ALL: NativeBinding = NativeBinding {
    name: "writeAll", result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_write_all" }, ..WRITE
};

pub const READ_FILE: NativeBinding = NativeBinding {
    name: "readFile", parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "maxBytes", ty: NativeType::Int, access: Mode::Copy, optional: true, rest: false },
    ], backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_read_file" }, ..READ
};
pub const WRITE_FILE: NativeBinding = NativeBinding {
    name: "writeFile", parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "bytes", ty: NativeType::Structured(&Signature::Named(NativeTypeId::Bytes)), access: Mode::Read, optional: false, rest: false },
    ], backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_write_file" }, ..WRITE_ALL
};


pub const BUFFER_READER: NativeBinding = NativeBinding {
    name: "bufferReader", parameters: &[
        NativeParameter { name: "file", ty: NativeType::Structured(&Signature::Named(NativeTypeId::File)), access: Mode::Consume, optional: false, rest: false },
        NativeParameter { name: "capacity", ty: NativeType::Int, access: Mode::Copy, optional: true, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::BufferedReader), &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_buffer_reader" }, ..OPEN
};
pub const BUFFER_WRITER: NativeBinding = NativeBinding {
    name: "bufferWriter",
    result: NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::BufferedWriter), &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_buffer_writer" }, ..BUFFER_READER
};
pub const FLUSH: NativeBinding = NativeBinding {
    name: "flush", parameters: &[
        NativeParameter { name: "writer", ty: NativeType::Structured(&Signature::Named(NativeTypeId::BufferedWriter)), access: Mode::Mutate, optional: false, rest: false },
    ], backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_flush" }, ..CLOSE
};

pub const OPEN_FILE: NativeBinding = NativeBinding {
    module: "std:fs", name: "openFile", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "options", ty: NativeType::Structured(&Signature::Named(NativeTypeId::FsOpenOptions)), access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::File), &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_open_file" }, features: &["fs"],
};

pub const STAT: NativeBinding = NativeBinding {
    module: "std:fs", name: "stat", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::FsMetadata), &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_stat" }, features: &["fs"],
};

pub const LSTAT: NativeBinding = NativeBinding {
    module: "std:fs", name: "lstat", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::FsMetadata), &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_lstat" }, features: &["fs"],
};

pub const FILE_STAT: NativeBinding = NativeBinding {
    module: "std:fs", name: "fileStat", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "file", ty: NativeType::Structured(&Signature::Named(NativeTypeId::File)), access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::FsMetadata), &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_file_stat" }, features: &["fs"],
};

pub const READ_DIR: NativeBinding = NativeBinding {
    module: "std:fs", name: "readDir", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "maxEntries", ty: NativeType::Int, access: Mode::Copy, optional: true, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Array(&Signature::Named(NativeTypeId::FsDirEntry)), &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_read_dir" }, features: &["fs"],
};

pub const MKDIR: NativeBinding = NativeBinding {
    module: "std:fs", name: "mkdir", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_mkdir" }, features: &["fs"],
};

pub const MKDIR_ALL: NativeBinding = NativeBinding {
    module: "std:fs", name: "mkdirAll", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_mkdir_all" }, features: &["fs"],
};

pub const REMOVE_FILE: NativeBinding = NativeBinding {
    module: "std:fs", name: "removeFile", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_remove_file" }, features: &["fs"],
};

pub const REMOVE_DIR: NativeBinding = NativeBinding {
    module: "std:fs", name: "removeDir", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_remove_dir" }, features: &["fs"],
};

pub const REMOVE_DIR_ALL: NativeBinding = NativeBinding {
    module: "std:fs", name: "removeDirAll", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_remove_dir_all" }, features: &["fs"],
};

pub const RENAME: NativeBinding = NativeBinding {
    module: "std:fs", name: "rename", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "destination", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_rename" }, features: &["fs"],
};

pub const COPY_FILE: NativeBinding = NativeBinding {
    module: "std:fs", name: "copyFile", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "destination", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Int, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_copy_file" }, features: &["fs"],
};

pub const HARD_LINK: NativeBinding = NativeBinding {
    module: "std:fs", name: "hardLink", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "destination", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_hard_link" }, features: &["fs"],
};

pub const CANONICALIZE: NativeBinding = NativeBinding {
    module: "std:fs", name: "canonicalize", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::String, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_canonicalize" }, features: &["fs"],
};

pub const READ_LINK: NativeBinding = NativeBinding {
    module: "std:fs", name: "readLink", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::String, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_read_link" }, features: &["fs"],
};

pub const APPEND_FILE: NativeBinding = NativeBinding {
    module: "std:fs", name: "appendFile", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "bytes", ty: NativeType::Structured(&Signature::Named(NativeTypeId::Bytes)), access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_append_file" }, features: &["fs", "bytes"],
};

pub const READ_TEXT_FILE: NativeBinding = NativeBinding {
    module: "std:fs", name: "readTextFile", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "maxBytes", ty: NativeType::Int, access: Mode::Copy, optional: true, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::String, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_read_text_file" }, features: &["fs"],
};

pub const WRITE_TEXT_FILE: NativeBinding = NativeBinding {
    module: "std:fs", name: "writeTextFile", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "text", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_write_text_file" }, features: &["fs"],
};

pub const TRUNCATE: NativeBinding = NativeBinding {
    module: "std:fs", name: "truncate", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "file", ty: NativeType::Structured(&Signature::Named(NativeTypeId::File)), access: Mode::Mutate, optional: false, rest: false },
        NativeParameter { name: "length", ty: NativeType::Int, access: Mode::Copy, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_truncate" }, features: &["fs"],
};

pub const SYNC_ALL: NativeBinding = NativeBinding {
    module: "std:fs", name: "syncAll", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "file", ty: NativeType::Structured(&Signature::Named(NativeTypeId::File)), access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_sync_all" }, features: &["fs"],
};

pub const SYNC_DATA: NativeBinding = NativeBinding {
    module: "std:fs", name: "syncData", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "file", ty: NativeType::Structured(&Signature::Named(NativeTypeId::File)), access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_sync_data" }, features: &["fs"],
};

pub const CREATE_TEMP_DIR: NativeBinding = NativeBinding {
    module: "std:fs", name: "createTempDir", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "parent", ty: NativeType::String, access: Mode::Read, optional: true, rest: false },
        NativeParameter { name: "prefix", ty: NativeType::String, access: Mode::Read, optional: true, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::String, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_create_temp_dir" }, features: &["fs", "temporary-files"],
};
