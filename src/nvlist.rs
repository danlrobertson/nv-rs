/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use std::str;
use std::os::unix::io::AsRawFd;
use common::{NvErr, NvResult, NvType};
use nvops::NvListOps;

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

    /// Genericially add a single value to the NvList
    ///
    /// ```
    /// use nv::{NvList, NvFlag, NvListOps};
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
    pub fn add<T: NvListOps>(&mut self, name: &str, value: T) -> () {
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
    ///
    /// ```
    /// use nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::All).unwrap();
    ///
    /// let other_list = NvList::default();
    ///
    /// list.add_nvlist("other list", &other_list);
    ///
    /// assert_eq!(other_list.get_bool("something"), None);
    /// ```
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

    /// Add slice of `bool` values
    ///
    /// ```
    /// use nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::All).unwrap();
    ///
    /// let slice = [true, false, true, false];
    ///
    /// list.add_bool_slice("the answer", &slice);
    /// ```
    pub fn add_bool_slice(&mut self, name: &str, value: &[bool]) -> () {
        if let Some(list) = self.list {
            unsafe {
                nvlist_add_bool_array(list, name.as_bytes().as_ptr(),
                                      value.as_ptr(), value.len());
            }
        }
    }

    /// Add slice of `u64`s
    ///
    /// ```
    /// use nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::All).unwrap();
    ///
    /// let slice = [42, 100];
    ///
    /// list.add_number_slice("the answer", &slice);
    /// ```
    pub fn add_number_slice(&mut self, name: &str, value: &[u64]) -> () {
        if let Some(list) = self.list {
            unsafe {
                nvlist_add_number_array(list, name.as_bytes().as_ptr(),
                                        value.as_ptr(), value.len());
            }
        }
    }

    /// Add a slice of strings
    pub fn add_string_slice(&mut self, name: &str, value: &[&str]) -> () {
        if let Some(list) = self.list {
            unsafe {
                let tmp: Vec<*const u8> = value.iter().map(|item| {
                    item.as_bytes().as_ptr()
                }).collect();
                nvlist_add_string_array(list, name.as_bytes().as_ptr(),
                                        tmp.as_slice().as_ptr(),
                                        value.len());
            }
        }
    }

    /// Add a slice of `NvList`s
    ///
    /// ```
    /// use nv::{NvList, NvFlag};
    ///
    /// let mut list = NvList::new(NvFlag::All).unwrap();
    ///
    /// let slice = [NvList::default(), NvList::new(NvFlag::All).unwrap(),
    ///              NvList::new(NvFlag::None).unwrap()];
    ///
    /// list.add_nvlist_slice("nvlists", &slice);
    /// ```
    pub fn add_nvlist_slice(&mut self, name: &str, value: &[NvList]) -> () {
        if let Some(list) = self.list {
            unsafe {
                let tmp: Vec<*const nvlist> = value.iter().filter(|item| match item.list {
                    Some(item) if !item.is_null() => true,
                    _ => false
                }).map(|item| {
                    item.list.unwrap() as *const nvlist
                }).collect();
                nvlist_add_nvlist_array(list, name.as_bytes().as_ptr(),
                                        tmp.as_slice().as_ptr(), tmp.len());
            }
        }
    }

    /// Returns `true` if a name/value pair
    /// exists in the `NvList` and `false`
    /// otherwise
    pub fn exists(&self, name: &str) -> bool {
        match self.list {
            Some(list) => {
                unsafe { nvlist_exists(list, name.as_bytes().as_ptr()) }
            },
            _ => false
        }
    }

    /// Returns `true` if a name/value pair
    /// of the specified type exists in the
    /// `NvList` and `false` otherwise
    pub fn exists_type(&self, name: &str, ty: NvType) -> bool {
        match self.list {
            Some(list) => {
                unsafe {
                    nvlist_exists_type(list, name.as_bytes().as_ptr(),
                                       ty as i32)
                }
            },
            None => false
        }
    }

    /// Get the first matching `bool` value paired with
    /// the given name
    ///
    /// ```
    /// use nv::{NvList, NvFlag};
    ///
    /// // Note: we're allowing duplicate values per name
    /// let mut list = NvList::new(NvFlag::All).unwrap();
    ///
    /// list.add_bool("is rust awesome?", true);
    /// list.add_bool("is rust awesome?", false);
    ///
    /// assert!(list.get_bool("is rust awesome?").unwrap(), true);
    /// ```
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        match self.list {
            Some(list) => {
                unsafe {
                    let char_arr = name.as_bytes().as_ptr();
                    if nvlist_exists_bool(list, char_arr) {
                        Some(nvlist_get_bool(list, name.as_bytes().as_ptr()))
                    } else {
                        None
                    }
                }
            },
            _ => None
        }
    }

    /// Get the first matching `u64` value paired with
    /// the given name
    pub fn get_number(&self, name: &str) -> Option<u64> {
        match self.list {
            Some(list) => {
                unsafe {
                    let char_arr = name.as_bytes().as_ptr();
                    if nvlist_exists_number(list, char_arr) {
                        Some(nvlist_get_number(list, name.as_bytes().as_ptr()))
                    } else {
                        None
                    }
                }
            },
            _ => None
        }
    }

    /// Get the first matching `u64` value paired with
    /// the given name
    ///
    /// ```
    /// use nv::{NvList, NvFlag};
    ///
    /// // Note: we're allowing duplicate values per name
    /// let mut list = NvList::new(NvFlag::None).unwrap();
    ///
    /// list.add_string("Hello", "World!");
    ///
    /// assert_eq!(list.get_string("Hello").unwrap(), "World!");
    /// ```
    pub fn get_string(&self, name: &str) -> Option<String> {
        match self.list {
            Some(list) => {
                unsafe {
                    let char_arr = name.as_bytes().as_ptr();
                    if nvlist_exists_string(list, char_arr) {
                        let ret = nvlist_get_string(list, name.as_bytes().as_ptr());
                        if ret.is_null() {
                            None
                        } else {
                            let len = strlen(ret);
                            Some(String::from_raw_parts(ret as *mut u8, len, len))
                        }
                    } else {
                        None
                    }
                }
            },
            _ => None
        }
    }

    /// Get the first matching `NvList` value paired with
    /// the given name and clone it
    ///
    /// ```
    /// use nv::{NvList, NvFlag};
    ///
    /// // Note: we're allowing duplicate values per name
    /// let mut list = NvList::new(NvFlag::All).unwrap();
    ///
    /// list.add_bool("other list", true);
    ///
    /// let mut other_list = NvList::new(NvFlag::None).unwrap();
    /// other_list.add_number("the answer", 42);
    ///
    /// list.add_nvlist("other list", &other_list);
    ///
    /// // Note: Since we use `get_nvlist` we will get the
    /// // NvList not the boolean value
    /// let other_nvlist = list.get_nvlist("other list").unwrap();
    ///
    /// assert_eq!(other_nvlist.get_number("the answer").unwrap(), 42);
    /// ```
    pub fn get_nvlist(&self, name: &str) -> Option<NvList> {
        match self.list {
            Some(list) => {
                unsafe {
                    let char_arr = name.as_bytes().as_ptr();
                    if nvlist_exists_nvlist(list, char_arr) {
                        let res = nvlist_get_nvlist(list, name.as_bytes().as_ptr());
                        Some(NvList {
                            list: Some(nvlist_clone(res))
                        })
                    } else {
                        None
                    }
                }
            },
            _ => None
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
    fn nvlist_add_bool_array(list: *mut nvlist, name: *const u8,
                             value: *const bool, size: usize) -> ();
    fn nvlist_add_number_array(list: *mut nvlist, name: *const u8,
                               value: *const u64, size: usize) -> ();
    fn nvlist_add_string_array(list: *mut nvlist, name: *const u8,
                               value: *const *const u8, size: usize) -> ();
    fn nvlist_add_nvlist_array(list: *mut nvlist, name: *const u8,
                               value: *const *const nvlist, size: usize) -> ();
    fn nvlist_exists(list: *mut nvlist, name: *const u8) -> bool;
    fn nvlist_exists_type(list: *const nvlist, name: *const u8, ty: i32) -> bool;
    fn nvlist_exists_bool(list: *const nvlist, name: *const u8) -> bool;
    fn nvlist_exists_number(list: *const nvlist, name: *const u8) -> bool;
    fn nvlist_exists_string(list: *const nvlist, name: *const u8) -> bool;
    fn nvlist_exists_nvlist(list: *const nvlist, name: *const u8) -> bool;
    fn nvlist_get_bool(list: *mut nvlist, name: *const u8) -> bool;
    fn nvlist_get_number(list: *mut nvlist, name: *const u8) -> u64;
    fn nvlist_get_string(list: *mut nvlist, name: *const u8) -> *const u8;
    fn nvlist_get_nvlist(list: *mut nvlist, name: *const u8) -> *const nvlist;
    fn strlen(target: *const u8) -> usize;
}
