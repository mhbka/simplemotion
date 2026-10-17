pub mod animation;
mod objects;
mod raw;
mod resolved;
mod utils;

pub use objects::*;
pub(super) use raw::RawMotionDocument;
pub use resolved::*;
