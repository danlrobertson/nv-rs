/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


extern crate nv;

mod tests {
    use nv::{NvList, NvFlag};

    #[test]
    fn test_create_nvlist() {
        let x = NvList::new(NvFlag::All);
        let _ = x.clone();
    }
}
