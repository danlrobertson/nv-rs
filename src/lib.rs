/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


#![deny(missing_docs)]

//! # `nv`
//!
//! Rust bindings to the FreeBSD [`libnv`](https://www.freebsd.org/cgi/man.cgi?query=nv)
//! which allows easy management of name/value pairs which may be sent and received
//! over sockets.

/// Error type for the `nv` crate
#[derive(Copy, Clone, Debug)]
pub enum NvErr {
    /// Error in construction of nvlist
    ConstructionErr,
    /// Error not set
    ErrorNotSet(i32)
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
    DescriptorArray = 12
}

/// Enumeration of options available to be passed to
/// the creation of an `nvlist`
#[repr(i32)]
#[derive(Copy, Clone, Debug)]
pub enum NvFlag {
    /// No user specified options
    None = 0,
    /// Use case-insensitive name lookups
    IgnoreCase = 1,
    /// There may be duplicate names in the `nvlist`
    NoUnique = 2,
    /// Both of the previous flags
    All = 3
}

impl NvFlag {
    /// Convert an `i32` into an NvFlag
    pub fn from_i32(flags: i32) -> NvResult<NvFlag> {
        match flags {
            0 => Ok(NvFlag::None),
            1 => Ok(NvFlag::IgnoreCase),
            2 => Ok(NvFlag::NoUnique),
            3 => Ok(NvFlag::All),
            _ => Err(NvErr::ConstructionErr)
        }
    }
}

impl PartialEq for NvFlag {
    fn eq(&self, other: &NvFlag) -> bool {
        *self as i32 == *other as i32
    }
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Default)]
struct nvlist;

/// A list of name/value pairs
pub struct NvList {
    list: Option<*mut nvlist>
}

impl NvList {
    /// Create a new name/value pair list (`nvlist`)
    ///
    /// ```
    /// use nv::{NvList, NvFlag};
    ///
    /// let nvlist = NvList::new(NvFlag::None).unwrap();
    /// ```
    pub fn new(flags: NvFlag) -> NvResult<NvList> {
        let raw_list = unsafe { nvlist_create(flags as i32) };
        if raw_list.is_null() {
            Err(NvErr::ConstructionErr)
        } else {
            Ok(NvList { list: Some(raw_list) })
        }
    }

    /// Determines if the `nvlist` is empty
    ///
    /// ```
    /// use nv::{NvList, NvFlag};
    /// let nvlist = NvList::new(NvFlag::IgnoreCase).unwrap();
    ///
    /// if nvlist.is_empty() {
    ///     println!("Hello, World!");
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        match self.list {
            Some(list) => unsafe { nvlist_empty(list as *const nvlist) },
            None => true,
        }
    }

    /// The flags the `nvlist` was created with
    ///
    /// ```
    /// use nv::{NvList, NvFlag};
    /// let nvlist = NvList::new(NvFlag::NoUnique).unwrap();
    ///
    /// assert_eq!(nvlist.flags(), NvFlag::NoUnique);
    /// ```
    pub fn flags(&self) -> NvFlag {
        match self.list {
            // We know `from_i32` will succeed, so we can just unwrap it
            Some(list) => {
                NvFlag::from_i32(unsafe { nvlist_flags(list as *const nvlist) }).unwrap()
            }
            None => NvFlag::None
        }
    }

    /// Gets error value that the list may have accumulated
    ///
    /// ```
    /// use nv::NvList;
    ///
    /// let list = NvList::default();
    ///
    /// // ENOMEM is returned when the nvlist is null
    /// assert_eq!(0xc, list.error());
    /// ```
    pub fn error(&self) -> i32 {
        match self.list {
            Some(list) => unsafe { nvlist_error(list as *const nvlist) },
            None => 0xc
        }
    }

    /// Sets the `NvList` to be in an error state
    ///
    /// ```
    /// use nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::All).unwrap();
    ///
    /// // EINVAL
    /// list.set_error(0x16).unwrap();
    ///
    /// assert_eq!(0x16, list.error());
    /// ```
    pub fn set_error(&self, error: i32) -> NvResult<()> {
        match self.list {
            Some(list) => Ok(unsafe { nvlist_set_error(list, error) }),
            None => Err(NvErr::ErrorNotSet(error))
        }
    }

    /// Generic add a value to the NvList
    ///
    /// ```
    /// use nv::{NvList, NvFlag, NvListAdd};
    ///
    /// let mut list = NvList::new(NvFlag::All).unwrap();
    ///
    /// let the_answer: u64 = 42;
    /// let not_the_answer: Option<u64> = None;
    ///
    /// list.add("the answer", the_answer);
    /// list.add("not the answer", not_the_answer);
    /// ```
    pub fn add<T: NvListAdd>(&mut self, name: &str, value: T) -> () {
        value.add(self, name);
    }

    /// Add a number to the `NvList`
    ///
    /// ```
    /// use nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::All).unwrap();
    ///
    /// list.add_number("the answer", 42u64);
    /// ```
    pub fn add_number(&mut self, name: &str, value: u64) -> () {
        if let Some(list) = self.list {
            unsafe { nvlist_add_number(list, name.as_bytes().as_ptr(), value); }
        }
    }

    /// Add a null value to the `NvList`
    ///
    /// ```ignore
    /// list.add_null("Hello, World!");
    /// ```
    pub fn add_null(&mut self, name: &str) -> () {
        if let Some(list) = self.list {
            unsafe { nvlist_add_null(list, name.as_bytes().as_ptr()); }
        }
    }
}

/// Trait implemented for types that are allowed to be
/// added to an `NvList`
pub trait NvListAdd {
    /// Add the value to the `NvList`
    fn add(&self, nvlist: &mut NvList, name: &str) -> ();
}

impl NvListAdd for u64 {
    /// Add a `u64` to the `NvList`
    fn add(&self, list: &mut NvList, name: &str) -> () {
        list.add_number(name, *self);
    }
}

impl<T> NvListAdd for Option<T> where T: NvListAdd {
    /// Add an Option value to the `NvList`.
    fn add(&self, list: &mut NvList, name: &str) -> () {
        match self {
            &Some(ref val) => val.add(list, name),
            &None => list.add_null(name)
        }
    }
}

impl Default for NvList {
    fn default() -> NvList {
        NvList { list: None }
    }
}

impl Drop for NvList {
    fn drop(&mut self) {
        if let Some(raw_list) = self.list {
            unsafe { nvlist_destroy(raw_list); }
        }
    }
}

#[link(name="nv")]
extern {
    fn nvlist_create(flags: i32) -> *mut nvlist;
    fn nvlist_destroy(list: *mut nvlist) -> ();
    fn nvlist_empty(list: *const nvlist) -> bool;
    fn nvlist_flags(list: *const nvlist) -> i32;
    fn nvlist_error(list: *const nvlist) -> i32;
    fn nvlist_set_error(list: *mut nvlist, error: i32) -> ();
    fn nvlist_add_number(list: *mut nvlist, name: *const u8, value: u64) -> ();
    fn nvlist_add_null(list: *mut nvlist, name: *const u8) -> ();
}
