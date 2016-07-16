/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use std::os::unix::io::AsRawFd;
use common::{NvErr, NvResult};
use nvadd::NvListAdd;

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
    /// let copy = list.clone();
    /// list.add("how very meta of you", copy);
    /// ```
    pub fn add<T: NvListAdd>(&mut self, name: &str, value: T) -> () {
        value.nv_add(self, name);
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

    /// Add a `bool` to the list
    pub fn add_bool(&mut self, name: &str, value: bool) -> () {
        if let Some(list) = self.list {
            unsafe { nvlist_add_bool(list, name.as_bytes().as_ptr(), value); }
        }
    }

    /// Add a `u64` to the `NvList`
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

    /// Add string to the list
    pub fn add_string(&mut self, name: &str, value: &str) -> () {
        if let Some(list) = self.list {
            unsafe {
                nvlist_add_string(list, name.as_bytes().as_ptr(),
                                  value.as_bytes().as_ptr())
            }
        }
    }

    /// Add `NvList` to the list
    pub fn add_nvlist(&mut self, name: &str, value: &NvList) -> () {
        match (self.list, value.list) {
            // Both are valid
            (Some(this), Some(other)) if !other.is_null() => {
                unsafe { nvlist_add_nvlist(this, name.as_bytes().as_ptr(), other) }
            }
            // This is valid, but the other is not
            (Some(this), _) => {
                unsafe {
                    nvlist_add_nvlist(this, name.as_bytes().as_ptr(),
                                      nvlist_create(self.flags() as i32))
                }
            }
            // Something bad happened... nop
            _ => {}
        }
    }

    /// Add binary data to the list
    pub unsafe fn add_binary(&mut self, name: &str, value: *mut i8, size: u32) -> () {
        if let Some(list) = self.list {
            nvlist_add_binary(list, name.as_bytes().as_ptr(), value, size);
        }
    }

    /// Write `NvList` to a file descriptor
    ///
    /// ```
    /// use std::fs::File;
    /// use nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::All).unwrap();
    ///
    /// list.add_number("the answer", 42u64);
    ///
    /// list.dump(File::create("/tmp/foo").unwrap());
    /// ```
    pub fn dump<T: AsRawFd>(&self, file: T) -> () {
        if let Some(list) = self.list {
            unsafe { nvlist_dump(list, file.as_raw_fd()) }
        }
    }

    /// The size of the current list
    pub fn len(&self) -> i32 {
        match self.list {
            Some(list) => unsafe { nvlist_size(list) },
            None => 0
        }
    }
}

impl Clone for NvList {
    fn clone(&self) -> NvList {
        NvList { list: self.list.map(|list| unsafe { nvlist_clone(list) }) }
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
    fn nvlist_clone(list: *const nvlist) -> *mut nvlist;
    fn nvlist_dump(list: *const nvlist, fd: i32) -> ();
    fn nvlist_size(list: *const nvlist) -> i32;
    // add value
    fn nvlist_add_null(list: *mut nvlist, name: *const u8) -> ();
    fn nvlist_add_bool(list: *mut nvlist, name: *const u8, value: bool) -> ();
    fn nvlist_add_number(list: *mut nvlist, name: *const u8, value: u64) -> ();
    fn nvlist_add_string(list: *mut nvlist, name: *const u8, value: *const u8) -> ();
    fn nvlist_add_nvlist(list: *mut nvlist, name: *const u8, value: *const nvlist) -> ();
    fn nvlist_add_binary(list: *mut nvlist, name: *const u8,
                         value: *mut i8, size: u32) -> ();
}
