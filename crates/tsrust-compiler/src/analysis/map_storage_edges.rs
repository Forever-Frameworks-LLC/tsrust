//! Checked storage edges include Map leaves inside owning record moves.
use super::*;

pub(super) struct Edge {
    pub(super) source:usize,
    pub(super) source_path:Vec<String>,
    pub(super) destination:usize,
    pub(super) destination_path:Vec<String>,
    pub(super) span:Span,
}

impl Analyzer {
    pub(super) fn map_storage_edges(&self,writes:&[record_storage_writes::Write<'_>],roots:&HashMap<usize,Type>)->Vec<Edge> {
        let mut edges=Vec::new();
        for write in writes {
            let mut pending=vec![(write.path.clone(),write.value)];
            while let Some((destination_path,value))=pending.pop() {
                match &value.kind {
                    ExprKind::Unary{op:UnaryOp::RecordCarrier,value,..}=>pending.push((destination_path,value)),
                    ExprKind::Record(fields)=>for (name,value) in fields {let mut path=destination_path.clone();path.push(name.text.clone());pending.push((path,value));},
                    ExprKind::Array(values)=>for value in values {pending.push((destination_path.clone(),value));},
                    _=>{
                        let Some((source,source_path))=Self::scoped_array_storage_path(value) else {continue;};
                        let Some(ty)=roots.get(&source).and_then(|ty|self.scoped_array_field_type(*ty,&source_path)) else {continue;};
                        let mut types=vec![(ty,source_path,destination_path)];
                        while let Some((ty,source_path,destination_path))=types.pop() {
                            match ty {
                                Type::Map(_)=>edges.push(Edge{source,source_path,destination:write.binding,destination_path,span:value.span}),
                                Type::Record(record)=>for field in &self.program.records[record].fields {
                                    let mut source=source_path.clone();source.push(field.name.text.clone());
                                    let mut destination=destination_path.clone();destination.push(field.name.text.clone());
                                    if let Some(ty)=field.ty {types.push((ty,source,destination));}
                                },
                                _=>{},
                            }
                        }
                    },
                }
            }
        }
        edges
    }
}
