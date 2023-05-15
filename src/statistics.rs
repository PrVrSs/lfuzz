use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct Statistics {
    /// Number of fuzzing cases
    pub count: u32,

    /// List of all unique fuzzer keys
    pub unique_keys: HashSet<u32>,
}
