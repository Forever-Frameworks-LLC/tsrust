//! Nominal access contracts for owned records and borrowed field views.
use crate::{scoped_record_places::Layout, scoped_record_places::Leaf};

pub(crate) struct Types<'a> {
    pub(crate) read_trait: &'a str,
    pub(crate) mutate_trait: &'a str,
    pub(crate) owned: &'a str,
    pub(crate) shared: &'a str,
    pub(crate) exclusive: &'a str,
    pub(crate) parameters: &'a [String],
}

pub(crate) fn declarations(layout: &Layout, types: Types<'_>, native: impl Fn(&Leaf) -> String) -> String {
    let parameters=types.parameters;
    declarations_with_owned(layout,types,&native,OwnedParameters{parameters,read_bounds:&[],mutate_bounds:&[]},|leaf| {
        let place=leaf.path.iter().map(|index|format!(".field{index}")).collect::<String>();
        OwnedAccess{ty:native(leaf),read:format!("&self{place}"),mutate:format!("&mut self{place}")}
    })
}

pub(crate) struct OwnedParameters<'a> {
    pub(crate) parameters: &'a [String],
    pub(crate) read_bounds: &'a [String],
    pub(crate) mutate_bounds: &'a [String],
}
pub(crate) struct OwnedAccess {
    pub(crate) ty: String,
    pub(crate) read: String,
    pub(crate) mutate: String,
}
pub(crate) fn declarations_with_owned(layout: &Layout, types: Types<'_>, native: impl Fn(&Leaf) -> String, owned: OwnedParameters<'_>, access: impl Fn(&Leaf) -> OwnedAccess) -> String {
    use std::fmt::Write;
    let mut output = String::new();
    let fields = layout.leaves.iter().map(|leaf| {
        let associated = leaf.path.iter().map(|index| format!("Field{index}")).collect::<String>();
        let method = leaf.path.iter().map(|index| format!("field{index}")).collect::<Vec<_>>().join("_");
        let place = leaf.path.iter().map(|index| format!(".field{index}")).collect::<String>();
        (associated, method, place, native(leaf), access(leaf))
    }).collect::<Vec<_>>();
    writeln!(output, "trait {} {{", types.read_trait).expect("string write");
    for (associated, method, _, _, _) in &fields {
        writeln!(output, "type {associated}: ?Sized; fn tsr_{method}(&self)->&Self::{associated};").expect("string write");
    }
    writeln!(output, "}} trait {}: {} {{", types.mutate_trait, types.read_trait).expect("string write");
    for (associated, method, _, _, _) in &fields {
        writeln!(output, "fn tsr_{method}_mut(&mut self)->&mut Self::{associated};").expect("string write");
    }
    output.push_str("}\n");
    let owned_parameters = if owned.parameters.is_empty() { String::new() } else { format!("<{}>",owned.parameters.join(", ")) };
    let view_parameters = format!("<{}>",std::iter::once("'a".to_owned()).chain(types.parameters.iter().cloned()).collect::<Vec<_>>().join(", "));
    for (carrier, parameters, borrowed, mutable) in [(types.owned,&owned_parameters,false,true),(types.shared,&view_parameters,true,false),(types.exclusive,&view_parameters,true,true)] {
        let bounds=if borrowed || owned.read_bounds.is_empty(){String::new()}else{format!(" where {}",owned.read_bounds.join(", "))};
        writeln!(output, "impl{parameters} {} for {carrier}{bounds} {{",types.read_trait).expect("string write");
        for (associated, method, place, ty, owner) in &fields {
            let (ty,read)=if borrowed{(ty.clone(),format!("&*self{place}"))}else{(owner.ty.clone(),owner.read.clone())};
            writeln!(output, "type {associated}={ty}; fn tsr_{method}(&self)->&Self::{associated} {{ {read} }}").expect("string write");
        }
        output.push_str("}\n");
        if mutable {
            let bounds=if borrowed || owned.mutate_bounds.is_empty(){String::new()}else{format!(" where {}",owned.mutate_bounds.join(", "))};
            writeln!(output, "impl{parameters} {} for {carrier}{bounds} {{",types.mutate_trait).expect("string write");
            for (associated, method, place, _, owner) in &fields {
                let mutate=if borrowed{format!("&mut *self{place}")}else{owner.mutate.clone()};
                writeln!(output, "fn tsr_{method}_mut(&mut self)->&mut Self::{associated} {{ {mutate} }}").expect("string write");
            }
            output.push_str("}\n");
        }
    }
    output
}
