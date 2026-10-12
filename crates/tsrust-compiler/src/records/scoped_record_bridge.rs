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

#[cfg(test)]
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
#[cfg(test)]
pub(crate) fn declarations_with_owned(layout: &Layout, types: Types<'_>, native: impl Fn(&Leaf) -> String, owned: OwnedParameters<'_>, access: impl Fn(&Leaf) -> OwnedAccess) -> String {
    declarations_with_transport(layout, types, native, owned, access, None)
}

/// Whole-record transport splits concrete storage once before generic access.
pub(crate) struct OrderAccess {
    pub(crate) member:String,
    pub(crate) place:String,
    pub(crate) width:usize,
    pub(crate) owned:String,
}
pub(crate) struct OwnedTransport {
    pub(crate) orders:Vec<OrderAccess>,
    pub(crate) read: String,
    pub(crate) mutate: String,
}
pub(crate) fn declarations_with_transport(layout: &Layout, types: Types<'_>, native: impl Fn(&Leaf) -> String, owned: OwnedParameters<'_>, access: impl Fn(&Leaf) -> OwnedAccess, transport: Option<OwnedTransport>) -> String {
    use std::fmt::Write;
    let mut output = String::new();
    let fields = layout.leaves.iter().map(|leaf| {
        let associated = leaf.path.iter().map(|index| format!("Field{index}")).collect::<String>();
        let method = leaf.path.iter().map(|index| format!("field{index}")).collect::<Vec<_>>().join("_");
        let place = leaf.path.iter().map(|index| format!(".field{index}")).collect::<String>();
        (associated, method, place, native(leaf), access(leaf))
    }).collect::<Vec<_>>();
    let read_fields=format!("{}Fields",types.read_trait);
    let mutate_fields=format!("{}Fields",types.mutate_trait);
    let transport_type=|name:&str|format!("{name}<'_,{}>",fields.iter().map(|(associated,_,_,_,_)|format!("Self::{associated}")).collect::<Vec<_>>().join(","));
    if transport.is_some() {
        let parameters=fields.iter().enumerate().map(|(index,_)|format!("F{index}:?Sized")).collect::<Vec<_>>().join(",");
        for (name,mutable) in [(&read_fields,false),(&mutate_fields,true)] {
            let modifier=if mutable{"mut "}else{""};
            let mut members=fields.iter().enumerate().map(|(index,(_,method,_,_,_))|format!("{method}:&'a {modifier}F{index}")).collect::<Vec<_>>().join(",");
            for order in &transport.as_ref().expect("transport").orders {let _=write!(members,",{}:&'a {modifier}[usize;{}]",order.member,order.width);}
            writeln!(output,"struct {name}<'a,{parameters}>{{{members}}}").expect("string write");
        }
    }
    writeln!(output, "trait {} {{", types.read_trait).expect("string write");
    for (associated, method, _, _, _) in &fields {
        writeln!(output, "type {associated}: ?Sized; fn tsr_{method}(&self)->&Self::{associated};").expect("string write");
    }
    if let Some(transport)=&transport{for order in &transport.orders{writeln!(output,"fn tsr_{}(&self)->&[usize;{}];",order.member,order.width).expect("string write");}}
    if transport.is_some(){writeln!(output,"fn tsr_fields(&self)->{};",transport_type(&read_fields)).expect("string write");}
    writeln!(output, "}} trait {}: {} {{", types.mutate_trait, types.read_trait).expect("string write");
    for (associated, method, _, _, _) in &fields {
        writeln!(output, "fn tsr_{method}_mut(&mut self)->&mut Self::{associated};").expect("string write");
    }
    if transport.is_some(){writeln!(output,"fn tsr_fields_mut(&mut self)->{};",transport_type(&mutate_fields)).expect("string write");}
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
        if let Some(transport)=&transport {
            for order in &transport.orders {let read=if borrowed{format!("&*self{}",order.place)}else{order.owned.clone()};writeln!(output,"fn tsr_{}(&self)->&[usize;{}]{{{read}}}",order.member,order.width).expect("string write");}
            let body=if borrowed {
                let mut members=fields.iter().map(|(_,method,place,_,_)|format!("{method}:&*self{place}")).collect::<Vec<_>>().join(",");
                for order in &transport.orders{let _=write!(members,",{}:&*self{}",order.member,order.place);}
                format!("{read_fields}{{{members}}}")
            }else{transport.read.clone()};
            writeln!(output,"fn tsr_fields(&self)->{}{{{body}}}",transport_type(&read_fields)).expect("string write");
        }
        output.push_str("}\n");
        if mutable {
            let bounds=if borrowed || owned.mutate_bounds.is_empty(){String::new()}else{format!(" where {}",owned.mutate_bounds.join(", "))};
            writeln!(output, "impl{parameters} {} for {carrier}{bounds} {{",types.mutate_trait).expect("string write");
            for (associated, method, place, _, owner) in &fields {
                let mutate=if borrowed{format!("&mut *self{place}")}else{owner.mutate.clone()};
                writeln!(output, "fn tsr_{method}_mut(&mut self)->&mut Self::{associated} {{ {mutate} }}").expect("string write");
            }
            if let Some(transport)=&transport {
                let body=if borrowed {
                    let mut members=fields.iter().map(|(_,method,place,_,_)|format!("{method}:&mut *self{place}")).collect::<Vec<_>>().join(",");
                    for order in &transport.orders{let _=write!(members,",{}:&mut *self{}",order.member,order.place);}
                    format!("{mutate_fields}{{{members}}}")
                }else{transport.mutate.clone()};
                writeln!(output,"fn tsr_fields_mut(&mut self)->{}{{{body}}}",transport_type(&mutate_fields)).expect("string write");
            }
            output.push_str("}\n");
        }
    }
    output
}
