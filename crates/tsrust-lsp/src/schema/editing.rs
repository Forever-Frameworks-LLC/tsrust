//! Native import identity survives incomplete editing, before project checking finishes.
use super::*;
impl SourceIndex {
    /// Repair only a standalone unfinished member for completion metadata.
    /// Keep the following source intact; diagnostics never use this probe.
    pub(in crate::intelligence) fn unfinished_member_probe(&self) -> Option<(String,usize,usize,usize)> {
        for dot in 1..self.tokens.len() {
            if self.text(dot)!="." || !name(self.text(dot-1)) {continue;}
            let begin=dot-1;
            if begin>0 && !matches!(self.text(begin-1),"{"|";"|"}") {continue;}
            let next=self.tokens.get(dot+1);
            if next.is_some_and(|next| !matches!(next.text.as_str(),"}"|";")
                && !self.source[self.tokens[dot].span.end..next.span.start].contains('\n')) {continue;}
            let start=self.tokens[begin].span.start;
            let end=self.tokens[dot].span.end;
            let receiver_end=self.tokens[begin].span.end;
            let probe=format!("{}typeof {};{}",&self.source[..start],&self.source[start..receiver_end],&self.source[end..]);
            return Some((probe,start,7,receiver_end));
        }
        None
    }

    pub(in crate::intelligence) fn recover_native_imports(&mut self) {
        for start in 0..self.tokens.len() {
            if self.text(start)!="import" || self.tokens[start].scope!=0 || self.text(start+1)!="{" {continue;}
            let Some(close)=self.pairs[start+1] else {continue};
            if self.text(close+1)!="from" {continue;}
            let module=self.text(close+2).trim_matches(['\'', '"']).to_owned();
            if !module.starts_with("std:") {continue;}
            self.import_native_names(Span::new(self.tokens[start+1].span.end,self.tokens[close].span.start),&module);
            let mut cursor=start+2;
            while cursor<close {
                let exported=self.text(cursor).to_owned();
                if !name(&exported) {break;}
                let local=if self.text(cursor+1)=="as" {cursor+=2;cursor} else {cursor};
                let local_name=self.text(local).to_owned();
                if !name(&local_name) {break;}
                self.import_native(&module,&exported,&local_name,self.tokens[local].span);
                cursor+=1;
                if cursor<close && self.text(cursor)!="," {break;}
                cursor+=1;
            }
        }
    }

    pub(in crate::intelligence) fn recover_result_payloads(&mut self) {
        use tsrust_compiler::type_syntax::{parse,ValueAnnotationKind};
        for keyword in 0..self.tokens.len() {
            if self.text(keyword)!="match" || self.text(keyword+1)!="(" {continue;}
            let Some(close)=self.pairs[keyword+1] else {continue;};
            if self.text(close+1)!="{" {continue;}
            let Some((ty,end))=self.simple_expression_type(keyword+2,self.tokens[keyword].span.start,0) else {continue;};
            if end!=close || !ty.starts_with("Result<") {continue;}
            let Ok(annotation)=parse(&ty) else {continue;};
            let ValueAnnotationKind::Applied {arguments,..}=annotation.kind else {continue;};
            if arguments.len()!=2 {continue;}
            let Some(scope)=self.scopes.iter().position(|scope|scope.span.start==self.tokens[close+1].span.start) else {continue;};
            let end=self.pairs[close+1].unwrap_or(self.tokens.len());
            for pattern in close+2..end {
                if self.tokens[pattern].scope!=scope || !matches!(self.text(pattern),"Ok"|"Err") {continue;}
                if self.text(pattern+1)!="(" || self.text(pattern+3)!=")" || self.text(pattern+4)!="=>" || self.text(pattern+5)!="{" {continue;}
                let argument=&arguments[usize::from(self.text(pattern)=="Err")];
                let payload=ty[argument.span.start..argument.span.end].to_owned();
                let span=self.tokens[pattern+2].span;
                if let Some(symbol)=self.symbols.iter_mut().find(|symbol|symbol.span==span && symbol.ty.is_empty()) {
                    symbol.signature=format!("(match) {}: {payload}",symbol.name);
                    symbol.ty=payload;
                }
            }
        }
    }

    pub(in crate::intelligence) fn result_value_help(&self, ty:&str, name:&str)->String {
        if !ty.starts_with("Result<") {return String::new();}
        let validation=if ty.contains("ValidationError") {
            "\n\nValidationError contains `issues: SchemaIssue[]` and `truncated: boolean`. Each issue includes `code`, `message`, `path`, `pathTruncated`, `expected`, and `observed`. Path segments identify a `field` or an `index`."
        } else {""};
        format!("\n\nA Result has two alternatives: `Ok(value)` holds the success value; `Err(error)` holds the failure. Match before accessing either payload; it is not a record with `.value` and `.error` properties.\n\n```tsrust\nmatch ({name}) {{\n  Ok(value) => {{ /* use the success value */ }},\n  Err(error) => {{ /* handle the error */ }}\n}}\n```{validation}")
    }
}
