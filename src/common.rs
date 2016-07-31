// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.


/// Error type for the `nv` crate
#[derive(Copy, Clone, Debug)]
pub enum NvErr {
    /// Error in construction of nvlist
    ConstructionErr,
    /// Error not set
    ErrorNotSet(i32),
}

/// Result type for the `nv` crate
pub type NvResult<T> = Result<T, NvErr>;

/// Max size the name for a name/value pair may take on
pub const NV_NAME_MAX: u32 = 2048;

/// Enumeration of available data types that the
/// [`nvlist`](https://www.freebsd.org/cgi/man.cgi?query=nv) API supports
#[repr(i32)]
#[derive(Copy, Clone, Debug)]
pub enum NvType {
    /// Empty type
    None = 0,
    /// There is no associated data with the name
    Null = 1,
    /// The value is a `bool` value
    Bool = 2,
    /// The value is a `u64` value
    Number = 3,
    /// The value is a C string
    String = 4,
    /// The value is another `nvlist`
    NvList = 5,
    /// The value is a file descriptor
    Descriptor = 6,
    /// The value is a binary buffer
    Binary = 7,
    /// The value is an array of `bool` values
    BoolArray = 8,
    /// The value is an array of `u64` values
    NumberArray = 9,
    /// The value is an array of C strings
    StringArray = 10,
    /// The value is an array of other `nvlist`'s
    NvListArray = 11,
    /// The value is an array of file descriptors
    DescriptorArray = 12,
}
