//! Filesystem Gene
//!
//! namespace: string
//! path: [string]
//! commands:
//!     ls: list files and their meme hash
//!     ln: link a file to a meme
//!     rm: remove the file (keep the meme)
//!
//! Every path can be a file, and the same time, a directory.
//! It is just a meme-tagged forest with namespaces as roots.

use tokio::time::Instant;

use crate::message::{Id, Costs};

pub async fn v1(_uid: &Id, _arg: &str, _change: &mut Costs, _deadline: Instant) -> String {
    "".to_string()
}
