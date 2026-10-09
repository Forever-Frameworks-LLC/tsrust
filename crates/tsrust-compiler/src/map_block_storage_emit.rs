//! Return a labelled block's owner on both normal and break exits.
use super::*;
impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn map_block_storage_epoch(&self,output:&mut String,fun:&CheckedFunction,statement:&Stmt,declarations:&[Stmt],depth:usize,loop_label:Option<usize>)->bool {
        let epochs=fun.storage_exits.map_block_epochs(statement.span.start);
        if epochs.is_empty(){return false;}
        let StmtKind::Block(body)=&statement.kind else{return false;};
        let _scope=self.control_targets.enter(ControlTarget::Block(statement.span.start));
        let indent="    ".repeat(depth);
        let names=epochs.iter().map(|epoch|self.map_entry_epoch_place(fun,epoch)).collect::<Vec<_>>();
        let pattern=names.iter().map(|place|format!("mut {place}")).collect::<Vec<_>>().join(",");
        let _=writeln!(output,"{indent}let ({pattern},) = 'map_block_{}: {{",statement.span.start);
        self.control_block_epochs(output,fun,body,depth+1,loop_label,Some(declarations));
        for epoch in epochs {if epoch.input!=epoch.output{let _=writeln!(output,"{indent}    {}",self.map_entry_epoch_code(fun,epoch));}}
        let _=writeln!(output,"{indent}    ({},)
{indent}}};",names.join(","));
        self.exception_regions.renew_storage(&names,statement.span.end);
        true
    }
}
