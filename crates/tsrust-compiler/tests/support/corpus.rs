use tsrust_compiler::Mode;

pub struct Case {
    pub name: &'static str,
    pub claim: &'static str,
    pub contracts: &'static [(&'static str, Mode)],
    pub expected: Expected,
}

pub enum Expected {
    Output(&'static str),
    Rejected {
        source_code: &'static str,
        rust_code: &'static str,
    },
}

pub const CASES: &[Case] = &[
    Case {
        name: r#"read_reuse"#,
        claim: r#"Reading a parameter does not consume the caller value."#,
        contracts: &[(r#"log.user"#, Mode::Read)],
        expected: Expected::Output(
            r#"Pat
Pat
"#,
        ),
    },
    Case {
        name: r#"mutate_reuse"#,
        claim: r#"A mutable argument borrow ends before the next call."#,
        contracts: &[
            (r#"rename.user"#, Mode::Mutate),
            (r#"log.user"#, Mode::Read),
        ],
        expected: Expected::Output(
            r#"Sam
"#,
        ),
    },
    Case {
        name: r#"move_assignment"#,
        claim: r#"An owned assignment transfers the value to its new binding."#,
        contracts: &[],
        expected: Expected::Output(
            r#"Pat
"#,
        ),
    },
    Case {
        name: r#"use_after_assignment"#,
        claim: r#"Reading the original binding after owned assignment is rejected."#,
        contracts: &[],
        expected: Expected::Rejected {
            source_code: r#"transferred"#,
            rust_code: r#"E0382"#,
        },
    },
    Case {
        name: r#"clone_independence"#,
        claim: r#"Explicit cloning creates an independent record and string."#,
        contracts: &[(r#"rename.user"#, Mode::Mutate)],
        expected: Expected::Output(
            r#"Pat
Sam
"#,
        ),
    },
    Case {
        name: r#"consume_return"#,
        claim: r#"Returning an owned parameter transfers it through the call."#,
        contracts: &[(r#"take.user"#, Mode::Consume)],
        expected: Expected::Output(
            r#"Pat
"#,
        ),
    },
    Case {
        name: r#"use_after_call"#,
        claim: r#"Reading a value after passing it to a consuming parameter is rejected."#,
        contracts: &[(r#"take.user"#, Mode::Consume)],
        expected: Expected::Rejected {
            source_code: r#"transferred"#,
            rust_code: r#"E0382"#,
        },
    },
    Case {
        name: r#"two_read_arguments"#,
        claim: r#"The same record can be passed to two read-only parameters."#,
        contracts: &[
            (r#"logBoth.first"#, Mode::Read),
            (r#"logBoth.second"#, Mode::Read),
        ],
        expected: Expected::Output(
            r#"Pat
Pat
"#,
        ),
    },
    Case {
        name: r#"overlapping_access"#,
        claim: r#"One call cannot hold mutable and immutable access to the same record."#,
        contracts: &[
            (r#"renameAndLog.first"#, Mode::Mutate),
            (r#"renameAndLog.second"#, Mode::Read),
        ],
        expected: Expected::Rejected {
            source_code: r#"conflicting-access"#,
            rust_code: r#"E0502"#,
        },
    },
    Case {
        name: r#"transitive_mutation"#,
        claim: r#"A caller inherits the mutable access required by its callee."#,
        contracts: &[
            (r#"rename.user"#, Mode::Mutate),
            (r#"normalize.user"#, Mode::Mutate),
        ],
        expected: Expected::Output(
            r#"Sam
"#,
        ),
    },
    Case {
        name: r#"owned_string_result"#,
        claim: r#"String transforms return owned results without consuming their input record."#,
        contracts: &[(r#"normalize.user"#, Mode::Read)],
        expected: Expected::Output(
            r#"pat
 Pat 
"#,
        ),
    },
];
