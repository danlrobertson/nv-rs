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
            type RetType = $ret_type;
            /// Add a `$type_` value to the `NvList`
            fn nv_add(&self, list: &mut NvList, name: &str) -> () {
                impl_nv_list_add!{self, name, list.$add_method, $deref}
            }

            /// Get a `$type_` from the `NvList`
            fn nv_get(&self, list: &mut NvList, name: &str) -> Option<$ret_type> {
                list.$get_method(name)
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
    /// Return type of `nv_get`
    type RetType;
    /// Add the value to the `NvList`
    fn nv_add(&self, nvlist: &mut NvList, name: &str) -> ();

    /// Retrieve value from an `NvList`
    fn nv_get(&self, nvlist: &mut NvList, name: &str) -> Option<Self::RetType>
        where Self::RetType: Sized;
}

impl_nv_list_ops!{bool, add_bool, get_bool}

impl_nv_list_ops!{u64, add_number, get_number}

impl_nv_list_ops!{str, add_string, get_string, String, true}

impl_nv_list_ops!{NvList, add_nvlist, get_nvlist, NvList, true}

impl<T> NvListOps for Option<T>
    where T: NvListOps
{
    type RetType = Option<T::RetType>;
    /// Add an `Option` value to the `NvList`.
    fn nv_add(&self, list: &mut NvList, name: &str) -> () {
        match self {
            &Some(ref val) => val.nv_add(list, name),
            &None => list.add_null(name),
        }
    }

    /// Get an `Option` value from the `NvList`.
    fn nv_get(&self, list: &mut NvList, name: &str) -> Option<Option<T::RetType>> {
        match self {
            &Some(ref val) => Some(val.nv_get(list, name)),
            &None => None,
        }
    }
}
