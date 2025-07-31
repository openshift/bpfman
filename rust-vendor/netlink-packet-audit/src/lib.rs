// SPDX-License-Identifier: MIT

#[macro_use]
extern crate log;

use core::ops::Range;
/// Represent a multi-bytes field with a fixed size in a packet
pub(crate) type Field = Range<usize>;

mod codec;
pub use codec::NetlinkAuditCodec;

pub mod status;
pub use self::status::*;

pub mod rules;
pub use self::rules::*;

mod message;
pub use self::message::*;

mod buffer;
pub use self::buffer::*;

pub mod constants;
pub use self::constants::*;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;
