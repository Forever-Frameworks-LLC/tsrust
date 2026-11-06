//! Owned pull environments reuse ordinary callback ownership, errors and specialization.
pub(crate) const SOURCE:&str=include_str!("source.tsr");
pub(crate) const PREFIX:&str="@stdlib:iterator#";
pub(crate) const METHODS:&[&str]=&["next","map","filter","take","drop","flatMap","reduce","forEach","some","every","find","toArray"];
use crate::{Mode,native_bindings::{NativeBinding,NativeBackend,NativeParameter,NativeType}};
const fn intrinsic(name:&'static str,access:Mode)->NativeBinding{NativeBinding{module:"std:iterator-runtime",name,returned_borrow:None,parameters:match access{Mode::Consume=>CURSOR_PARAMETERS,_=>NEXT_PARAMETERS},result:NativeType::ObservedValue,backend:NativeBackend::Bundled{symbol:"iterator_intrinsic"},features:&[]}}
pub(crate) const ARRAY_CURSOR:NativeBinding=intrinsic("arrayCursor",Mode::Consume);
pub(crate) const ARRAY_NEXT:NativeBinding=intrinsic("arrayNext",Mode::Mutate);
pub(crate) const RUNTIME:&str=include_str!("runtime.rs");

const CURSOR_PARAMETERS:&[NativeParameter]=&[NativeParameter{name:"value",ty:NativeType::ObservedValue,access:Mode::Consume,optional:false,rest:false}];
const NEXT_PARAMETERS:&[NativeParameter]=&[NativeParameter{name:"value",ty:NativeType::ObservedValue,access:Mode::Mutate,optional:false,rest:false}];
