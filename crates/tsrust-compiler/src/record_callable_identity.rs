//! Explicit callable identities keep source argument slots separate from binding IDs.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Callable {
    Function(usize),
    Closure(usize),
}

pub(super) struct View<'a> {
    pub(super) function: &'a CheckedFunction,
    pub(super) parameters: Vec<usize>,
}

impl Callable {
    pub(super) fn view<'a>(
        self,
        functions: &'a [Option<CheckedFunction>],
        closures: &'a [CheckedClosure],
    ) -> Option<View<'a>> {
        match self {
            Self::Function(id) => {
                let function = functions.get(id)?.as_ref()?;
                Some(View {
                    function,
                    parameters: (0..function.parameter_count).collect(),
                })
            }
            Self::Closure(id) => {
                let closure = closures.get(id)?;
                Some(View {
                    function: &closure.function,
                    parameters: closure.parameters.clone(),
                })
            }
        }
    }

    pub(super) fn call(expr: &Expr) -> Option<Self> {
        match expr.target {
            Some(CallTarget::Function(id) | CallTarget::ForwardFunction(id)) => {
                Some(Self::Function(id))
            }
            Some(CallTarget::Closure(_)) => match expr.callable_origin {
                Some(CallableOrigin::Closure(id)) => Some(Self::Closure(id)),
                _ => None,
            },
            _ => None,
        }
    }
}
