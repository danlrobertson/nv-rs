// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use nvlist::NvList;

macro_rules! impl_nv_list_add {
    ($self_:ident, $name:ident, $list:ident.$method:ident, true) => {
        $list.$method($name, $self_);
    };
    ($self_:ident, $name:ident, $list:ident.$method:ident, false) => {
        $list.$method($name, *$self_);
    }
}

macro_rules! impl_nv_list_ops {
    ($type_:ty, $add_method:ident, $get_method:ident) => {
        impl_nv_list_ops!{$type_, $add_method, $get_method, $type_}
    };
    ($type_:ty, $add_method:ident, $get_method:ident, $ret_type:ty) => {
        impl_nv_list_ops!{$type_, $add_method, $get_method, $ret_type, false}
    };
    ($type_:ty, $add_method:ident, $get_method:ident, $ret_type:ty, $deref:ident) => {
        impl NvListOps for $type_ {
            /// Add a `$type_` value to the `NvList`
            fn nv_add(&self, list: &mut NvList, name: &str) -> () {
                impl_nv_list_add!{self, name, list.$add_method, $deref}
            }
        }
    };
}

/// Trait implemented for types that are allowed to be
/// used with an `NvList`
// I really don't like this trait, but in the meantime it
// provides the simplest and most generic interface for adding
// values to `NvList`s
pub trait NvListOps {
    /// Add the value to the `NvList`
    fn nv_add(&self, nvlist: &mut NvList, name: &str) -> ();
}

impl_nv_list_ops!{bool, add_bool, get_bool}

impl_nv_list_ops!{u64, add_number, get_number}

impl_nv_list_ops!{str, add_string, get_string, String, true}

impl_nv_list_ops!{NvList, add_nvlist, get_nvlist, NvList, true}

impl<T> NvListOps for Option<T>
    where T: NvListOps
{
    fn nv_add(&self, list: &mut NvList, name: &str) -> () {
        match self {
            &Some(ref val) => val.nv_add(list, name),
            &None => list.add_null(name),
        }
    }
}
