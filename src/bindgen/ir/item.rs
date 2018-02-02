/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::BTreeMap;
use std::io::Write;
use std::mem;

use bindgen::config::Config;
use bindgen::dependencies::Dependencies;
use bindgen::ir::{AnnotationSet, Cfg, Constant, Enum, GenericParams, OpaqueItem, Static, Struct,
                  TraverseTypes, Type, Typedef, Union};
use bindgen::library::Library;
use bindgen::mangle;
use bindgen::monomorph::Monomorphs;
use bindgen::writer::{Source, SourceWriter};

/// An item is any type of rust item besides a function
pub trait Item: Source + TraverseTypes + Clone + Into<ItemContainer> {
    fn name(&self) -> &str;
    fn cfg(&self) -> &Option<Cfg>;
    fn annotations(&self) -> &AnnotationSet;
    fn annotations_mut(&mut self) -> &mut AnnotationSet;

    fn generic_params(&self) -> &GenericParams {
        &GenericParams(None)
    }

    fn is_generic(&self) -> bool {
        !self.generic_params().is_empty()
    }

    fn container(&self) -> ItemContainer {
        self.clone().into()
    }

    fn rename_for_config(&mut self, _config: &Config) {}

    fn add_dependencies(&self, library: &Library, out: &mut Dependencies) {
        self.add_dependencies_ignoring_generics(self.generic_params(), library, out);
    }

    fn mangle(&mut self, _new_name: String) {}

    fn instantiate_monomorph(
        &self,
        generic_values: &[Type],
        library: &Library,
        out: &mut Monomorphs,
    ) {
        assert!(self.is_generic());
        assert_eq!(self.generic_params().len(), generic_values.len());

        let mappings = self.generic_params()
            .iter()
            .map(|s| s.as_ref())
            .zip(generic_values.iter())
            .collect::<Vec<_>>();

        let mut monomorph = self.clone();
        monomorph.mangle(mangle::mangle_path(self.name(), &generic_values));
        monomorph.specialize(&mappings);

        monomorph.add_monomorphs(library, out);

        out.insert(self, monomorph, generic_values.to_owned());
    }
}

#[derive(Debug, Clone)]
pub enum ItemContainer {
    Constant(Constant),
    Static(Static),
    OpaqueItem(OpaqueItem),
    Struct(Struct),
    Union(Union),
    Enum(Enum),
    Typedef(Typedef),
}

impl From<Constant> for ItemContainer {
    fn from(src: Constant) -> Self {
        ItemContainer::Constant(src)
    }
}

impl From<Static> for ItemContainer {
    fn from(src: Static) -> Self {
        ItemContainer::Static(src)
    }
}

impl From<OpaqueItem> for ItemContainer {
    fn from(src: OpaqueItem) -> Self {
        ItemContainer::OpaqueItem(src)
    }
}

impl From<Struct> for ItemContainer {
    fn from(src: Struct) -> Self {
        ItemContainer::Struct(src)
    }
}

impl From<Union> for ItemContainer {
    fn from(src: Union) -> Self {
        ItemContainer::Union(src)
    }
}

impl From<Enum> for ItemContainer {
    fn from(src: Enum) -> Self {
        ItemContainer::Enum(src)
    }
}

impl From<Typedef> for ItemContainer {
    fn from(src: Typedef) -> Self {
        ItemContainer::Typedef(src)
    }
}

macro_rules! item_container_exec {
    (@inner ($($mut:ident)*) $self:ident $name:ident $args:tt) => {
        match *$self {
            ItemContainer::Constant(ref $($mut)* x) => x.$name $args,
            ItemContainer::Static(ref $($mut)* x) => x.$name $args,
            ItemContainer::OpaqueItem(ref $($mut)* x) => x.$name $args,
            ItemContainer::Struct(ref $($mut)* x) => x.$name $args,
            ItemContainer::Union(ref $($mut)* x) => x.$name $args,
            ItemContainer::Enum(ref $($mut)* x) => x.$name $args,
            ItemContainer::Typedef(ref $($mut)* x) => x.$name $args,
        }
    };

    (mut $self:ident . $name:ident $args:tt) => {
        item_container_exec!(@inner (mut) $self $name $args)
    };

    ($self:ident . $name:ident $args:tt) => {
        item_container_exec!(@inner () $self $name $args)
    };
}

impl TraverseTypes for ItemContainer {
    fn traverse_types<F: FnMut(&Type)>(&self, callback: &mut F) {
        item_container_exec!(self.traverse_types(callback))
    }

    fn traverse_types_mut<F: FnMut(&mut Type)>(&mut self, callback: &mut F) {
        item_container_exec!(mut self.traverse_types_mut(callback))
    }

    fn simplify_option_to_ptr(&mut self) {
        item_container_exec!(mut self.simplify_option_to_ptr())
    }

    fn add_dependencies_ignoring_generics(
        &self,
        generic_params: &GenericParams,
        library: &Library,
        out: &mut Dependencies,
    ) {
        item_container_exec!(self.add_dependencies_ignoring_generics(generic_params, library, out))
    }
}

impl Source for ItemContainer {
    fn write<F: Write>(&self, config: &Config, writer: &mut SourceWriter<F>) {
        item_container_exec!(self.write(config, writer))
    }
}

