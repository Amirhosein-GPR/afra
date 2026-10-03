use regex::Regex;

use crate::module::config::BINARY_EXT;

pub struct RegexContainer {
    pub src_file_regex: Regex,
    pub bin_file_regex: Regex,
    pub branches: Regex,
    pub definition_label: Regex,
    pub redundant_function_call: Regex,
    pub original_source_line_number: Regex,
    pub processed_source_line_number: Regex,
    pub target_label: Regex,
    pub target_address: Regex,
    pub gem5_trace_instruction_address: Regex,
    pub jump_to_subroutine: Regex,
    pub instruction_name: Regex,
}

impl RegexContainer {
    pub fn new() -> Self {
        Self {
            src_file_regex: Regex::new(r"(benchmarks[\w/-]*)sources([\w/-]*)\.(?:c|cc)$").unwrap(),
            bin_file_regex: Regex::new(
                    format!(
                        r"(benchmarks[\w/-]*)binaries([\w/-]*){}$",
                        regex::escape(BINARY_EXT)
                    )
                    .as_str(),
                )
                .unwrap(),
            branches: Regex::new(r"\b\s+((?:b\.\w+)|(?:bc\.\w+)|(?:blrabz)|(?:blrab)|(?:blraaz)|(?:blraa)|(?:bl)|(?:blr)|(?:br)|(?:ret)|(?:b)|(?:cbnz)|(?:cbz)|(?:tbnz)|(?:tbz))\s+\b").unwrap(),
            definition_label: Regex::new(r"^[0-9a-f]+\s<([.\w]*)>:$").unwrap(),
            redundant_function_call: Regex::new(r"^\S+\(\):$").unwrap(),
            original_source_line_number: Regex::new(r"/\S+/(\S+)").unwrap(),
            processed_source_line_number: Regex::new(r"^\[[\w.]*:(\d+)\]\s+").unwrap(),
            target_label: Regex::new(r"<\w+>$").unwrap(),
            target_address: Regex::new(r"(\w+)(?:\s+<.+>)?$").unwrap(),
            gem5_trace_instruction_address: Regex::new(r".+?:.+?:.+?:\s+0x(\w+)[.\s\d]+:").unwrap(),
            jump_to_subroutine: Regex::new(r"^[0-9a-f]+:\s+[0-9a-f]+\s+(?:blr|bl)").unwrap(),
            instruction_name: Regex::new(r"\w+:\s+\w+\s+([\w.]+)").unwrap(),
        }
    }
}
