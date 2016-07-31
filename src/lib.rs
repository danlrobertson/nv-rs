// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.


#![deny(missing_docs)]
#![allow(improper_ctypes)]

//! # `nv`
//!
//! Rust bindings to the FreeBSD [`libnv`](https://www.freebsd.org/cgi/man.cgi?query=nv)
//! which allows easy management of name/value pairs which may be sent and received
//! over sockets.

mod common;
mod nvlist;
mod nvops;

pub use common::{NvErr, NvResult, NvType, NV_NAME_MAX};
pub use nvlist::{NvFlag, NvList};
pub use nvops::NvListOps;
