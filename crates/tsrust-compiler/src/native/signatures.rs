//! Identity-bearing native signature descriptions shared by checking and editor help.

/// Stable declaration identity; visible aliases do not change it. These IDs do
/// not publish source exports: module loading must resolve registered declarations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NativeTypeId {
    Job,
    AsyncError,
    AsyncChannel,
    AsyncSender,
    Thread,
    ScopedThread,
    ThreadError,
    SpawnError,
    ThreadPoolStats,
    ThreadPoolOptions,
    ThreadPool,
    TaskContext,
    Channel,
    Sender,
    ChannelError,
    SendError,
    TestRegistry,
    Schema,
    ValidationContext,
    ValidationError,
    SchemaConstructionError,
    SchemaExportError,
    SchemaIssue,
    SchemaPath,
    Bytes,
    ByteError,
    ByteView,
    ByteViewMut,
    File,
    BufferedReader,
    BufferedWriter,
    FsError,
    EnvError,
    JsonValue,
    JsonView,
    JsonRaw,
    StringView,
    JsonError,
    JsonLimits,
    JsonBuildLimits,
    JsonEntry,
    JsonlReader,
    JsonlReaderView,
    JsonlWriter,
    JsonlWriterView,
    JsonlError,
    Deque,
    OrderedMap,
    OrderedSet,
    Heap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intrinsic {
    ProgramVoidCallback,
    ProgramReusableVoidCallback,
    OwnedVoidCallback,
    SharedCallback,
    ObservedValue,
}

