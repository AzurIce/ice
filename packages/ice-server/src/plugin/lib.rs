use std::sync::{Arc, Mutex};

use rhai::{Array, Dynamic, Engine};
use rune::{ContextError, Module};

use crate::server::Server;

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

pub fn engine_with_lib() -> Engine {
    let mut engine = Engine::new();
    engine.register_fn("regex_match", regex_match);
    engine.register_fn("regex_captures", regex_captures);
    engine
}
