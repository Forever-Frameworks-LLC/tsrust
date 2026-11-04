fn tsr_string_array_join(values: &Array<String>, separator: &str) -> String {
    let output = match values {
        Array::Heap(values) => values.join(separator),
        Array::Small { values, len } => {
            let mut parts = [""; 4];
            for (part, value) in parts.iter_mut().zip(&values[..*len]) {
                *part = value.as_ref().expect("occupied inline array slot").as_ref();
            }
            parts[..*len].join(separator)
        }
    };
    String::Owned(output)
}
