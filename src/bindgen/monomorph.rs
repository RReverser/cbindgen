/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::mem;

use bindgen::ir::{GenericPath, Item, ItemContainer, Path, Type};

#[derive(Default, Clone, Debug)]
pub struct Monomorphs {
    replacements: HashMap<GenericPath, Path>,
    items: Vec<ItemContainer>,
}

impl Monomorphs {
    pub fn contains(&self, path: &GenericPath) -> bool {
        self.replacements.contains_key(path)
    }

    pub fn insert<T: Item>(&mut self, generic: &T, mut monomorph: T, parameters: Vec<Type>) {
        let replacement_path = GenericPath::new(generic.name().to_owned(), parameters);

        debug_assert!(!generic.generic_params().is_empty());
        debug_assert!(!self.contains(&replacement_path));

        monomorph.set_generic_name(replacement_path.mangle());

        self.replacements
            .insert(replacement_path, monomorph.name().to_owned());
        self.items.push(monomorph.into());
    }

    pub fn mangle_path(&self, path: &GenericPath) -> Option<&Path> {
        self.replacements.get(path)
    }

    pub fn drain(&mut self) -> Vec<ItemContainer> {
        mem::replace(&mut self.items, Vec::new())
    }
}
