use std::sync::{Arc, Mutex};

use super::Core;

// pub enum Command {
//     SimpleCommand(dyn Task),
//     ConfirmCommand(dyn Task),
// }

pub trait Task {
    fn perform(core: Arc<Mutex<Core>>) where Self: Sized;
}