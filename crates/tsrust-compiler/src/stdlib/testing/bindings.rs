use crate::native_bindings::{NativeBackend, NativeBinding, NativeParameter, NativeType};
use crate::Mode;
pub(crate) const CHECK_ASSERTION: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:testing-runtime",
    name: "checkAssertion",
    parameters: &[
        NativeParameter { name: "condition", ty: NativeType::Boolean, access: Mode::Copy, optional: false, rest: false },
        NativeParameter { name: "message", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Void,
    backend: NativeBackend::Bundled { symbol: "tsrust_testing_check_assertion" },
    features: &["testing"],
};
pub(crate) const REGISTER_TEST: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:testing-runtime",
    name: "registerTest",
    parameters: &[
        NativeParameter { name: "name", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "callback", ty: NativeType::ProgramVoidCallback, access: Mode::Consume, optional: false, rest: false },
    ],
    result: NativeType::Void,
    backend: NativeBackend::Bundled { symbol: "tsrust_testing_register_test" },
    features: &["testing", "testing-runner"],
};
pub(crate) const RUN_TESTS: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:testing-runtime",
    name: "runTests",
    parameters: &[],
    result: NativeType::Int,
    backend: NativeBackend::Bundled { symbol: "tsrust_testing_run_tests" },
    features: &["testing", "testing-runner"],
};

pub(crate) const FINISH_TESTS: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:testing-runtime", name: "finishTests", parameters: &[], result: NativeType::Void,
    backend: NativeBackend::Bundled { symbol: "tsrust_testing_finish_tests" },
    features: &["testing", "testing-runner"],
};

pub(crate) const BEGIN_SUITE: NativeBinding = NativeBinding { returned_borrow: None, module: "std:testing-runtime", name: "beginSuite", parameters: &[NativeParameter { name: "name", ty: NativeType::String, access: Mode::Read, optional: false, rest: false }], result: NativeType::Void, backend: NativeBackend::Bundled { symbol: "tsrust_testing_begin_suite" }, features: &["testing", "testing-runner"] };

pub(crate) const END_SUITE: NativeBinding = NativeBinding { returned_borrow: None, module: "std:testing-runtime", name: "endSuite", parameters: &[], result: NativeType::Void, backend: NativeBackend::Bundled { symbol: "tsrust_testing_end_suite" }, features: &["testing", "testing-runner"] };

pub(crate) const BEGIN_FILE: NativeBinding = NativeBinding { returned_borrow: None, module: "std:testing-runtime", name: "beginFile", parameters: &[NativeParameter { name: "name", ty: NativeType::String, access: Mode::Read, optional: false, rest: false }], result: NativeType::Void, backend: NativeBackend::Bundled { symbol: "tsrust_testing_begin_file" }, features: &["testing", "testing-runner"] };
pub(crate) const REGISTER_HOOK: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:testing-runtime", name: "registerHook",
    parameters: &[
        NativeParameter { name: "kind", ty: NativeType::Int, access: Mode::Copy, optional: false, rest: false },
        NativeParameter { name: "callback", ty: NativeType::ProgramVoidCallback, access: Mode::Consume, optional: false, rest: false },
    ], result: NativeType::Void,
    backend: NativeBackend::Bundled { symbol: "tsrust_testing_register_hook" },
    features: &["testing", "testing-runner", "testing-hooks"],
};

pub(crate) const SHARE_CALLBACK: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:testing-runtime", name: "shareCallback",
    parameters: &[NativeParameter { name: "callback", ty: NativeType::SharedCallback, access: Mode::Consume, optional: false, rest: false }],
    result: NativeType::SharedCallback,
    backend: NativeBackend::Bundled { symbol: "@compiler-share-callback" }, features: &[],
};

pub(crate) const NUMERIC_VALUE: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:testing-runtime", name: "numericValue",
    parameters: &[NativeParameter { name: "actual", ty: NativeType::ObservedValue, access: Mode::Read, optional: false, rest: false }],
    result: NativeType::Number,
    backend: NativeBackend::Bundled { symbol: "@compiler-testing-numeric" }, features: &[],
};

pub(crate) const LENGTH_VALUE: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:testing-runtime", name: "lengthValue",
    parameters: &[NativeParameter { name: "actual", ty: NativeType::ObservedValue, access: Mode::Read, optional: false, rest: false }],
    result: NativeType::Number, backend: NativeBackend::Bundled { symbol: "@compiler-testing-lengthValue" }, features: &[],
};

pub(crate) const HAS_LENGTH: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:testing-runtime", name: "hasLength",
    parameters: &[NativeParameter { name: "actual", ty: NativeType::ObservedValue, access: Mode::Read, optional: false, rest: false }],
    result: NativeType::Boolean, backend: NativeBackend::Bundled { symbol: "@compiler-testing-hasLength" }, features: &[],
};

pub(crate) const CONTAINS_VALUE: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:testing-runtime", name: "containsValue",
    parameters: &[
        NativeParameter { name: "actual", ty: NativeType::ObservedValue, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "expected", ty: NativeType::ObservedValue, access: Mode::Read, optional: false, rest: false },
    ], result: NativeType::Boolean,
    backend: NativeBackend::Bundled { symbol: "@compiler-testing-contains" }, features: &[],
};

pub(crate) const ASSERTION_DETAILS: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:testing-runtime", name: "assertionDetails",
    parameters: &[
        NativeParameter { name: "actual", ty: NativeType::ObservedValue, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "expected", ty: NativeType::ObservedValue, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "message", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::String, backend: NativeBackend::Bundled { symbol: "@compiler-assertion-details" }, features: &[],
};

pub(crate) const CREATE_REGISTRY: NativeBinding = NativeBinding {
    returned_borrow: None, module: "std:testing-runtime", name: "createRegistry", parameters: &[],
    result: NativeType::Structured(&crate::native_signatures::Signature::Named(crate::native_signatures::NativeTypeId::TestRegistry)),
    backend: NativeBackend::Bundled { symbol: "TsrTestRegistry::default" }, features: &["testing", "testing-runner"],
};

pub(crate) const INVOKE_REGISTRATION: NativeBinding = NativeBinding {
    returned_borrow: None, module: "std:testing-runtime", name: "invokeRegistration",
    parameters: &[NativeParameter { name: "callback", ty: NativeType::ProgramReusableVoidCallback, access: Mode::Mutate, optional: false, rest: false }],
    result: NativeType::Void, backend: NativeBackend::Bundled { symbol: "@compiler-invoke-registration" }, features: &["testing", "testing-runner"],
};

pub(crate) const INVOKE_SUITE: NativeBinding = NativeBinding {
    returned_borrow: None, module: "std:testing-runtime", name: "invokeSuite",
    parameters: &[
        NativeParameter { name: "name", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "callback", ty: NativeType::ProgramReusableVoidCallback, access: Mode::Mutate, optional: false, rest: false },
    ],
    result: NativeType::Void, backend: NativeBackend::Bundled { symbol: "@compiler-invoke-suite" }, features: &["testing", "testing-runner"],
};
