//! Shared native ABI lowering for checked calls and forwarding callable bodies.
use crate::native_bindings::{NativeBackend, NativeBinding};

/// Arguments have already been emitted with each registry parameter's access mode.
pub(crate) fn call(binding: &NativeBinding, arguments: Vec<String>) -> String {
    assert!(
        binding.accepts_arity(arguments.len()),
        "checked native arity"
    );
    let NativeBackend::Bundled { symbol } = binding.backend;
    if binding.parameters.len()==1 && binding.parameters[0].rest {
        let holds=arguments.iter().enumerate().map(|(i,value)|format!("let native_argument_{i} = {value};")).collect::<Vec<_>>().join(" ");
        let values=(0..arguments.len()).map(|i|format!("native_argument_{i}.as_ref()")).collect::<Vec<_>>().join(", ");
        return format!("{{ {holds} {symbol}(&[{values}]) }}");
    }
    if binding.module == "std:bytes" && binding.name == "fromArray" {
        return format!("{symbol}(({}).as_slice())", arguments[0]);
    }
    let mut arguments = arguments.into_iter();
    let lowered = binding
        .parameters
        .iter()
        .map(|parameter| match (parameter.optional, arguments.next()) {
            (false, Some(value)) => value,
            (true, Some(value)) => format!("Some({value})"),
            (true, None) => "None".into(),
            (false, None) => unreachable!("checked required native argument"),
        })
        .collect::<Vec<_>>()
        .join(", ");
    let NativeBackend::Bundled { symbol } = binding.backend;
    format!("{symbol}({lowered})")
}