/// Recursive metadata contains identities rather than checked-arena IDs or type
/// spelling. The resolver must intern it in the destination checked program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signature {
    String,
    Boolean,
    Int,
    Number,
    Void,
    Null,
    Named(NativeTypeId),
    Applied(NativeTypeId, &'static [Signature]),
    /// Index in the binding's explicit type parameter list, never a type name.
    Parameter(usize),
    /// Absence uses the ordinary undefined alternative, including borrowed values.
    Optional(&'static Signature),
    Array(&'static Signature),
    Union(&'static [Signature]),
    Result(&'static Signature, &'static Signature),
    Intrinsic(Intrinsic),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderError {
    MissingType(NativeTypeId),
    MissingParameter(usize),
    Arity(NativeTypeId),
    Depth,
    Size,
}

impl Signature {
    /// Find scoped native storage without rendering names or recursing on the
    /// Rust stack. Signature and declaration-field edges share the same bounds.
    pub fn contains_borrowed_native(&self) -> Result<bool, RenderError> {
        self.contains_borrowed_native_with_parameters(&[])
    }

    pub fn contains_borrowed_native_with_parameters(&self, parameters: &[Signature]) -> Result<bool, RenderError> {
        if matches!(
            self,
            Self::String
                | Self::Boolean
                | Self::Int
                | Self::Number
                | Self::Void
                | Self::Intrinsic(_)
        ) {
            return Ok(false);
        }
        let mut pending = vec![(self, 0usize)];
        let mut visited = 0;
        while let Some((signature, depth)) = pending.pop() {
            if depth > 128 {
                return Err(RenderError::Depth);
            }
            visited += 1;
            if visited > 4096 {
                return Err(RenderError::Size);
            }
            match signature {
                Self::Parameter(index) => pending.push((parameters.get(*index).ok_or(RenderError::MissingParameter(*index))?, depth + 1)),
                Self::Named(id) => {
                    let declaration = crate::native_types::declaration(*id);
                    if !declaration.type_parameters.is_empty() { return Err(RenderError::Arity(*id)); }
                    if declaration.borrow_access().is_some() {
                        return Ok(true);
                    }
                    pending.extend(
                        declaration
                            .fields
                            .iter()
                            .map(|field| (&field.signature, depth + 1)),
                    );
                }
                Self::Applied(id, arguments) => {
                    let declaration = crate::native_types::declaration(*id);
                    if declaration.type_parameters.len() != arguments.len() { return Err(RenderError::Arity(*id)); }
                    if declaration.borrow_access().is_some() { return Ok(true); }
                    pending.extend(arguments.iter().map(|argument| (argument, depth + 1)));
                    pending.extend(declaration.fields.iter().map(|field| (&field.signature, depth + 1)));
                }
                Self::Union(members) => pending.extend(members.iter().map(|member| (member, depth + 1))),
                Self::Array(element) | Self::Optional(element) => pending.push((element, depth + 1)),
                Self::Result(value, error) => {
                    pending.push((error, depth + 1));
                    pending.push((value, depth + 1));
                }
                _ => {}
            }
        }
        Ok(false)
    }

    /// Render using the caller's visible native aliases. Unknown names stay an
    /// error instead of being guessed from an unrelated declaration's spelling.
    pub fn render(&self, visible: impl FnMut(NativeTypeId) -> Option<String>) -> Result<std::borrow::Cow<'static, str>, RenderError> {
        self.render_with_parameters(visible, |_| None)
    }

    pub fn render_with_parameters(&self, mut visible: impl FnMut(NativeTypeId) -> Option<String>, mut parameter: impl FnMut(usize) -> Option<String>) -> Result<std::borrow::Cow<'static, str>, RenderError> {
        let primitive = match self {
            Self::String => Some("string"), Self::Boolean => Some("boolean"),
            Self::Int => Some("int"), Self::Number => Some("number"), Self::Void => Some("void"), Self::Null => Some("null"),
            Self::Intrinsic(Intrinsic::ProgramReusableVoidCallback) => Some("Mutable<Function<void>>"),
            Self::Intrinsic(Intrinsic::ProgramVoidCallback) => Some("Owned<Function<void>>"),
            Self::Intrinsic(Intrinsic::OwnedVoidCallback) => Some("Owned<Function<void>>"),
            Self::Intrinsic(Intrinsic::SharedCallback) => Some("Function"),
            Self::Intrinsic(Intrinsic::ObservedValue) => Some("unknown"),
            _ => None,
        };
        if let Some(text) = primitive { return Ok(std::borrow::Cow::Borrowed(text)); }
        enum Work<'a> { Type(&'a Signature, usize), Text(&'static str) }
        let mut pending = vec![Work::Type(self, 0)];
        let mut output = String::new();
        let mut visited = 0;
        while let Some(work) = pending.pop() {
            match work {
                Work::Text(text) => output.push_str(text),
                Work::Type(signature, depth) => {
                    if depth > 128 { return Err(RenderError::Depth); }
                    visited += 1;
                    if visited > 4096 { return Err(RenderError::Size); }
                    match signature {
                        Self::String => output.push_str("string"),
                        Self::Boolean => output.push_str("boolean"),
                        Self::Int => output.push_str("int"),
                        Self::Number => output.push_str("number"),
                        Self::Void => output.push_str("void"),
                        Self::Null => output.push_str("null"),
                        Self::Named(id) => {
                            if !crate::native_types::declaration(*id).type_parameters.is_empty() { return Err(RenderError::Arity(*id)); }
                            output.push_str(&visible(*id).ok_or(RenderError::MissingType(*id))?);
                        }
                        Self::Applied(id, arguments) => {
                            if crate::native_types::declaration(*id).type_parameters.len() != arguments.len() { return Err(RenderError::Arity(*id)); }
                            output.push_str(&visible(*id).ok_or(RenderError::MissingType(*id))?);
                            output.push('<');
                            pending.push(Work::Text(">"));
                            for (index, argument) in arguments.iter().enumerate().rev() {
                                pending.push(Work::Type(argument, depth + 1));
                                if index > 0 { pending.push(Work::Text(", ")); }
                            }
                        }
                        Self::Parameter(index) => output.push_str(&parameter(*index).ok_or(RenderError::MissingParameter(*index))?),
                        Self::Intrinsic(intrinsic) => output.push_str(match intrinsic {
                            Intrinsic::ProgramReusableVoidCallback => "Mutable<Function<void>>",
                            Intrinsic::ProgramVoidCallback | Intrinsic::OwnedVoidCallback => "Owned<Function<void>>",
                            Intrinsic::SharedCallback => "Function",
                            Intrinsic::ObservedValue => "unknown",
                        }),
                        Self::Array(element) => {
                            pending.push(Work::Text("[]"));
                            pending.push(Work::Type(element, depth + 1));
                        }
                        Self::Union(members) => {
                            output.push('(');
                            pending.push(Work::Text(")"));
                            for (index,member) in members.iter().enumerate().rev() {
                                pending.push(Work::Type(member,depth+1));
                                if index>0 {pending.push(Work::Text(" | "));}
                            }
                        }
                        Self::Optional(element) => {
                            output.push('(');
                            pending.push(Work::Text(" | undefined)"));
                            pending.push(Work::Type(element, depth + 1));
                        }
                        Self::Result(value, error) => {
                            output.push_str("Result<");
                            pending.push(Work::Text(">"));
                            pending.push(Work::Type(error, depth + 1));
                            pending.push(Work::Text(", "));
                            pending.push(Work::Type(value, depth + 1));
                        }
                    }
                }
            }
            if output.len() > 65536 { return Err(RenderError::Size); }
        }
        Ok(std::borrow::Cow::Owned(output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_native_signatures_require_explicit_type_arguments() {
        let signature = Signature::Result(&Signature::Array(&Signature::Parameter(0)), &Signature::Parameter(1));
        assert_eq!(signature.render(|_| None), Err(RenderError::MissingParameter(0)));
        assert_eq!(signature.render_with_parameters(|_| None, |index| ["T", "E"].get(index).map(|s| s.to_string())).unwrap(), "Result<T[], E>");
        assert_eq!(Signature::Parameter(0).contains_borrowed_native(), Err(RenderError::MissingParameter(0)));
        assert_eq!(signature.contains_borrowed_native_with_parameters(&[Signature::Named(NativeTypeId::ByteView), Signature::Int]), Ok(true));
        assert_eq!(signature.contains_borrowed_native_with_parameters(&[Signature::Int, Signature::Boolean]), Ok(false));
        assert_eq!(Signature::Parameter(0).contains_borrowed_native_with_parameters(&[Signature::Parameter(0)]), Err(RenderError::Depth));
        let optional = Signature::Array(&Signature::Optional(&Signature::Parameter(0)));
        assert_eq!(optional.render_with_parameters(|_| None, |_| Some("T".into())).unwrap(), "(T | undefined)[]");
        assert_eq!(optional.contains_borrowed_native_with_parameters(&[Signature::Named(NativeTypeId::ByteView)]), Ok(true));
    }

    #[test]
    fn applied_native_signatures_validate_arity_and_follow_element_borrows() {
        let queue = Signature::Applied(NativeTypeId::Deque, &[Signature::Parameter(0)]);
        assert_eq!(queue.render_with_parameters(|_| Some("Queue".into()), |_| Some("Window".into())).unwrap(), "Queue<Window>");
        assert_eq!(queue.contains_borrowed_native_with_parameters(&[Signature::Named(NativeTypeId::ByteView)]), Ok(true));
        assert_eq!(Signature::Applied(NativeTypeId::Deque, &[]).render(|_| Some("Queue".into())), Err(RenderError::Arity(NativeTypeId::Deque)));
        assert_eq!(Signature::Named(NativeTypeId::Deque).contains_borrowed_native(), Err(RenderError::Arity(NativeTypeId::Deque)));
    }

    #[test]
    fn nested_signatures_keep_native_identity_and_use_visible_aliases() {
        static ARRAY: Signature = Signature::Array(&Signature::Named(NativeTypeId::Bytes));
        let result = Signature::Result(&ARRAY, &Signature::Named(NativeTypeId::ByteError));
        assert_eq!(result.render(|id| Some(match id { NativeTypeId::Job => "Job".into(), NativeTypeId::AsyncError => "AsyncError".into(), NativeTypeId::AsyncChannel => "Channel".into(), NativeTypeId::AsyncSender => "Sender".into(), NativeTypeId::TaskContext => "TaskContext".into(), NativeTypeId::Channel => "Channel".into(), NativeTypeId::Sender => "Sender".into(), NativeTypeId::ChannelError => "ChannelError".into(), NativeTypeId::SendError => "SendError".into(), NativeTypeId::Thread => "Thread".into(), NativeTypeId::ScopedThread => "ScopedThread".into(), NativeTypeId::ThreadError => "ThreadError".into(), NativeTypeId::SpawnError => "SpawnError".into(), NativeTypeId::ThreadPoolStats => "ThreadPoolStats".into(), NativeTypeId::ThreadPoolOptions => "ThreadPoolOptions".into(), NativeTypeId::ThreadPool => "ThreadPool".into(), NativeTypeId::TestRegistry => "TestRegistry".into(), NativeTypeId::SchemaExportError => "SchemaExportError".into(), NativeTypeId::ValidationContext => "ValidationContext".into(), NativeTypeId::SchemaConstructionError => "SchemaConstructionError".into(), NativeTypeId::ValidationError => "ValidationError".into(), NativeTypeId::SchemaIssue => "SchemaIssue".into(), NativeTypeId::SchemaPath => "SchemaPath".into(), NativeTypeId::Schema => "Schema".into(), NativeTypeId::Bytes => "Buffer".into(), NativeTypeId::ByteError => "ErrorInfo".into(), NativeTypeId::ByteView => "Window".into(), NativeTypeId::ByteViewMut => "MutableWindow".into(), NativeTypeId::File => "Handle".into(), NativeTypeId::BufferedReader => "Reader".into(), NativeTypeId::BufferedWriter => "Writer".into(), NativeTypeId::FsError => "FileFailure".into(), NativeTypeId::EnvError => "EnvironmentFailure".into(), NativeTypeId::JsonValue => "JsonValue".into(), NativeTypeId::JsonView => "JsonView".into(), NativeTypeId::JsonRaw => "JsonRaw".into(), NativeTypeId::StringView => "StringView".into(), NativeTypeId::JsonError => "JsonError".into(), NativeTypeId::JsonLimits => "JsonLimits".into(), NativeTypeId::JsonBuildLimits => "JsonBuildLimits".into(), NativeTypeId::JsonEntry => "JsonEntry".into(), NativeTypeId::JsonlReader => "JsonlReader".into(), NativeTypeId::JsonlReaderView => "JsonlReaderView".into(), NativeTypeId::JsonlWriter => "JsonlWriter".into(), NativeTypeId::JsonlWriterView => "JsonlWriterView".into(), NativeTypeId::JsonlError => "JsonlError".into(), NativeTypeId::Deque => "Queue".into(), NativeTypeId::OrderedMap => "TreeMap".into(), NativeTypeId::OrderedSet => "TreeSet".into(), NativeTypeId::Heap => "PriorityQueue".into() })).unwrap(), "Result<Buffer[], ErrorInfo>");
        assert_eq!(Signature::Named(NativeTypeId::Bytes).render(|_| None), Err(RenderError::MissingType(NativeTypeId::Bytes)));
    }

    #[test]
    fn signature_rendering_bounds_depth_without_recursive_frames() {
        let mut value: &'static Signature = &Signature::Int;
        for _ in 0..128 { value = Box::leak(Box::new(Signature::Array(value))); }
        assert_eq!(value.render(|_| None).unwrap(), format!("int{}", "[]".repeat(128)));
        let excessive = Signature::Array(value);
        assert_eq!(excessive.render(|_| None), Err(RenderError::Depth));
    }


    #[test]
    fn signature_rendering_bounds_broad_metadata_and_alias_output() {
        let mut value: &'static Signature = &Signature::Int;
        for _ in 0..13 { value = Box::leak(Box::new(Signature::Result(value, value))); }
        assert_eq!(value.render(|_| None), Err(RenderError::Size));
        assert_eq!(Signature::Named(NativeTypeId::Bytes).render(|_| Some("x".repeat(65537))), Err(RenderError::Size));
    }

    #[test]
    fn borrowed_native_shapes_keep_identity_through_nested_signatures() {
        let signature = Signature::Result(
            &Signature::Array(&Signature::Named(NativeTypeId::ByteView)),
            &Signature::Named(NativeTypeId::ByteError),
        );
        assert_eq!(signature.contains_borrowed_native(), Ok(true));
        assert_eq!(
            signature
                .render(|id| Some(crate::native_types::declaration(id).name.into()))
                .unwrap(),
            "Result<ByteView[], ByteError>"
        );
        assert_eq!(
            Signature::Named(NativeTypeId::ByteViewMut).contains_borrowed_native(),
            Ok(true)
        );
        assert_eq!(
            Signature::Result(&Signature::Int, &Signature::Named(NativeTypeId::ByteView))
                .contains_borrowed_native(),
            Ok(true)
        );
        assert_eq!(
            Signature::Named(NativeTypeId::Bytes).contains_borrowed_native(),
            Ok(false)
        );
        assert_eq!(
            Signature::Named(NativeTypeId::ByteError).contains_borrowed_native(),
            Ok(false)
        );
        let mut deep: &'static Signature = &Signature::Int;
        for _ in 0..129 {
            deep = Box::leak(Box::new(Signature::Array(deep)));
        }
        assert_eq!(deep.contains_borrowed_native(), Err(RenderError::Depth));
        let mut broad: &'static Signature = &Signature::Int;
        for _ in 0..13 {
            broad = Box::leak(Box::new(Signature::Result(broad, broad)));
        }
        assert_eq!(broad.contains_borrowed_native(), Err(RenderError::Size));
    }

    #[test]
    fn ordinary_native_signatures_preserve_existing_help() {
        use crate::native_bindings::NativeType;
        for (ty, expected) in [(NativeType::String,"string"),(NativeType::Boolean,"boolean"),(NativeType::Int,"int"),(NativeType::Number,"number"),(NativeType::Void,"void"),(NativeType::OwnedVoidCallback,"Owned<Function<void>>"),(NativeType::SharedCallback,"Function"),(NativeType::ObservedValue,"unknown")] {
            let rendered = ty.signature().render(|_| None).unwrap();
            assert!(matches!(rendered, std::borrow::Cow::Borrowed(_)));
            assert_eq!(rendered, expected);
        }
    }
}
