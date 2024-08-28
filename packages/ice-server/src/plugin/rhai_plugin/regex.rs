use rhai::{Array, Dynamic};

use rhai::plugin::*;

#[rhai::export_module]
pub mod module {
    pub fn regex_match(s: String, re: String) -> bool {
        let re = regex::Regex::new(&re).unwrap();
        re.is_match(&s)
    }

    pub fn regex_captures(s: String, re: String) -> Array {
        let re = regex::Regex::new(&re).unwrap();
        re.captures(&s)
            .map(|cap| {
                cap.iter()
                    .map(|m| m.map_or("".to_string(), |m| m.as_str().to_string()))
                    .map(Dynamic::from)
                    .collect()
            })
            .unwrap_or(vec![])
    }
}
