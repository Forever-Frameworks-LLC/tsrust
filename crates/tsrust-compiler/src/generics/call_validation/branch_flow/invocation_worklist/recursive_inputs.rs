//! Recursive request keys widen array growth before caching the next frame.
use super::*;

#[inline(never)]
pub(super) fn bound(request:&mut Request,declarations:&HashMap<String,Contract>) {
    fn bindings(bindings:&mut [Binding],declarations:&HashMap<String,Contract>) {
        for binding in bindings {
            if let Some(elements)=&mut binding.elements {elements.bound(|left,right|expression_flow::merge_element(left,right,declarations));}
        }
    }
    bindings(&mut request.caller.bindings,declarations);
    for capture in request.caller.captures.values_mut() {
        if let environment_state::Capture::Live {bindings:captured,..}=capture {bindings(captured,declarations);}
    }
    for argument in &mut request.arguments {
        if let Some(elements)=&mut argument.elements {elements.bound(|left,right|expression_flow::merge_element(left,right,declarations));}
    }
}
