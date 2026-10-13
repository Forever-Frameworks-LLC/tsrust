//! Pipeline identity retains callback input separately from validated output.
use crate::syntax::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Operation {
    /// Inspect a typed borrowed value before any consuming stage.
    Refine,
    /// Consume the receiving schema's validated output.
    Transform,
    /// Consume raw input before the receiving schema validates the result.
    Coerce,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DiagnosticSegment { Field(String), Index(usize) }

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Stage {
    /// Exact receiving schema record, not merely its output type.
    pub source: usize,
    pub diagnostic_path: Vec<DiagnosticSegment>,
    pub custom_message: bool,
    pub operation: Operation,
    /// Unique readonly callable storage; its argument can still be owned.
    pub callback: usize,
    pub callback_input: Type,
    pub callback_output: Type,
    /// Accepted owned input of the whole chain.
    pub input: Type,
    /// Successful output of the whole chain.
    pub output: Type,
    pub json_input: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArrayPlan {
    pub element: usize,
    pub input: Type,
    pub output: Type,
    pub json_input: bool,
}

/// Collection shape is distinct from its output: a record decoder and a native
/// Map validator can produce the same Map without accepting the same inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CollectionKind { Record, Map, Set }

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CollectionPlan {
    pub kind: CollectionKind,
    /// Native Map key schema; record keys remain untransformed strings.
    pub key: Option<usize>,
    pub element: usize,
    pub input: Type,
    pub output: Type,
    pub json_input: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ObjectPlan {
    pub shape: usize,
    /// Exact field schema identities in shape declaration order.
    pub fields: Vec<usize>,
    pub input: Type,
    pub output: Type,
    pub json_input: bool,
    pub policy: ObjectPolicy,
    /// The typed fields before a retaining wrapper is applied.
    pub value: Type,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ObjectPolicy { Reject, Strip, Retain }

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UnionPlan {
    pub branches: Vec<usize>,
    pub selector: Option<super::plans::Selector>,
    pub input: Type,
    pub output: Type,
    pub json_input: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Wrapper {
    pub source: usize,
    pub kind: WrapperKind,
    pub input: Type,
    pub output: Type,
    pub json_input: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WrapperKind { Presence(Type), Default(usize), Message, Constraint(usize) }

/// Compiler-only identity of a definition and its uniquely owned body plan.
/// Native storage must borrow this root when a reference edge is validated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RecursivePlan {
    pub body: usize,
    pub input: Type,
    pub output: Type,
    pub json_input: bool,
}

/// A self edge contains no runtime schema, allocation, or shared owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RecursiveReference {
    pub root: usize,
    pub input: Type,
    pub output: Type,
    pub json_input: bool,
}

/// A checked addIssue call carries no dynamic path or input value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IssueCall {
    pub context: usize,
    pub output: Type,
    pub path: Vec<DiagnosticSegment>,
}

#[derive(Default)]
pub(crate) struct Pipelines {
    issue_calls: Vec<IssueCall>,
    // Records preserve exact chain identity through variables and record fields.
    // Never index this registry by output: unrelated input contracts can produce
    // the same output, and must not become interchangeable.
    recursive: std::collections::BTreeMap<usize, RecursivePlan>,
    references: std::collections::BTreeMap<usize, RecursiveReference>,
    wrappers: std::collections::BTreeMap<usize, Wrapper>,
    stages: std::collections::BTreeMap<usize, Stage>,
    arrays: std::collections::BTreeMap<usize, ArrayPlan>,
    collections: std::collections::BTreeMap<usize, CollectionPlan>,
    objects: std::collections::BTreeMap<usize, ObjectPlan>,
    unions: std::collections::BTreeMap<usize, UnionPlan>,
}

impl Pipelines {
    pub(crate) fn issue_call(&self,id:usize)->&IssueCall { &self.issue_calls[id] }
    pub(crate) fn intern_issue_call(&mut self,call:IssueCall)->usize {
        if let Some(id)=self.issue_calls.iter().position(|existing|*existing==call){return id;}
        let id=self.issue_calls.len();self.issue_calls.push(call);id
    }

    pub(crate) fn has_recursive(&self)->bool { !self.recursive.is_empty() }
    pub(crate) fn has_collections(&self)->bool { !self.collections.is_empty() }
    pub(crate) fn has_messages(&self)->bool { self.wrappers.values().any(|plan| matches!(plan.kind,WrapperKind::Message|WrapperKind::Constraint(_))) }
    pub(crate) fn types(&self, record: usize) -> Option<(Type, Type, bool)> {
        self.recursive(record).map(|plan|(plan.input,plan.output,plan.json_input))
            .or_else(||self.reference(record).map(|plan|(plan.input,plan.output,plan.json_input)))
            .or_else(||self.wrapper(record).map(|plan|(plan.input,plan.output,plan.json_input))).or_else(||self.get(record).map(|stage| (stage.input, stage.output, stage.json_input)))
            .or_else(|| self.collection(record).map(|plan| (plan.input, plan.output, plan.json_input)))
            .or_else(|| self.array(record).map(|plan| (plan.input, plan.output, plan.json_input)))
            .or_else(|| self.object(record).map(|plan| (plan.input, plan.output, plan.json_input)))
            .or_else(|| self.union(record).map(|plan| (plan.input, plan.output, plan.json_input)))
    }
    /// Text decoding is separate from accepting an already built JsonValue.
    /// Explicit coercion supplies a native child input after decoding its own raw
    /// input shape, so a Map/Set underneath that boundary needs no wire encoding.
    pub(crate) fn accepts_text(&self,record:usize)->bool {
        let mut pending=vec![record];
        let mut seen=std::collections::HashSet::new();
        while let Some(record)=pending.pop() {
            if !seen.insert(record) {continue;}
            if let Some(stage)=self.get(record) {
                if stage.operation!=Operation::Coerce {pending.push(stage.source);}
            } else if let Some(plan)=self.collection(record) {
                if plan.kind!=CollectionKind::Record {return false;}
                pending.push(plan.element);
            } else {pending.extend(self.children(record));}
        }
        true
    }
    pub(crate) fn recursive(&self,record:usize)->Option<&RecursivePlan> {self.recursive.get(&record)}
    pub(crate) fn reference(&self,record:usize)->Option<&RecursiveReference> {self.references.get(&record)}
    pub(crate) fn insert_recursive(&mut self,record:usize,plan:RecursivePlan) {
        assert!(self.types(record).is_none());
        self.recursive.insert(record,plan);
    }
    pub(crate) fn insert_reference(&mut self,record:usize,plan:RecursiveReference) {
        assert!(self.types(record).is_none());
        self.references.insert(record,plan);
    }
    /// Check the resolved plan, including schemas held in local variables.
    /// Source syntax alone cannot establish productivity after name resolution.
    pub(crate) fn validate_recursive_body(&self,body:usize,root:usize)->Result<(),super::recursive::Invalid> {
        use super::recursive::{Definition,Shape};
        let mut graph=Definition::default();
        let mut nodes=std::collections::HashMap::new();
        let mut pending=vec![(body,false)];
        while let Some((record,ready))=pending.pop() {
            if nodes.contains_key(&record) {continue;}
            let terminal=self.reference(record).is_some()||self.recursive(record).is_some();
            let children=if terminal {Vec::new()}else{self.children(record)};
            if !ready && !children.is_empty() {
                pending.push((record,true));
                pending.extend(children.into_iter().rev().map(|child|(child,false)));
                continue;
            }
            let child=|id|*nodes.get(&id).expect("checked child schema graph");
            let shape=if let Some(reference)=self.reference(record) {
                if reference.root==root {Shape::Reference}else{Shape::Leaf}
            } else if self.recursive(record).is_some() {Shape::Leaf}
            else if let Some(plan)=self.array(record) {Shape::Array(child(plan.element))}
            else if self.collection(record).is_some() {
                // All supported collections can terminate empty.
                let elements=graph.push(Shape::Object(children.iter().copied().map(child).collect()));
                Shape::Array(elements)
            } else if let Some(plan)=self.object(record) {Shape::Object(plan.fields.iter().copied().map(child).collect())}
            else if let Some(plan)=self.union(record) {Shape::Choice(plan.branches.iter().copied().map(child).collect())}
            else if let Some(plan)=self.wrapper(record) {
                let source=child(plan.source);
                if matches!(plan.kind,WrapperKind::Presence(_)) {
                    let absent=graph.push(Shape::Leaf);
                    Shape::Choice(vec![source,absent])
                }else{Shape::Adapter(source)}
            } else if let Some(stage)=self.get(record) {Shape::Adapter(child(stage.source))}
            else {Shape::Leaf};
            nodes.insert(record,graph.push(shape));
        }
        graph.validate(nodes[&body])
    }
    /// Roots that a generated validator must receive as borrowed parameters.
    /// A definition supplies its own root to its body. Reference markers have
    /// no stored root, so callers must forward the corresponding borrow.
    pub(crate) fn borrowed_roots(&self,record:usize)->Vec<usize> {
        let mut pending=vec![record];
        let mut seen=std::collections::HashSet::new();
        let mut required=std::collections::BTreeSet::new();
        let mut provided=std::collections::BTreeSet::new();
        while let Some(record)=pending.pop() {
            if !seen.insert(record) {continue;}
            if let Some(reference)=self.reference(record) {
                required.insert(reference.root);
                // Following this edge would incorrectly turn a borrowed root
                // into a definition supplied by this subtree.
            } else {
                if self.recursive(record).is_some() {provided.insert(record);}
                pending.extend(self.children(record));
            }
        }
        required.difference(&provided).copied().collect()
    }
    /// Analysis follows self edges; this is not a native storage layout walk.
    fn children(&self,record:usize)->Vec<usize> {
        if let Some(plan)=self.recursive(record) {vec![plan.body]}
        else if let Some(plan)=self.reference(record) {vec![plan.root]}
        else if let Some(plan)=self.wrapper(record) {vec![plan.source]}
        else if let Some(plan)=self.get(record) {vec![plan.source]}
        else if let Some(plan)=self.collection(record) {plan.key.into_iter().chain(std::iter::once(plan.element)).collect()}
        else if let Some(plan)=self.array(record) {vec![plan.element]}
        else if let Some(plan)=self.object(record) {plan.fields.clone()}
        else if let Some(plan)=self.union(record) {plan.branches.clone()}
        else {Vec::new()}
    }
    pub(crate) fn collection(&self, record: usize) -> Option<&CollectionPlan> { self.collections.get(&record) }
    pub(crate) fn collection_record(&self, plan: &CollectionPlan) -> Option<usize> {
        self.collections.iter().find_map(|(record,existing)|(existing==plan).then_some(*record))
    }
    pub(crate) fn insert_collection(&mut self, record: usize, plan: CollectionPlan) {
        assert!(self.types(record).is_none());
        assert_eq!(plan.key.is_some(),plan.kind==CollectionKind::Map);
        assert!(!plan.json_input || plan.kind==CollectionKind::Record);
        self.collections.insert(record,plan);
    }
    pub(crate) fn wrapper(&self, record: usize) -> Option<&Wrapper> { self.wrappers.get(&record) }
    pub(crate) fn wrapper_record(&self, plan: &Wrapper) -> Option<usize> {
        self.wrappers.iter().find_map(|(id, existing)|(existing==plan).then_some(*id))
    }
    pub(crate) fn insert_wrapper(&mut self, record: usize, plan: Wrapper) {
        assert!(self.types(record).is_none());
        self.wrappers.insert(record,plan);
    }
    pub(crate) fn union(&self, record: usize) -> Option<&UnionPlan> { self.unions.get(&record) }
    pub(crate) fn union_record(&self, plan: &UnionPlan) -> Option<usize> {
        self.unions.iter().find_map(|(record, existing)| (existing == plan).then_some(*record))
    }
    pub(crate) fn insert_union(&mut self, record: usize, plan: UnionPlan) {
        assert!(!self.stages.contains_key(&record) && !self.arrays.contains_key(&record) && !self.objects.contains_key(&record));
        if let Some(previous) = self.unions.get(&record) { assert_eq!(previous, &plan); }
        else { self.unions.insert(record, plan); }
    }
    pub(crate) fn object(&self, record: usize) -> Option<&ObjectPlan> { self.objects.get(&record) }
    pub(crate) fn object_record(&self, plan: &ObjectPlan) -> Option<usize> {
        self.objects.iter().find_map(|(record, existing)| (existing == plan).then_some(*record))
    }
    pub(crate) fn insert_object(&mut self, record: usize, plan: ObjectPlan) {
        assert!(!self.stages.contains_key(&record) && !self.arrays.contains_key(&record));
        if let Some(previous) = self.objects.get(&record) { assert_eq!(previous, &plan); }
        else { self.objects.insert(record, plan); }
    }
    pub(crate) fn array(&self, record: usize) -> Option<&ArrayPlan> { self.arrays.get(&record) }
    #[cfg(test)]
    pub(crate) fn has_coercion(&self, record: usize) -> bool {
        let mut pending=vec![record];
        let mut seen=std::collections::HashSet::new();
        while let Some(record)=pending.pop() {
            if !seen.insert(record) {continue;}
            if self.get(record).is_some_and(|stage|stage.operation==Operation::Coerce) {return true;}
            pending.extend(self.children(record));
        }
        false
    }
    pub(crate) fn array_record(&self, plan: &ArrayPlan) -> Option<usize> {
        self.arrays.iter().find_map(|(record, existing)| (existing == plan).then_some(*record))
    }
    pub(crate) fn insert_array(&mut self, record: usize, plan: ArrayPlan) {
        assert!(!self.stages.contains_key(&record));
        if let Some(previous) = self.arrays.get(&record) { assert_eq!(previous, &plan); }
        else { self.arrays.insert(record, plan); }
    }
    /// Enumerate semantic dependencies, including self edges. This does not
    /// describe native ownership: reference edges never store their root.
    pub(crate) fn dependencies(&self, root: usize) -> (Vec<usize>, Vec<usize>) {
        let mut pending = vec![root];
        let mut seen = std::collections::HashSet::new();
        let mut callbacks = Vec::new();
        let mut leaves = Vec::new();
        while let Some(record) = pending.pop() {
            if !seen.insert(record) { continue; }
            if let Some(plan)=self.recursive(record) {pending.push(plan.body);}
            else if let Some(plan)=self.reference(record) {pending.push(plan.root);}
            else if let Some(plan)=self.wrapper(record) {
                pending.push(plan.source);
                if let WrapperKind::Default(family)=plan.kind { callbacks.push(family); }
            }
            else if let Some(stage) = self.get(record) { callbacks.push(stage.callback); pending.push(stage.source); }
            else if let Some(plan) = self.collection(record) { pending.extend(plan.key); pending.push(plan.element); }
            else if let Some(plan) = self.array(record) { pending.push(plan.element); }
            else if let Some(plan) = self.object(record) { pending.extend(plan.fields.iter().copied()); }
            else if let Some(plan) = self.union(record) { pending.extend(plan.branches.iter().copied()); }
            else { leaves.push(record); }
        }
        (callbacks, leaves)
    }
    pub(crate) fn coercion_inputs(&self,root:usize)->Vec<Type>{
        let (callbacks,_)=self.dependencies(root);
        self.stages.values().filter(|stage|stage.operation==Operation::Coerce&&callbacks.contains(&stage.callback)).map(|stage|stage.callback_input).collect()
    }
    pub(crate) fn callbacks(&self) -> impl Iterator<Item = usize> + '_ {
        self.stages.values().map(|stage| stage.callback).chain(self.wrappers.values().filter_map(|plan|
            if let WrapperKind::Default(family)=plan.kind {Some(family)}else{None}))
    }

    pub(crate) fn get(&self, record: usize) -> Option<&Stage> {
        self.stages.get(&record)
    }

    pub(crate) fn record_for(&self, stage: &Stage) -> Option<usize> {
        self.stages.iter().find_map(|(record, existing)| (existing == stage).then_some(*record))
    }

    pub(crate) fn insert(&mut self, record: usize, stage: Stage) {
        if let Some(previous) = self.stages.get(&record) {
            assert_eq!(previous, &stage, "schema pipeline record changed contract");
            return;
        }
        self.stages.insert(record, stage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage(input: Type) -> Stage {
        Stage {
            source: 2, diagnostic_path: Vec::new(), custom_message: false, operation: Operation::Coerce, callback: 4,
            callback_input: input, callback_output: Type::Int,
            input, output: Type::Int, json_input: false,
        }
    }

    #[test]
    fn resolved_recursive_plans_require_progress_and_finite_values() {
        let mut plans=Pipelines::default();
        plans.insert_reference(11,RecursiveReference{root:10,input:Type::Int,output:Type::Int,json_input:true});
        plans.insert_array(12,ArrayPlan{element:11,input:Type::Int,output:Type::Int,json_input:true});
        assert!(plans.validate_recursive_body(12,10).is_ok());
        assert!(plans.validate_recursive_body(11,10).is_err());
        plans.insert_object(13,ObjectPlan{shape:0,fields:vec![11],input:Type::Int,output:Type::Int,json_input:true,policy:ObjectPolicy::Reject,value:Type::Int});
        assert!(plans.validate_recursive_body(13,10).is_err());
        plans.insert_wrapper(14,Wrapper{source:11,kind:WrapperKind::Presence(Type::Null),input:Type::Int,output:Type::Int,json_input:true});
        plans.insert_object(15,ObjectPlan{shape:0,fields:vec![14],input:Type::Int,output:Type::Int,json_input:true,policy:ObjectPolicy::Reject,value:Type::Int});
        assert!(plans.validate_recursive_body(15,10).is_ok());
        // A nullable direct self still makes no structural progress.
        assert!(plans.validate_recursive_body(14,10).is_err());
    }

    #[test]
    fn recursive_validator_parameters_follow_storage_not_reference_edges() {
        let mut plans=Pipelines::default();
        plans.insert_reference(11,RecursiveReference{root:10,input:Type::String,output:Type::Int,json_input:true});
        plans.insert_array(12,ArrayPlan{element:11,input:Type::String,output:Type::Int,json_input:true});
        plans.insert_recursive(10,RecursivePlan{body:12,input:Type::String,output:Type::Int,json_input:true});
        assert_eq!(plans.borrowed_roots(11),vec![10]);
        assert_eq!(plans.borrowed_roots(12),vec![10]);
        assert!(plans.borrowed_roots(10).is_empty());
        // An inner definition supplies itself but still borrows an outer root.
        plans.insert_reference(21,RecursiveReference{root:20,input:Type::String,output:Type::Int,json_input:true});
        plans.insert_union(22,UnionPlan{branches:vec![11,21,11],selector:None,input:Type::String,output:Type::Int,json_input:true});
        plans.insert_recursive(20,RecursivePlan{body:22,input:Type::String,output:Type::Int,json_input:true});
        assert_eq!(plans.borrowed_roots(22),vec![10,20]);
        assert_eq!(plans.borrowed_roots(20),vec![10]);
        assert_eq!(plans.types(10),Some((Type::String,Type::Int,true)));
    }

    #[test]
    fn recursive_metadata_visits_each_plan_without_owning_its_root() {
        let mut plans=Pipelines::default();
        plans.insert_reference(11,RecursiveReference{root:10,input:Type::Int,output:Type::Int,json_input:true});
        plans.insert_array(12,ArrayPlan{element:11,input:Type::Int,output:Type::Int,json_input:true});
        plans.insert(13,Stage{operation:Operation::Refine,..stage(Type::Int)});
        plans.insert_union(14,UnionPlan{branches:vec![12,13],selector:None,input:Type::Int,output:Type::Int,json_input:true});
        plans.insert_recursive(10,RecursivePlan{body:14,input:Type::Int,output:Type::Int,json_input:true});
        assert_eq!(plans.types(11),plans.types(10));
        assert!(plans.accepts_text(10));
        assert!(!plans.has_coercion(10));
        assert_eq!(plans.dependencies(10),(vec![4],vec![2]));
        assert_eq!(plans.dependencies(11),(vec![4],vec![2]));
    }

    #[test]
    fn recursive_text_acceptance_preserves_native_collection_boundaries() {
        let mut plans=Pipelines::default();
        plans.insert_reference(11,RecursiveReference{root:10,input:Type::Int,output:Type::Int,json_input:false});
        plans.insert_collection(12,CollectionPlan{kind:CollectionKind::Set,key:None,element:11,input:Type::Int,output:Type::Int,json_input:false});
        plans.insert_recursive(10,RecursivePlan{body:12,input:Type::Int,output:Type::Int,json_input:false});
        assert!(!plans.accepts_text(10));
        assert!(!plans.has_coercion(10));
        assert_eq!(plans.dependencies(10),(vec![],vec![]));
        // A coercion supplies its own raw input domain before native validation.
        plans.insert(13,Stage{source:10,..stage(Type::String)});
        assert!(plans.accepts_text(13));
        assert!(plans.has_coercion(13));
        assert_eq!(plans.dependencies(13),(vec![4],vec![]));
    }

    #[test]
    fn collections_preserve_input_domain_and_child_callback_dependencies() {
        let mut plans=Pipelines::default();
        plans.insert(10,stage(Type::String));
        let record=CollectionPlan{kind:CollectionKind::Record,key:None,element:10,input:Type::String,output:Type::Int,json_input:true};
        plans.insert_collection(20,record.clone());
        let map=CollectionPlan{kind:CollectionKind::Map,key:Some(3),json_input:false,..record.clone()};
        plans.insert_collection(21,map.clone());
        assert_eq!(plans.collection_record(&record),Some(20));
        assert_eq!(plans.collection_record(&map),Some(21));
        assert_eq!(plans.types(20),Some((Type::String,Type::Int,true)));
        assert_eq!(plans.types(21),Some((Type::String,Type::Int,false)));
        let (callbacks,leaves)=plans.dependencies(21);
        assert_eq!(callbacks,vec![4]);
        assert!(leaves.contains(&2)&&leaves.contains(&3));
        assert!(plans.has_coercion(20));
        assert!(plans.has_coercion(21));
    }

    #[test]
    fn equal_outputs_do_not_merge_distinct_input_contracts() {
        let mut plans = Pipelines::default();
        let text = stage(Type::String);
        let boolean = stage(Type::Boolean);
        plans.insert(10, text.clone());
        plans.insert(11, boolean.clone());
        assert_eq!(plans.record_for(&text), Some(10));
        assert_eq!(plans.record_for(&boolean), Some(11));
        assert_eq!(plans.get(10).unwrap().input, Type::String);
        assert_eq!(plans.get(11).unwrap().input, Type::Boolean);
        let mut other_receiver = text.clone();
        other_receiver.source = 3;
        assert_eq!(plans.record_for(&other_receiver), None);
        let mut other_callback = text;
        other_callback.callback = 5;
        assert_eq!(plans.record_for(&other_callback), None);
    }

    #[test]
    fn operation_order_is_part_of_chain_identity() {
        let mut plans = Pipelines::default();
        let mut conversion = stage(Type::Int);
        conversion.operation = Operation::Transform;
        plans.insert(10, conversion.clone());
        let mut refinement = conversion.clone();
        refinement.operation = Operation::Refine;
        refinement.callback_output = Type::Boolean;
        plans.insert(11, refinement.clone());
        assert_eq!(plans.record_for(&conversion), Some(10));
        assert_eq!(plans.record_for(&refinement), Some(11));
        refinement.source = 10;
        assert_eq!(plans.record_for(&refinement), None);
    }

    #[test]
    fn nested_array_dependencies_retain_callback_and_leaf_schema_identity() {
        let mut plans = Pipelines::default();
        plans.insert(10, stage(Type::String));
        let array_type = |ty| Type::Array(crate::syntax::ArrayElement::from_value_type(ty).unwrap());
        let first = ArrayPlan { element: 10, input: array_type(Type::String), output: array_type(Type::Int), json_input: false };
        plans.insert_array(11, first.clone());
        plans.insert_array(12, ArrayPlan { element: 11, input: array_type(first.input), output: array_type(first.output), json_input: false });
        assert_eq!(plans.dependencies(12), (vec![4], vec![2]));
        assert!(plans.has_coercion(12));
        assert_eq!(plans.types(12), Some((array_type(first.input), array_type(first.output), false)));
    }
}
