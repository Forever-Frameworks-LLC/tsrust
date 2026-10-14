//! Channel payloads cross the backend boundary by move, including failed sends.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Emitter<'_> {
    pub(super) fn channel_constructor_expression(&self, fun: &CheckedFunction, expr: &Expr, mode: Mode) -> Option<String> {
        let Some(Type::Record(record)) = expr.ty else { return None; };
        if self.program.records[record].native_type != Some(NativeTypeId::Channel) { return None; }
        let ExprKind::NativeConstructor(args, _) = &expr.kind else { return None; };
        let capacity = self.expr(fun, &args[0], Mode::Read);
        let failed = self.runtime_failure_exit("RangeError", "range", "Channel capacity must be 1..1000000.", expr.span).unwrap_or_else(|| "panic!(\"Channel capacity must be 1..1000000.\")".into());
        let code = format!("match <{}>::new({capacity}){{Ok(channel)=>channel,Err(_)=>{{{failed}}}}}", self.native_type(expr.ty.unwrap()));
        Some(match mode { Mode::Read => format!("&({code})"), Mode::Mutate => format!("&mut ({code})"), _ => code })
    }
    pub(super) fn channel_native(&self, fun: &CheckedFunction, expr: &Expr, name: &str, args: &[Expr]) -> Option<String> {
        if !name.starts_with("__channel") && !name.starts_with("__sender") { return None; }
        let receiver = self.expr(fun, &args[0], if name == "__senderClose" { Mode::Mutate } else { Mode::Read });
        if name == "__channelSender" { return Some(format!("({receiver}).sender()")); }
        if matches!(name, "__channelClose" | "__senderClose") { return Some(format!("({receiver}).close()")); }
        let Type::Union(union) = expr.ty.unwrap() else { unreachable!() };
        let mut ok = String::new(); let mut err = String::new();
        for (tag, member) in self.program.unions[union.index()].members.iter().enumerate() {
            let Type::Record(record) = member else { unreachable!() };
            let declaration = &self.program.records[*record];
            let success = declaration.application.as_ref().unwrap().declaration.text == "Result#Ok";
            let payload = if success { "value".to_owned() } else {
                let Type::Record(failure) = declaration.fields[0].ty.unwrap() else { unreachable!() };
                let fields = if name.starts_with("__sender") { "field0:String::Borrowed(error.code),field1:error.value" } else { "field0:String::Borrowed(error)" };
                format!("Record{}{{{fields}{}}}", self.native_record_declaration(failure), self.schema_record_order(failure))
            };
            let fields = if declaration.fields.is_empty() { String::new() } else { format!("field0:{payload},") };
            let value = format!("Union{}::V{tag}(Record{}{{{fields}{}}})", union.index(), self.native_record_declaration(*record), self.schema_record_order(*record).trim_start_matches(','));
            if success { ok = value; } else { err = value; }
        }
        let call = match name {
            "__senderSend" | "__senderTrySend" => format!("{{let sender={receiver};sender.send({}, {})}}", self.expr(fun, &args[1], Mode::Consume), name == "__senderSend"),
            _ => format!("({receiver}).receive({})", name == "__channelReceive"),
        };
        Some(format!("match {call}{{Ok(value)=>{ok},Err(error)=>{err}}}"))
    }
}
