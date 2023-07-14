//! Info gene returns information about this VOxOV instance,
//! including the maintainer, credit rate, and gene list.
//! The gene list is generated by macros,
//! and others are in the config struct.

use crate::message::Id;

pub async fn v1(_uid: &Id, _arg: &str, c: String) -> String {
    c
}
