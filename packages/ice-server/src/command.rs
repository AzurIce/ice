pub mod bksnap;
pub mod bkarch;

use super::Core;

pub trait Command {
    fn cmd(&self) -> String;
    fn perform(&mut self, core: &mut Core, args: Vec<String>);
}
