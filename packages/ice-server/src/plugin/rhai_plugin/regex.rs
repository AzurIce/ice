use rhai::{Array, Dynamic};

use rhai::plugin::*;

#[rhai::export_module]
pub mod module {
    /// Check if a string matches a regex
    pub fn regex_match(s: String, re: String) -> bool {
        let re = regex::Regex::new(&re).unwrap();
        re.is_match(&s)
    }

    /// Get all captures of a regex
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

    pub fn regex_captures_iter(s: String, re: String) -> Array {
        let re = regex::Regex::new(&re).unwrap();
        re.captures_iter(&s)
            .map(|cap| {
                cap.iter()
                    .map(|m| m.map_or("".to_string(), |m| m.as_str().to_string()))
                    .map(Dynamic::from)
                    .collect()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::module::*;

    #[test]
    fn test_regex() {
        let s = "]: There are 2 objective(s): [test] [test2]";
        let objectives_regex = r"]: There are \d+ objective\(s\): (.*)";
        let objs = regex_captures(s.to_string(), objectives_regex.to_string());
        println!("objs {:?}", objs);
        let re = r"\[([^\]]+)\]";
        let captures = regex_captures(objs[1].to_string(), re.to_string());
        println!("captures: {:?}", captures);
        let captures_iter = regex_captures_iter(objs[1].to_string(), re.to_string());
        println!("captures_iter: {:?}", captures_iter);
    }
}
