/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use nvlist::NvList;

/// Trait implemented for types that are allowed to be
/// added to an `NvList`
// I really don't like this trait, but in the meantime it
// provides the simplest and most generic interface for adding
// values to `NvList`s
pub trait NvListAdd {
    /// Add the value to the `NvList`
    fn nv_add(&self, nvlist: &mut NvList, name: &str) -> ();
}

impl NvListAdd for bool {
    /// Add a `bool` to the `NvList`
    fn nv_add(&self, list: &mut NvList, name: &str) -> () {
        list.add_bool(name, *self);
    }
}

impl NvListAdd for u64 {
    /// Add a `u64` to the `NvList`
    fn nv_add(&self, list: &mut NvList, name: &str) -> () {
        list.add_number(name, *self);
    }
}

impl NvListAdd for str {
    /// Add a `&str` to the `NvList`
    fn nv_add(&self, list: &mut NvList, name: &str) -> () {
        list.add_string(name, self);
    }
}

impl NvListAdd for NvList {
    /// Add a `NvList` to another `NvList`
    fn nv_add(&self, list: &mut NvList, name: &str) -> () {
        list.add_nvlist(name, self);
    }
}

impl<T> NvListAdd for Option<T> where T: NvListAdd {
    /// Add an Option value to the `NvList`.
    fn nv_add(&self, list: &mut NvList, name: &str) -> () {
        match self {
            &Some(ref val) => val.nv_add(list, name),
            &None => list.add_null(name)
        }
    }
}

