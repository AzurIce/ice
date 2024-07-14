use std::{error::Error, io::Write, path::Path};

use curl::easy::Easy;
use log::info;

pub mod api;
pub mod config;
pub mod core;
pub mod error;
pub mod loader;
pub mod utils;
