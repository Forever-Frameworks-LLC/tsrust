//! Finite native callable families. Source storage integration is a separate gate.
use crate::analysis::{ArgumentContract, CallbackContract, CheckedClosure};
use crate::callable_contracts::Contracts;
use crate::callable_storage::StorageContract;
use crate::syntax::Type;
use crate::{Diagnostic, Mode, Span};
use std::fmt::Write;

pub(crate) struct Family<'a> {
    contract: &'a CallbackContract,
    invocation: Mode,
    origins: Vec<usize>,
}

impl<'a> Family<'a> {
    pub(crate) fn checked(
        contract: &'a CallbackContract,
        signatures: &Contracts,
        origins: impl IntoIterator<Item = usize>,
        closures: &[CheckedClosure],
        span: Span,
    ) -> Result<Self, Diagnostic> {
        let storage =
            StorageContract::checked(signatures, contract.structural).ok_or_else(|| {
                Diagnostic::new(
                    "callback-escape",
                    "Callable storage requires an owned contract.",
                    span,
                )
            })?;
        Self::checked_storage(contract, signatures, origins, closures, span, storage, |actual, expected| actual == expected)
    }

    #[cfg(test)]
    pub(crate) fn checked_shared(
        contract: &'a CallbackContract,
        signatures: &Contracts,
        origins: impl IntoIterator<Item = usize>,
        closures: &[CheckedClosure],
        span: Span,
    ) -> Result<Self, Diagnostic> {
        let storage = StorageContract::checked_shared(signatures, contract.structural)
            .ok_or_else(|| Diagnostic::new("callback-access", "Shared callable storage requires a readonly contract.", span))?;
        Self::checked_storage(contract, signatures, origins, closures, span, storage, |actual, expected| actual == expected)
    }

    pub(crate) fn checked_results(
        contract: &'a CallbackContract, signatures: &Contracts,
        origins: impl IntoIterator<Item=usize>, closures: &[CheckedClosure], span: Span,
        equivalent: impl Fn(Type,Type)->bool,
    )->Result<Self,Diagnostic> {
        let storage=if contract.kind==crate::syntax::CallbackKind::SharedRead {StorageContract::checked_shared(signatures,contract.structural)} else {StorageContract::checked(signatures,contract.structural)}
            .ok_or_else(||Diagnostic::new("callback-escape","Callable storage requires an owned contract.",span))?;
        Self::checked_storage(contract,signatures,origins,closures,span,storage,equivalent)
    }

    fn checked_storage(
        contract: &'a CallbackContract,
        signatures: &Contracts,
        origins: impl IntoIterator<Item = usize>,
        closures: &[CheckedClosure],
        span: Span,
        storage: StorageContract,
        equivalent: impl Fn(Type,Type)->bool,
    ) -> Result<Self, Diagnostic> {
        let mut origins = origins.into_iter().collect::<Vec<_>>();
        origins.sort_unstable();
        origins.dedup();
        if origins.iter().any(|&origin|closures.get(origin).is_some_and(|closure|closure.borrowed))
            && let Some(layout)=crate::native_view_callable_storage::family_slots(contract,signatures,origins.iter().copied(),closures,span)? {
            let mut error=Diagnostic::new("native-view-callable-storage",
                format!("This callback owns its environment, but its family has {} alternative(s) carrying borrowed byte views. Lifetime-bearing callable storage is not implemented yet.",layout.alternatives.len()),span);
            for alternative in &layout.alternatives {
                error=error.related(closures[alternative.origin].span,"Carried byte-view environment is created here.");
            }
            return Err(error);
        }
        for &origin in &origins {
            let closure = closures.get(origin).ok_or_else(|| {
                Diagnostic::new(
                    "callable-layout",
                    "Unknown callable environment origin.",
                    span,
                )
            })?;
            let closure = storage.environment(signatures, closure, span)?.closure();
            if closure.polymorphic || closure.function.template || closure.lifetime_count != 0 {
                return Err(Diagnostic::new(
                    "callable-layout",
                    "Callable storage needs a concrete owned environment layout.",
                    span,
                ));
            }
            if closure.parameters.len() != contract.params.len()
                || !equivalent(closure.function.result, contract.result)
            {
                return Err(Diagnostic::new(
                    "type",
                    "Callable environment signature does not match its storage contract.",
                    span,
                ));
            }
            for (&binding, expected) in closure.parameters.iter().zip(&contract.params) {
                let actual = &closure.function.bindings[binding];
                if actual.input_ty.unwrap_or(actual.ty) != expected.ty || actual.mode != expected.mode {
                    return Err(Diagnostic::new(
                        "callback-access",
                        "Callable storage parameter types and access must match exactly.",
                        span,
                    ));
                }
            }
        }
        if contract.params.iter().any(|param| {
            matches!(
                param.ty,
                Type::Callback(_) | Type::Callable(_) | Type::Closure(_) | Type::StoredCallable(_)
            )
        }) || match contract.result {
            Type::Closure(id) => closures.get(id).is_none_or(|closure| closure.borrowed || closure.lifetime_count != 0),
            Type::Callback(_) | Type::Callable(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_) => true,
            _ => false,
        } {
            return Err(Diagnostic::new(
                "callable-layout",
                "Nested callable storage signatures require their own concrete layout.",
                span,
            ));
        }
        Ok(Self {
            contract,
            invocation: storage.invocation(signatures),
            origins,
        })
    }

