//! Map
//!
//! A MongoDB wrapper provides the mapping abstaction for other genes.
//!
//! # VOxOV managed fields
//!
//! - _id: unique identifier.
//! - _uid: user identifier.
//! - _pub: visibility.
//! - _eol: end of life.
//! - _tips: price.
//! - _size: the size of doc.
//!
//! _id and _uid are immutable.
//! _pub is managed by the censor gene.
//! _eol is set in request, and it can be extended.
//!
//! # Indexed fields
//!
//! - _ns: namespace.
//! - _k0, _k1, _k2, _k3: indexed keys. Might increase n in the future.
//! - _geo: geospacial data.
//!
//! _ns is a history lesson in engineering.
//! _kn can have various types. Their meaning is defined by _ns.
//! Range query is supported for _k*.
//! _geo is managed by gene geo.
//!
//! # User defined fields
//! 
//! Everything other than _*.
//!
//! # Query syntax
//!
//! Query object is in json format.
//!
//! ```
//! {
//!     _type: insert/find/delete/prolong,
//!     _id: no insert query,
//!     _pub: bool,
//!     _eol: insert/prolong only,
//!     _tips: uint,
//!     _size: usize,
//!     _ns: string,
//!     _kn: [(range_start, range_end), key, ...]
//!     _geo: TODO
//!     user: any,
//! }
//! ```

use tokio::time::Instant;

use crate::message::{Costs, Id, Uint};

pub async fn v1(
    _uid: &Id,
    _arg: &str,
    _changes: &mut Costs,
    _space: Uint,
    _deadline: Instant,
) -> String {
    "".to_string()
}
