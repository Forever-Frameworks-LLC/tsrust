//! Move callback environments and completion payloads through native threads.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Emitter<'_> {
    pub(super) fn thread_native(&self, fun: &CheckedFunction, expr: &Expr, id: usize, args: &[Expr]) -> Option<String> {
        let binding = &crate::native_bindings::BINDINGS[id];
        if binding.module != "std:threads" { return None; }
        self.native_bindings.borrow_mut().insert(id);
        if let Some(code) = self.channel_native(fun, expr, binding.name, args) { return Some(code); }
        if matches!(binding.name, "__ownedPoolStats") {
            let call = format!("({}).stats()", self.expr(fun, &args[0], Mode::Read));
            let Type::Record(record) = expr.ty.unwrap() else { unreachable!() };
            let fields = ["workers", "capacity", "queued", "running", "completed", "outstanding"].iter().enumerate().map(|(index, name)| format!("field{index}:stats.{name} as i64")).collect::<Vec<_>>().join(",");
            return Some(format!("{{let stats={call};Record{}{{{fields}{}}}}}", self.native_record_declaration(record), self.schema_record_order(record)));
        }
        if binding.name == "__taskCancelled" { return Some(format!("({}).cancellation_requested()", self.expr(fun, &args[0], Mode::Read))); }
        if binding.name == "cancellationRequested" { return Some("tsrust_native_threads::cancellation_requested()".into()); }
        if binding.name == "cancel" { return Some(format!("tsrust_native_threads::cancel({})", self.expr(fun, &args[0], Mode::Read))); }
        let callback_index = usize::from(binding.name == "__ownedPoolTask");
        let value = args.get(callback_index).map(|arg| self.expr(fun, arg, Mode::Consume)).unwrap_or_default();
        if binding.name == "detach" { return Some(format!("tsrust_native_threads::detach({value})")); }
        let Type::Union(union) = expr.ty.unwrap() else { unreachable!("thread Result") };
        let variants = self.program.unions[union.index()].members.iter().enumerate().map(|(tag, ty)| {
            let Type::Record(record) = ty else { unreachable!() };
            let application = self.program.records[*record].application.as_ref().unwrap();
            (application.declaration.text.as_str(), tag, *record)
        }).collect::<Vec<_>>();
        let (_, ok_tag, ok) = *variants.iter().find(|(name, _, _)| *name == "Result#Ok").unwrap();
        let (_, err_tag, err) = *variants.iter().find(|(name, _, _)| *name == "Result#Err").unwrap();
        let wrap = |tag: usize, record: usize, payload: &str| {
            let fields = if self.program.records[record].fields.is_empty() { String::new() } else { format!("field0:{payload},") };
            format!("Union{}::V{tag}(Record{}{{{fields}{}}})", union.index(), self.native_record_declaration(record), self.schema_record_order(record).trim_start_matches(','))
        };
        let ok_value = wrap(ok_tag, ok, "value");
        let Type::Record(failure) = self.program.records[err].fields[0].ty.unwrap() else { unreachable!() };
        let failure_name = self.native_record_declaration(failure);
        let order = self.schema_record_order(failure);
        let failed = wrap(err_tag, err, &format!("Record{failure_name}{{field0:String::Borrowed(error.code),field1:String::Owned(error.to_string()){order}}}"));
        let call = if matches!(binding.name, "__poolCreate" | "__ownedPoolShutdown") {
            let call = if binding.name == "__poolCreate" {
                if args.is_empty() { "tsrust_native_threads::create_default_pool()".into() } else {
                    let Type::Record(id) = args[0].ty.unwrap() else { unreachable!() };
                    let fields = &self.program.records[id].fields;
                    let workers = fields.iter().position(|field| field.name.text == "workers").unwrap();
                    let capacity = fields.iter().position(|field| field.name.text == "capacity").unwrap();
                    format!("{{let options={};tsrust_native_threads::create_pool(options.field{workers},options.field{capacity})}}", self.expr(fun, &args[0], Mode::Read))
                }
            } else { format!("({}).shutdown()", self.expr(fun, &args[0], Mode::Read)) };
            format!("match {call}{{Ok(value)=>{ok_value},Err(error)=>{failed}}}")
        } else if matches!(binding.name, "spawn" | "spawnScoped" | "__ownedPoolTask") {
            let Type::Closure(origin) = args[callback_index].ty.unwrap() else { unreachable!() };
            let Type::Record(handle) = self.program.records[ok].fields[0].ty.unwrap() else { unreachable!() };
            let parameters = &self.program.records[handle].application.as_ref().unwrap().resolved_arguments;
            let error_type = self.native_type(parameters[1]);
            let context_argument = self.program.closures[origin].parameters.first().map(|&id| match self.program.closures[origin].function.bindings[id].mode {
                Mode::Read => "&task_context", Mode::Mutate => "&mut task_context", _ => "task_context",
            }).unwrap_or("");
            let context_setup = if context_argument.is_empty() { "" } else if context_argument == "&mut task_context" { "let mut task_context=tsrust_native_threads::TaskContext::current();" } else { "let task_context=tsrust_native_threads::TaskContext::current();" };
            let completion = if let Some(name) = self.result_closures.borrow().get(&origin) {
                let runtime = self.program.records.iter().position(|record| record.name.text == "#RuntimeError").map(Type::Record);
                let unions = crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
                let converted = exception_contract_emit::checked_catch_value("error", name, &self.program.error_effects.closure_errors[origin], &unions, runtime, expr.span).expect("checked worker errors");
                format!("callback.call({context_argument}).map_err(|error|{converted})")
            } else { format!("Ok::<_,{error_type}>(callback.call({context_argument}))") };
            let mutable = if self.program.closures[origin].invocation == Mode::Mutate { "mut " } else { "" };
            let spawn: &str = match binding.name {
                "__ownedPoolTask" => "pool_receiver.task",
                _ => "tsrust_native_threads::spawn_dedicated",
            };
            let receiver = if binding.name == "__ownedPoolTask" { format!("let pool_receiver={};", self.expr(fun, &args[0], Mode::Read)) } else { String::new() };
            format!("{{{receiver}let {mutable}callback={value};match {spawn}(move||{{{context_setup}{completion}}}){{Ok(value)=>{ok_value},Err(error)=>{failed}}}}}")
        } else {
            let error_field = self.program.records[failure].fields[1].ty.unwrap();
            let Type::Record(handle) = args[0].ty.unwrap() else { unreachable!() };
            let thrown = self.program.records[handle].application.as_ref().unwrap().resolved_arguments[1];
            let inject = |ty, payload: &str| {
                if ty == error_field { return payload.to_owned(); }
                let Type::Union(id) = error_field else { return payload.into(); };
                let members = &self.program.unions[id.index()].members;
                if let Type::Union(source) = ty {
                    let arms = self.program.unions[source.index()].members.iter().enumerate().map(|(tag, member)| {
                        let target = members.iter().position(|candidate| candidate == member).expect("worker error member");
                        format!("Union{}::V{tag}(payload)=>Union{}::V{target}({})", source.index(), id.index(), self.union_transfer_payload(source,id,*member,"payload"))
                    }).collect::<Vec<_>>().join(",");
                    format!("match {payload}{{{arms}}}")
                } else {
                    let tag = members.iter().position(|&member| member == ty).expect("worker error member");
                    format!("Union{}::V{tag}({})", id.index(), self.union_store_payload(id,ty,payload))
                }
            };
            let worker = wrap(err_tag, err, &format!("Record{failure_name}{{field0:String::Borrowed(\"worker\"),field1:{}{order}}}", inject(thrown,"error")));
            let panic = wrap(err_tag, err, &format!("Record{failure_name}{{field0:String::Borrowed(\"panic\"),field1:{}{order}}}", inject(Type::Undefined,"TsrUndefined")));
            let cancelled = wrap(err_tag, err, &format!("Record{failure_name}{{field0:String::Borrowed(\"cancelled\"),field1:{}{order}}}", inject(Type::Undefined,"TsrUndefined")));
            format!("match tsrust_native_threads::join({value}){{Ok(value)=>{ok_value},Err(tsrust_native_threads::JoinError::Worker(error))=>{worker},Err(tsrust_native_threads::JoinError::Panic)=>{panic},Err(tsrust_native_threads::JoinError::Cancelled)=>{cancelled}}}")
        };
        Some(call)
    }
    pub(super) fn thread_record(&self, output: &mut String, id: usize) -> bool {
        let record = &self.program.records[id];
        if matches!(record.native_type, Some(NativeTypeId::Channel | NativeTypeId::Sender)) {
            let Some(application) = &record.application else { return true; };
            let payload = application.resolved_arguments[0];
            let name = if record.native_type == Some(NativeTypeId::Channel) { "Channel" } else { "Sender" };
            writeln!(output, "type Record{id} = tsrust_native_threads::{name}<{}>;", self.native_type(payload)).unwrap();
            return true;
        }
        if record.native_type == Some(NativeTypeId::TaskContext) { writeln!(output,"type Record{id} = tsrust_native_threads::TaskContext;").unwrap(); return true; }
        if record.native_type == Some(NativeTypeId::ThreadPool) { writeln!(output,"type Record{id} = tsrust_native_threads::OwnedPool;").unwrap(); return true; }
        if !matches!(record.native_type, Some(NativeTypeId::Thread | NativeTypeId::ScopedThread)) { return false; }
        if let Some(application) = &record.application {
            let parameters = &application.resolved_arguments;
            let name = "Thread";
            writeln!(output,"type Record{id} = tsrust_native_threads::{name}<{},{}>;",self.native_type(parameters[0]),self.native_type(parameters[1])).unwrap();
        }
        true
    }
}
