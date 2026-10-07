//! Nominal access contracts for owned records and borrowed field views.
use crate::{scoped_record_places::Layout, syntax::Type};

pub(crate) struct Types<'a> {
    pub(crate) read_trait: &'a str,
    pub(crate) mutate_trait: &'a str,
    pub(crate) owned: &'a str,
    pub(crate) shared: &'a str,
    pub(crate) exclusive: &'a str,
    pub(crate) parameters: &'a [String],
}

pub(crate) fn declarations(layout: &Layout, types: Types<'_>, native: impl Fn(Type) -> String) -> String {
    use std::fmt::Write;
    let mut output = String::new();
    let fields = layout.leaves.iter().map(|leaf| {
        let associated = leaf.path.iter().map(|index| format!("Field{index}")).collect::<String>();
        let method = leaf.path.iter().map(|index| format!("field{index}")).collect::<Vec<_>>().join("_");
        let place = leaf.path.iter().map(|index| format!(".field{index}")).collect::<String>();
        (associated, method, place, native(leaf.ty))
    }).collect::<Vec<_>>();
    writeln!(output, "trait {} {{", types.read_trait).expect("string write");
    for (associated, method, _, _) in &fields {
        writeln!(output, "type {associated}: ?Sized; fn tsr_{method}(&self)->&Self::{associated};").expect("string write");
    }
    writeln!(output, "}} trait {}: {} {{", types.mutate_trait, types.read_trait).expect("string write");
    for (associated, method, _, _) in &fields {
        writeln!(output, "fn tsr_{method}_mut(&mut self)->&mut Self::{associated};").expect("string write");
    }
    output.push_str("}\n");
    let owned_parameters = if types.parameters.is_empty() { String::new() } else { format!("<{}>",types.parameters.join(", ")) };
    let view_parameters = format!("<{}>",std::iter::once("'a".to_owned()).chain(types.parameters.iter().cloned()).collect::<Vec<_>>().join(", "));
    for (carrier, parameters, borrowed, mutable) in [(types.owned,&owned_parameters,false,true),(types.shared,&view_parameters,true,false),(types.exclusive,&view_parameters,true,true)] {
        writeln!(output, "impl{parameters} {} for {carrier} {{",types.read_trait).expect("string write");
        for (associated, method, place, ty) in &fields {
            writeln!(output, "type {associated}={ty}; fn tsr_{method}(&self)->&Self::{associated} {{ &{}self{place} }}", if borrowed { "*" } else { "" }).expect("string write");
        }
        output.push_str("}\n");
        if mutable {
            writeln!(output, "impl{parameters} {} for {carrier} {{",types.mutate_trait).expect("string write");
            for (associated, method, place, _) in &fields {
                writeln!(output, "fn tsr_{method}_mut(&mut self)->&mut Self::{associated} {{ &mut {}self{place} }}",if borrowed { "*" } else { "" }).expect("string write");
            }
            output.push_str("}\n");
        }
    }
    output
}
