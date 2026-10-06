//! Source acceptance expectations and independent handwritten backend evidence.
//! A source rejection may deliberately differ from the backend's outcome.

pub enum SourceExpectation {
    Accept,
    Reject(&'static str),
    BackendOnly,
}

pub enum BackendExpectation {
    Output(&'static str),
    Reject(&'static str),
}

pub struct Case {
    pub name: &'static str,
    pub claim: &'static str,
    pub source: SourceExpectation,
    pub backend: BackendExpectation,
}

use BackendExpectation::{Output, Reject as RustReject};
use SourceExpectation::{Accept, BackendOnly, Reject};

pub const CASES: &[Case] = &[
    Case {
        name: "local_read_reuse",
        claim: "Local read capture permits repeated calls and concurrent owner reads.",
        source: Accept,
        backend: Output("Pat\nPat\nPat\n"),
    },
    Case {
        name: "local_mutate_scope",
        claim: "Mutable capture is reusable and releases its loan at block exit.",
        source: Accept,
        backend: Output("Sam\n"),
    },
    Case {
        name: "local_read_conflict",
        claim: "Owner mutation conflicts with a still-used read capture.",
        source: Reject("capture-conflict"),
        backend: RustReject("E0502"),
    },
    Case {
        name: "competing_mutable_captures",
        claim: "Two live mutable captures cannot share a record.",
        source: Reject("capture-conflict"),
        backend: RustReject("E0499"),
    },
    Case {
        name: "consume_capture_creation",
        claim: "Consuming capture transfers ownership at creation, before invocation.",
        source: Reject("transferred"),
        backend: RustReject("E0382"),
    },
    Case {
        name: "consume_once",
        claim: "A consuming closure returns its captured record once.",
        source: Accept,
        backend: Output("Pat\n"),
    },
    Case {
        name: "repeated_consuming_call",
        claim: "A second invocation of a consuming closure is rejected.",
        source: Reject("closure-consumed"),
        backend: RustReject("E0382"),
    },
    Case {
        name: "returned_read_owned",
        claim: "A returned closure owns a local record while allowing repeated read calls.",
        source: Accept,
        backend: Output("Pat\nPat\n"),
    },
    Case {
        name: "returned_mutate_owned",
        claim: "A returned closure can own and repeatedly mutate its record.",
        source: Accept,
        backend: Output("Sam\nSam\n"),
    },
    Case {
        name: "returned_clone_independent",
        claim: "An explicit clone lets the factory borrow its input and return independent owned state.",
        source: Accept,
        backend: Output("Pat\nSam\n"),
    },
    Case {
        name: "returned_parameter_transfer",
        claim: "Owning a parameter in an escaping closure consumes the caller's record.",
        source: Reject("transferred"),
        backend: RustReject("E0382"),
    },
    Case {
        name: "scalar_copy",
        claim: "Captured integers and booleans copy without consuming their sources or the closure.",
        source: Accept,
        backend: Output("7\n7\n7\ntrue\ntrue\ntrue\n"),
    },
    Case {
        name: "branch_capture_move",
        claim: "A move into an uncalled closure still affects ownership at a branch join.",
        source: Reject("transferred"),
        backend: RustReject("E0382"),
    },
    Case {
        name: "returned_once",
        claim: "A returned consuming closure transfers its owned environment on invocation.",
        source: Accept,
        backend: Output("Pat\n"),
    },
    Case {
        name: "lexical_loan_boundary",
        claim: "The proposed lexical loan rule rejects a case Rust accepts after last use; this is a deliberate prototype restriction.",
        source: Reject("capture-conflict"),
        backend: Output("Pat\nSam\n"),
    },
    Case {
        name: "borrowed_escape",
        claim: "An invalid backend lowering cannot return a closure borrowing a destroyed local.",
        source: BackendOnly,
        backend: RustReject("E0373"),
    },
];