    #[cfg(test)]
    pub(crate) fn emit(
        &self, output: &mut String, id: usize,
        parameter_type: impl Fn(ArgumentContract) -> String,
        result_type: impl Fn(Type) -> String,
    ) {
        self.emit_counted(output, id, false, parameter_type, result_type);
    }

    #[cfg(test)]
    pub(crate) fn emit_counted(
        &self,
        output: &mut String,
        id: usize,
        counted: bool,
        parameter_type: impl Fn(ArgumentContract) -> String,
        result_type: impl Fn(Type) -> String,
    ) {
        self.emit_counted_mapped(output,id,counted,parameter_type,result_type,|_,call|call);
    }

    /// Convert each origin's invocation result once, before family dispatch returns.
    pub(crate) fn emit_counted_mapped(
        &self,output:&mut String,id:usize,counted:bool,
        parameter_type:impl Fn(ArgumentContract)->String,
        result_type:impl Fn(Type)->String,
        adapt:impl Fn(usize,String)->String,
    ) {
        let name = crate::emit::ty(Type::StoredCallable(id));
        if self.invocation == Mode::Read {
            self.emit_shared(output, &name, id, counted, &parameter_type, &result_type, &adapt);
        } else {
            self.emit_named(output, &name, counted, parameter_type, result_type, &adapt);
        }
    }

    fn emit_shared(
        &self, output: &mut String, name: &str, id: usize, counted: bool,
        parameter_type: impl Fn(ArgumentContract) -> String,
        result_type: impl Fn(Type) -> String,
        adapt: &impl Fn(usize,String)->String,
    ) {
        let environment = format!("TsrSharedEnvironment{id}");
        self.emit_named(output, &environment, counted, &parameter_type, &result_type, adapt);
        writeln!(output, "struct {name}(TsrSharedReadCallback<{environment}>);").unwrap();
        writeln!(output, "impl Clone for {name} {{ fn clone(&self)->Self {{Self(self.0.clone())}} }}").unwrap();
        writeln!(output, "impl {name} {{").unwrap();
        for origin in &self.origins {
            let value = if self.origins.len() == 1 {format!("{environment}(value)")} else {format!("{environment}::V{origin}(value)")};
            writeln!(output, "fn from_origin_{origin}(value:Closure{origin})->Self {{Self(TsrSharedReadCallback::new({value}))}}").unwrap();
        }
        let parameters=self.contract.params.iter().enumerate().map(|(index,&parameter)|format!("argument{index}: {}",parameter_type(parameter))).collect::<Vec<_>>().join(", ");
        let arguments=(0..self.contract.params.len()).map(|index|format!("argument{index}")).collect::<Vec<_>>().join(", ");
        let separator=if parameters.is_empty(){""}else{", "};
        let result=result_type(self.contract.result);
        writeln!(output,"fn call(&self{separator}{parameters})->{result} {{self.0.0.call({arguments})}}").unwrap();
        if counted {
            let forwarded=if arguments.is_empty(){"count".to_owned()}else{format!("count, {arguments}")};
            writeln!(output,"fn call_with_count(&self,count:usize{separator}{parameters})->{result} {{self.0.0.call_with_count({forwarded})}}").unwrap();
        }
        output.push_str("}\n");
    }