impl Item for ItemContainer {
    fn name(&self) -> &str {
        item_container_exec!(self.name())
    }

    fn cfg(&self) -> &Option<Cfg> {
        item_container_exec!(self.cfg())
    }

    fn annotations(&self) -> &AnnotationSet {
        item_container_exec!(self.annotations())
    }

    fn annotations_mut(&mut self) -> &mut AnnotationSet {
        item_container_exec!(mut self.annotations_mut())
    }

    fn generic_params(&self) -> &GenericParams {
        item_container_exec!(self.generic_params())
    }

    fn container(&self) -> ItemContainer {
        self.clone()
    }

    fn rename_for_config(&mut self, config: &Config) {
        item_container_exec!(mut self.rename_for_config(config))
    }

    fn instantiate_monomorph(&self, generics: &[Type], library: &Library, out: &mut Monomorphs) {
        item_container_exec!(self.instantiate_monomorph(generics, library, out))
    }
}

#[derive(Debug, Clone)]
pub enum ItemValue<T: Item> {
    Cfg(Vec<T>),
    Single(T),
}

#[derive(Debug, Clone)]
pub struct ItemMap<T: Item> {
    data: BTreeMap<String, ItemValue<T>>,
}

impl<T: Item> ItemMap<T> {
    pub fn new() -> ItemMap<T> {
        ItemMap {
            data: BTreeMap::new(),
        }
    }

    pub fn rebuild(&mut self) {
        let old = mem::replace(self, ItemMap::new());
        old.for_all_items(|x| {
            self.try_insert(x.clone());
        });
    }

    pub fn try_insert(&mut self, item: T) -> bool {
        match (item.cfg().is_some(), self.data.get_mut(item.name())) {
            (true, Some(&mut ItemValue::Cfg(ref mut items))) => {
                items.push(item);
                return true;
            }
            (false, Some(&mut ItemValue::Cfg(_))) => {
                return false;
            }
            (true, Some(&mut ItemValue::Single(_))) => {
                return false;
            }
            (false, Some(&mut ItemValue::Single(_))) => {
                return false;
            }
            _ => {}
        }

        if item.cfg().is_some() {
            self.data
                .insert(item.name().to_owned(), ItemValue::Cfg(vec![item]));
        } else {
            self.data
                .insert(item.name().to_owned(), ItemValue::Single(item));
        }

        true
    }

    pub fn extend_with(&mut self, other: &ItemMap<T>) {
        other.for_all_items(|x| {
            self.try_insert(x.clone());
        });
    }

    pub fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.data.len());
        for container in self.data.values() {
            match container {
                &ItemValue::Cfg(ref items) => result.extend_from_slice(items),
                &ItemValue::Single(ref item) => {
                    result.push(item.clone());
                }
            }
        }
        result
    }

    pub fn get_items(&self, name: &str) -> Option<Vec<ItemContainer>> {
        match self.data.get(name) {
            Some(&ItemValue::Cfg(ref items)) => Some(items.iter().map(|x| x.container()).collect()),
            Some(&ItemValue::Single(ref item)) => Some(vec![item.container()]),
            None => None,
        }
    }

    pub fn filter<F>(&mut self, callback: F)
    where
        F: Fn(&T) -> bool,
    {
        let data = mem::replace(&mut self.data, BTreeMap::new());

        for (name, container) in data {
            match container {
                ItemValue::Cfg(items) => {
                    let mut new_items = Vec::new();
                    for item in items {
                        if !callback(&item) {
                            new_items.push(item);
                        }
                    }
                    if new_items.len() > 0 {
                        self.data.insert(name, ItemValue::Cfg(new_items));
                    }
                }
                ItemValue::Single(item) => if !callback(&item) {
                    self.data.insert(name, ItemValue::Single(item));
                },
            }
        }
    }

    pub fn for_all_items<F>(&self, mut callback: F)
    where
        F: FnMut(&T),
    {
        for container in self.data.values() {
            match container {
                &ItemValue::Cfg(ref items) => for item in items {
                    callback(item);
                },
                &ItemValue::Single(ref item) => callback(item),
            }
        }
    }

    pub fn for_all_items_mut<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut T),
    {
        for container in self.data.values_mut() {
            match container {
                &mut ItemValue::Cfg(ref mut items) => for item in items {
                    callback(item);
                },
                &mut ItemValue::Single(ref mut item) => callback(item),
            }
        }
    }

    #[allow(dead_code)]
    pub fn for_items<F>(&self, name: &str, mut callback: F)
    where
        F: FnMut(&T),
    {
        match self.data.get(name) {
            Some(&ItemValue::Cfg(ref items)) => for item in items {
                callback(item);
            },
            Some(&ItemValue::Single(ref item)) => {
                callback(item);
            }
            None => {}
        }
    }

    pub fn for_items_mut<F>(&mut self, name: &str, mut callback: F)
    where
        F: FnMut(&mut T),
    {
        match self.data.get_mut(name) {
            Some(&mut ItemValue::Cfg(ref mut items)) => for item in items {
                callback(item);
            },
            Some(&mut ItemValue::Single(ref mut item)) => {
                callback(item);
            }
            None => {}
        }
    }
}