    fn emit_named(
        &self, output: &mut String, name: &str, counted: bool,
        parameter_type: impl Fn(ArgumentContract) -> String,
        result_type: impl Fn(Type) -> String,
        adapt: &impl Fn(usize,String)->String,
    ) {

        let singleton = self.origins.len() == 1;
        if singleton {
            writeln!(output, "struct {name}(Closure{});", self.origins[0]).unwrap();
        } else {
            writeln!(output, "enum {name} {{").unwrap();
            for origin in &self.origins {
                writeln!(output, "V{origin}(Closure{origin}),").unwrap();
            }
            output.push_str("}\n");
        }
        let receiver = match self.invocation {
            Mode::Consume => "mut self",
            Mode::Read => "&self",
            _ => "&mut self",
        };
        let parameters = self
            .contract
            .params
            .iter()
            .enumerate()
            .map(|(index, &param)| format!("argument{index}: {}", parameter_type(param)))
            .collect::<Vec<_>>()
            .join(", ");
        let arguments = (0..self.contract.params.len())
            .map(|index| format!("argument{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let separator = if parameters.is_empty() { "" } else { ", " };
        writeln!(
            output,
            "impl {name} {{ fn call({receiver}{separator}{parameters}) -> {} {{",
            result_type(self.contract.result)
        )
        .unwrap();
        if self.origins.is_empty() {
            output.push_str(if self.invocation == Mode::Consume { "match self {}\n" } else { "match *self {}\n" });
        } else if singleton {
            writeln!(output, "{}",adapt(self.origins[0],format!("self.0.call({arguments})"))).unwrap();
        } else {
            output.push_str("match self {\n");
            let mutable = if self.invocation == Mode::Consume {
                "mut "
            } else {
                ""
            };
            for origin in &self.origins {
                writeln!(
                    output,
                    "Self::V{origin}({mutable}environment) => {},",adapt(*origin,format!("environment.call({arguments})"))
                )
                .unwrap();
            }
            output.push_str("}\n");
        }
        if !counted { output.push_str("} }\n"); return; }
        output.push_str("}\n");
        if counted {
            writeln!(output, "fn call_with_count({receiver}, count: usize{separator}{parameters}) -> {} {{", result_type(self.contract.result)).unwrap();
            let count_arguments = if arguments.is_empty() { "count".to_owned() } else { format!("count, {arguments}") };
            if self.origins.is_empty() {
                output.push_str(if self.invocation == Mode::Consume { "match self {}\n" } else { "match *self {}\n" });
            } else if singleton {
                writeln!(output, "{}",adapt(self.origins[0],format!("self.0.call_with_count({count_arguments})"))).unwrap();
            } else {
                output.push_str("match self {\n");
                let mutable = if self.invocation == Mode::Consume { "mut " } else { "" };
                for origin in &self.origins {
                    writeln!(output, "Self::V{origin}({mutable}environment) => {},",adapt(*origin,format!("environment.call_with_count({count_arguments})"))).unwrap();
                }
                output.push_str("}\n");
            }
            output.push_str("}\n");
        }
        output.push_str("}\n");
    }

    #[cfg(test)]
    pub(crate) fn emit_mutable_adapter(
        &self, output: &mut String, id: usize, generics: &str,
        arguments: &str, result: &str,
    ) {
        self.emit_invocation_adapters_counted(output, id, false, generics, arguments, result);
    }

    pub(crate) fn emit_invocation_adapters_counted(
        &self,
        output: &mut String,
        id: usize,
        counted: bool,
        generics: &str,
        arguments: &str,
        result: &str,
    ) {
        if !matches!(self.invocation, Mode::Read | Mode::Mutate | Mode::Consume) { return; }
        let forwarded = (0..self.contract.params.len())
            .map(|index| format!("args.{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let name = crate::emit::ty(Type::StoredCallable(id));
        if self.invocation == Mode::Consume {
            let counted_method = if counted {
                let count_arguments=if forwarded.is_empty(){"count".to_owned()}else{format!("count, {forwarded}")};
                format!("fn invoke_once_with_count(self,args:{arguments},count:usize)->{result} {{self.call_with_count({count_arguments})}}")
            } else {String::new()};
            writeln!(output,"impl{generics} OnceCallback<{arguments}, {result}> for {name} {{fn invoke_once(self,args:{arguments})->{result} {{self.call({forwarded})}} {counted_method}}}").unwrap();
            return;
        }
        if self.invocation == Mode::Read {
            let counted_method = if counted {
                let count_arguments=if forwarded.is_empty(){"count".to_owned()}else{format!("count, {forwarded}")};
                format!("fn invoke_read_with_count(&self,args:{arguments},count:usize)->{result} {{self.call_with_count({count_arguments})}}")
            } else {String::new()};
            writeln!(output,"impl{generics} ReadCallback<{arguments}, {result}> for {name} {{fn invoke_read(&self,args:{arguments})->{result} {{self.call({forwarded})}} {counted_method}}}").unwrap();
        }
        if counted {
            let count_arguments = if forwarded.is_empty() { "count".to_owned() } else { format!("count, {forwarded}") };
            writeln!(output, "impl{generics} Callback<{arguments}, {result}> for {name} {{ fn invoke(&mut self, args: {arguments}) -> {result} {{ self.call({forwarded}) }} fn invoke_with_count(&mut self,args:{arguments},count:usize)->{result} {{ self.call_with_count({count_arguments}) }} }}").unwrap();
        } else {
        writeln!(output, "impl{generics} Callback<{arguments}, {result}> for {name} {{ fn invoke(&mut self, args: {arguments}) -> {result} {{ self.call({forwarded}) }} }}").unwrap();
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::{
        fs,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct Temporary(std::path::PathBuf);
    impl Drop for Temporary {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    pub(crate) fn execute(rust: &str, expected: &str) {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = Temporary(
            std::env::temp_dir().join(format!("tsrust-family-{}-{stamp}", std::process::id())),
        );
        fs::create_dir(&directory.0).unwrap();
        let source = directory.0.join("main.rs");
        fs::write(&source, rust).unwrap();
        for level in [0, 3] {
            let binary = directory.0.join(format!("program-{level}"));
            let compilation = Command::new("rustup")
                .args([
                    "run",
                    crate::RUST_BACKEND_VERSION,
                    "rustc",
                    "--edition=2024",
                ])
                .arg(format!("-Copt-level={level}"))
                .arg(&source)
                .arg("-o")
                .arg(&binary)
                .output()
                .unwrap();
            assert!(
                compilation.status.success(),
                "{}",
                String::from_utf8_lossy(&compilation.stderr)
            );
            let output = Command::new(&binary).output().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        }
    }

    #[test]
    fn exception_family_converts_fallible_and_pure_origins_with_count() {
        let program=crate::check("function accept(f:Function<int,[n:int]>):void{}function main(){}").unwrap().program;
        let family=Family{contract:&program.callback_contracts[0],invocation:Mode::Consume,origins:vec![0,1]};
        let mut rust=String::from("struct Owner;impl Drop for Owner{fn drop(&mut self){DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);}}static DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);struct Closure0(Box<Owner>);impl Closure0{fn call(self,n:i64)->Result<i64,Box<Owner>>{if n==0{Err(self.0)}else{Ok(64)}}fn call_with_count(self,count:usize,n:i64)->Result<i64,Box<Owner>>{assert_eq!(count,7);self.call(n)}}struct Closure1;impl Closure1{fn call(self,n:i64)->i64{n}fn call_with_count(self,count:usize,n:i64)->i64{assert_eq!(count,7);n}}enum FamilyError{Owned(Box<Owner>)}");
        family.emit_counted_mapped(&mut rust,91,true,|_|"i64".into(),|_|"Result<i64,FamilyError>".into(),|origin,call|if origin==0{format!("({call}).map_err(FamilyError::Owned)")}else{format!("Ok({call})")});
        rust.push_str("fn main(){assert!(matches!(StoredCallable91::V0(Closure0(Box::new(Owner))).call(1),Ok(64)));assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),1);let error=StoredCallable91::V0(Closure0(Box::new(Owner))).call_with_count(7,0);assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),1);match error{Err(FamilyError::Owned(owner))=>drop(owner),_=>panic!(\"owned family error lost\")};assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),2);assert!(matches!(StoredCallable91::V1(Closure1).call(64),Ok(64)));assert!(matches!(StoredCallable91::V1(Closure1).call_with_count(7,64),Ok(64)));}");
        execute(&rust, "");
    }
    #[test]
    fn checked_factories_dispatch_multiple_owned_origins_and_singleton_at_o0_o3() {
        for once in [false, true] {
            let source = if once {
                "type State = { value: int; }; function left(start: int): () => int { const state: State = { value: start }; return (): int => { const moved = state; return moved.value; }; } function right(start: int): () => int { const state: State = { value: start + 10 }; return (): int => { const moved = state; return moved.value; }; } function sink(action: Once<Function<int>>, hold: (inner: Once<Function<int>>) => int): int { return hold(action); } function main() {}"
            } else {
                "function left(start: int): () => int { let value = start; return (): int => { value = value + 1; return value; }; } function right(start: int): () => int { const value = start + 10; return (): int => { return value; }; } function sink(action: Function<int>): int { return action(); } function main() {}"
            };
            let mut checked = crate::check(source).unwrap_or_else(|error| panic!("{error:?}"));
            let program = &mut checked.program;
            let ids = program.functions[..2]
                .iter()
                .map(|function| match function.result {
                    Type::Closure(id) => id,
                    other => panic!("{other:?}"),
                })
                .collect::<Vec<_>>();
            let contract = program
                .callback_contracts
                .iter()
                .find(|contract| contract.kind.is_owned() && contract.kind.is_once() == once)
                .unwrap_or_else(|| panic!("kinds {:?}", program.callback_contracts.iter().map(|contract| contract.kind).collect::<Vec<_>>()));
            let family = Family::checked(
                contract,
                &program.callable_shapes,
                [ids[1], ids[0], ids[0]],
                &program.closures,
                Span::default(),
            )
            .unwrap();
            assert_eq!(family.origins.len(), 2);
            for &origin in &[ids[1], ids[0], ids[0]] {
                program
                    .stored_contracts
                    .register_origin(
                        0,
                        origin,
                        &program.callback_contracts,
                        &program.callable_shapes,
                        &program.closures,
                        Span::default(),
                    )
                    .unwrap();
            }
            let mut rust = crate::emit::emit(program).replace("fn main()", "fn original_main()");

            let singleton = Family::checked(
                contract,
                &program.callable_shapes,
                [ids[0]],
                &program.closures,
                Span::default(),
            )
            .unwrap();
            singleton.emit(&mut rust, 1, |_| unreachable!(), |_| "i64".into());
            singleton.emit_mutable_adapter(&mut rust, 1, "", "()", "i64");
            // "OnceCallback<...>" also ends with this text; match the reusable trait only.
            assert_eq!(rust.contains(" Callback<(), i64> for StoredCallable0"), !once);
            assert_eq!(rust.contains(" Callback<(), i64> for StoredCallable1"), !once);
            let mut run = format!(
                "fn main() {{ let mut first = StoredCallable0::V{}(function0(2)); let mut second = StoredCallable0::V{}(function1(5)); let mut single = StoredCallable1(function0(8)); println!(\"{{}} {{}} {{}}\", first.call(), second.call(), single.call());",
                ids[0], ids[1]
            );
            if !once {
                run.push_str("println!(\"{} {} {}\", Callback::invoke(&mut first, ()), Callback::invoke(&mut second, ()), Callback::invoke(&mut single, ()));");
            }
            run.push('}');
            rust.push_str(&run);
            execute(&rust, if once { "2 15 8\n" } else { "3 15 9\n4 15 10\n" });
            assert!(!rust.contains("Box<dyn"));
        }
    }
    #[test]
    fn borrowed_arguments_dispatch_without_becoming_environment_loans() {
        let source = "function left(offset: int): (value: ReadOnly<string>) => int { return (value: string): int => { return value.length + offset; }; } function right(offset: int): (value: ReadOnly<string>) => int { return (value: string): int => { return value.length + offset + 10; }; } function sink(action: Function<int, [value: ReadOnly<string>]>): int { return action(\"a\"); } function main() {}";
        let checked = crate::check(source).unwrap_or_else(|error| panic!("{error:?}"));
        let program = &checked.program;
        let ids = program.functions[..2]
            .iter()
            .map(|function| match function.result {
                Type::Closure(id) => id,
                other => panic!("{other:?}"),
            })
            .collect::<Vec<_>>();
        let contract = program
            .callback_contracts
            .iter()
            .find(|contract| contract.kind.is_owned())
            .unwrap();
        let family = Family::checked(
            contract,
            &program.callable_shapes,
            ids.iter().copied(),
            &program.closures,
            Span::default(),
        )
        .unwrap();
        let mut rust = crate::emit::emit(program).replace("fn main()", "fn original_main()");
        family.emit(
            &mut rust,
            0,
            |param| {
                assert_eq!(param.mode, Mode::Read);
                "&String".into()
            },
            |_| "i64".into(),
        );
        family.emit_mutable_adapter(&mut rust, 0, "<'input>", "(&'input String,)", "i64");
        rust.push_str(&format!("fn main() {{ let mut left = StoredCallable0::V{}(function0(2)); let mut right = StoredCallable0::V{}(function1(5)); println!(\"{{}} {{}}\", Callback::invoke(&mut left, (&String::Borrowed(\"abc\"),)), Callback::invoke(&mut right, (&String::Borrowed(\"xy\"),))); }}", ids[0], ids[1]));
        execute(&rust, "5 17\n");
    }

    #[test]
    fn tagged_storage_drops_noncloneable_payloads_on_replace_return_and_container_drop() {
        let source = "function left(start: int): () => int { return (): int => { return start; }; } function right(start: int): () => int { return (): int => { return start + 1; }; } function sink(action: Function<int>): int { return action(); } function main() {}";
        let checked = crate::check(source).unwrap_or_else(|error| panic!("{error:?}"));
        let program = &checked.program;
        let ids = program.functions[..2]
            .iter()
            .map(|function| match function.result {
                Type::Closure(id) => id,
                other => panic!("{other:?}"),
            })
            .collect::<Vec<_>>();
        let contract = program
            .callback_contracts
            .iter()
            .find(|contract| contract.kind.is_owned())
            .unwrap();
        let family = Family::checked(
            contract,
            &program.callable_shapes,
            ids.iter().copied(),
            &program.closures,
            Span::default(),
        )
        .unwrap();
        let mut rust = "#![allow(dead_code, unused_mut, unused_assignments)]\nuse std::sync::atomic::{AtomicUsize, Ordering}; static DROPS: AtomicUsize = AtomicUsize::new(0); struct Payload; impl Drop for Payload { fn drop(&mut self) { DROPS.fetch_add(1, Ordering::SeqCst); } }\n".to_owned();
        for id in &ids {
            rust.push_str(&format!("struct Closure{id}(Payload); impl Closure{id} {{ fn call(&mut self) -> i64 {{ 7 }} }}\n"));
        }
        family.emit(&mut rust, 0, |_| unreachable!(), |_| "i64".into());
        rust.push_str(&format!("fn early(value: StoredCallable0) {{ let _held = value; }} fn main() {{ let mut selected = StoredCallable0::V{}(Closure{}(Payload)); selected = StoredCallable0::V{}(Closure{}(Payload)); assert_eq!(selected.call(), 7); drop(selected); early(StoredCallable0::V{}(Closure{}(Payload))); let values = vec![StoredCallable0::V{}(Closure{}(Payload)), StoredCallable0::V{}(Closure{}(Payload))]; drop(values); println!(\"{{}}\", DROPS.load(Ordering::SeqCst)); }}", ids[0], ids[0], ids[1], ids[1], ids[0], ids[0], ids[0], ids[0], ids[1], ids[1]));
        execute(&rust, "5\n");
    }
    #[test]
    fn family_rejects_missing_origins_and_unresolved_or_mismatched_layouts() {
        let mut checked = crate::check("function make(start: int): () => int { return (): int => { return start; }; } function sink(action: Function<int>): int { return action(); } function main() {}").unwrap_or_else(|error| panic!("{error:?}"));
        let program = &mut checked.program;
        let Type::Closure(origin) = program.functions[0].result else {
            panic!("concrete origin")
        };
        let contract = program
            .callback_contracts
            .iter()
            .find(|contract| contract.kind.is_owned())
            .unwrap();
        let span = Span::new(2, 9);
        assert!(Family::checked(contract, &program.callable_shapes, [], &program.closures, span).is_ok());
        for origins in [vec![program.closures.len()]] {
            let error = Family::checked(
                contract,
                &program.callable_shapes,
                origins,
                &program.closures,
                span,
            )
            .err()
            .unwrap();
            assert_eq!(error.code, "callable-layout");
            assert_eq!(error.span, span);
        }
        program.closures[origin].function.result = Type::Number;
        assert_eq!(
            Family::checked(
                contract,
                &program.callable_shapes,
                [origin],
                &program.closures,
                span
            )
            .err()
            .unwrap()
            .code,
            "type"
        );
        program.closures[origin].function.result = Type::Int;
        program.closures[origin].polymorphic = true;
        assert_eq!(
            Family::checked(
                contract,
                &program.callable_shapes,
                [origin],
                &program.closures,
                span
            )
            .err()
            .unwrap()
            .code,
            "callable-layout"
        );
        program.closures[origin].polymorphic = false;
        program.closures[origin].borrowed = true;
        assert_eq!(
            Family::checked(
                contract,
                &program.callable_shapes,
                [origin],
                &program.closures,
                span
            )
            .err()
            .unwrap()
            .code,
            "callback-escape"
        );
    }
    #[test]
    fn checked_storage_wrappers_move_factory_results_into_native_families() {
        for multiple in [false, true] {
            let mut checked = crate::check("function left(start: int): () => int { return (): int => { return start; }; } function right(start: int): () => int { return (): int => { return start + 1; }; } function sink(action: Function<int>): int { return action(); } function main() { const first = left(2); const second = right(5); console.log(first(), second()); }").unwrap_or_else(|error| panic!("{error:?}"));
            let program = &mut checked.program;
            for index in 0..if multiple { 2 } else { 1 } {
                let crate::syntax::StmtKind::Let {
                    value,
                    binding: Some(binding),
                    ..
                } = &mut program.functions[3].body[index].kind
                else {
                    panic!("checked initializer")
                };
                let Type::Closure(origin) = value.ty.unwrap() else {
                    panic!("factory result")
                };
                program
                    .stored_contracts
                    .register_origin(
                        0,
                        origin,
                        &program.callback_contracts,
                        &program.callable_shapes,
                        &program.closures,
                        value.span,
                    )
                    .unwrap();
                let span = value.span;
                value.wrap_storage(0);
                assert_eq!(value.span, span);
                let binding = *binding;
                program.functions[3].bindings[binding].ty = Type::StoredCallable(0);
            }
            let function = &mut program.functions[3];
            let crate::syntax::StmtKind::Expr(console) = &mut function.body[2].kind else {
                panic!("checked output")
            };
            let crate::syntax::ExprKind::Call(_, arguments) = &mut console.kind else {
                panic!("console call")
            };
            for argument in arguments {
                if let Some(crate::syntax::CallTarget::Closure(binding)) = argument.target {
                    if matches!(function.bindings[binding].ty, Type::StoredCallable(_)) {
                        argument.target = Some(crate::syntax::CallTarget::StoredCallable(binding));
                        function.bindings[binding].mutable = true;
                    }
                }
            }
            let rust = crate::emit::emit(program);
            assert!(rust.contains(if multiple {
                "enum StoredCallable0"
            } else {
                "struct StoredCallable0"
            }));
            execute(&rust, "2 6\n");
        }
    }
}

#[cfg(test)]
#[test]
fn shared_family_emission_invokes_through_immutable_reference_counted_handles() {
    let checked=crate::check("function reader(actual:int[]):Function<int>{return():int=>{return actual.length;};}function main(){}").unwrap();
    let mut signatures=Contracts::default();
    let contract=CallbackContract{structural:signatures.intern([],Type::Int,Some(crate::syntax::CallbackKind::SharedRead)),parameter_names:vec![],params:vec![],result:Type::Int,kind:crate::syntax::CallbackKind::SharedRead};
    let family=Family::checked_shared(&contract,&signatures,[0],&checked.program.closures,Span::default()).unwrap();
    let mut rust=checked.emit_rust().replace("fn main()","fn source_main()");
    family.emit(&mut rust,99,|_|unreachable!(), |_|"i64".into());
    rust.push_str(include_str!("stdlib/testing/shared_callback_runtime.rs"));
    rust.push_str("fn main(){let first=StoredCallable99::from_origin_0(function0(NumericArray::from([1i64,2i64])));let second=first.clone();assert_eq!(first.call(),2);drop(first);assert_eq!(second.call(),2);println!(\"shared readonly family\");}");
    tests::execute(&rust,"shared readonly family\n");
}

#[cfg(test)]
#[test]
fn nested_result_requires_a_known_owned_environment() {
    let mut checked=crate::check("function make(value:int){return ()=>{return ()=>{return value;};};}function main(){const factories=[make(3)];}").unwrap();
    let program=&mut checked.program;
    let (_,declaration,origins)=program.stored_contracts.families().next().unwrap();
    let origins=origins.to_vec();let source=&program.callback_contracts[declaration];
    let mut contract=CallbackContract {structural:source.structural,parameter_names:source.parameter_names.clone(),params:source.params.clone(),result:source.result,kind:source.kind};
    let Type::Closure(result)=contract.result else {panic!("concrete nested result")};
    assert!(Family::checked(&contract,&program.callable_shapes,origins.clone(),&program.closures,Span::default()).is_ok());
    program.closures[result].borrowed=true;
    assert_eq!(Family::checked(&contract,&program.callable_shapes,origins.clone(),&program.closures,Span::default()).err().unwrap().code,"callable-layout");
    program.closures[result].borrowed=false;program.closures[result].lifetime_count=1;
    assert!(Family::checked(&contract,&program.callable_shapes,origins.clone(),&program.closures,Span::default()).is_err());
    program.closures[result].lifetime_count=0;contract.result=Type::Closure(usize::MAX);
    assert!(Family::checked(&contract,&program.callable_shapes,[],&program.closures,Span::default()).is_err());
}

#[cfg(test)]
#[test]
fn shared_family_handles_clone_without_cloning_environment_and_constructs_each_origin() {
    let mut signatures=Contracts::default();
    let contract=CallbackContract{structural:signatures.intern([],Type::Int,Some(crate::syntax::CallbackKind::SharedRead)),parameter_names:vec![],params:vec![],result:Type::Int,kind:crate::syntax::CallbackKind::SharedRead};
    let checked=crate::check("function first(actual:int[]):Function<int>{return():int=>{return actual.length;};}function second(actual:int[]):Function<int>{return():int=>{return actual.length+1;};}function main(){}").unwrap();
    let family=Family::checked_shared(&contract,&signatures,[0,1],&checked.program.closures,Span::default()).unwrap();
    let mut rust=checked.emit_rust().replace("fn main()","fn source_main()");
    rust.push_str(include_str!("stdlib/testing/shared_callback_runtime.rs"));
    family.emit(&mut rust,99,|_|unreachable!(), |_|"i64".into());
    rust.push_str("fn main(){let first=StoredCallable99::from_origin_0(function0(NumericArray::from([1i64,2i64])));let alias=first.clone();assert_eq!(first.call(),2);drop(first);assert_eq!(alias.call(),2);let second=StoredCallable99::from_origin_1(function1(NumericArray::from([1i64,2i64])));assert_eq!(second.call(),3);println!(\"shared origins preserved\");}");
    tests::execute(&rust,"shared origins preserved\n");
}

#[cfg(test)]
#[test]
fn checked_shared_factory_storage_uses_production_emission_and_handle_construction() {
    let mut checked=crate::check("function reader(actual:int[]):Function<int>{return():int=>{return actual.length;};}function query(predicate:ReadOnly<Function<int>>):int{return predicate();}function main(){}").unwrap();
    let program=&mut checked.program;
    let declaration=program.callback_contracts.len();
    let contract=CallbackContract{structural:program.callable_shapes.intern([],Type::Int,Some(crate::syntax::CallbackKind::SharedRead)),parameter_names:vec![],params:vec![],result:Type::Int,kind:crate::syntax::CallbackKind::SharedRead};
    let Type::StoredCallable(family)=program.stored_contracts.intern_shared(&program.callable_shapes,&contract,declaration).unwrap() else{unreachable!()};
    program.callback_contracts.push(contract);
    program.stored_contracts.register_origin(family,0,&program.callback_contracts,&program.callable_shapes,&program.closures,Span::default()).unwrap();
    program.functions[0].result=Type::StoredCallable(family);
    let crate::syntax::StmtKind::Return(Some(expression))=&mut program.functions[0].body[0].kind else{unreachable!()};
    expression.wrap_storage(family);
    let mut rust=checked.emit_rust().replace("fn main()","fn source_main()");
    rust.push_str("fn main(){let original=function0(NumericArray::from([1i64,2i64]));let alias=original.clone();assert_eq!(original.call(),2);drop(original);assert_eq!(alias.call(),2);assert_eq!(function1(&alias),2);println!(\"checked shared construction\");}");
    tests::execute(&rust,"checked shared construction\n");
}

#[cfg(test)]
#[test]
fn counted_shared_family_forwards_supplied_count_through_cloned_handles() {
    let checked=crate::check("function reader(actual:int[]):Function<int,[value:int]>{return(value:int):int=>{return actual.length+value;};}function main(){}").unwrap();
    let parameter=ArgumentContract{ty:Type::Int,mode:Mode::Copy,omission:crate::ParameterOmission::Required,rest:false};
    let mut signatures=Contracts::default();
    let contract=CallbackContract{structural:signatures.intern([(Type::Int,Mode::Copy)],Type::Int,Some(crate::syntax::CallbackKind::SharedRead)),parameter_names:vec!["value".into()],params:vec![parameter],result:Type::Int,kind:crate::syntax::CallbackKind::SharedRead};
    let family=Family::checked_shared(&contract,&signatures,[0],&checked.program.closures,Span::default()).unwrap();
    let mut rust=String::from("struct Closure0(Box<i64>);impl Closure0{fn call(&self,value:i64)->i64{*self.0+value}fn call_with_count(&self,count:usize,value:i64)->i64{*self.0+value+count as i64}}\n");
    rust.push_str(include_str!("stdlib/testing/shared_callback_runtime.rs"));
    family.emit_counted(&mut rust,99,true,|_|"i64".into(), |_|"i64".into());
    rust.push_str("fn main(){let original=StoredCallable99::from_origin_0(Closure0(Box::new(20)));let alias=original.clone();assert_eq!(original.call(10),30);assert_eq!(alias.call_with_count(2,10),32);drop(original);assert_eq!(alias.call_with_count(0,10),30);println!(\"shared counts preserved\");}");
    tests::execute(&rust,"shared counts preserved\n");
}
