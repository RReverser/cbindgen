/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::mem;

use bindgen::bindings::Bindings;
use bindgen::config::{Config, Language};
use bindgen::dependencies::Dependencies;
use bindgen::error::Error;
use bindgen::ir::{Constant, Function, GenericParams, Item, ItemContainer, ItemMap, Path, Static};
use bindgen::ir::{TraverseTypes, Type};
use bindgen::monomorph::Monomorphs;

#[derive(Debug, Clone)]
pub struct Library {
    config: Config,
    constants: ItemMap<Constant>,
    globals: ItemMap<Static>,
    types: ItemMap<ItemContainer>,
    functions: Vec<Function>,
}

impl TraverseTypes for Library {
    fn traverse_types<F: FnMut(&Type)>(&self, callback: &mut F) {
        self.types.for_all_items(|x| {
            x.traverse_types(callback);
        });
        self.globals.for_all_items(|x| {
            x.traverse_types(callback);
        });
        for x in &self.functions {
            x.traverse_types(callback);
        }
    }

    fn traverse_types_mut<F: FnMut(&mut Type)>(&mut self, callback: &mut F) {
        self.types.for_all_items_mut(|x| {
            x.traverse_types_mut(callback);
        });
        self.globals.for_all_items_mut(|x| {
            x.traverse_types_mut(callback);
        });
        for x in &mut self.functions {
            x.traverse_types_mut(callback);
        }
    }
}

impl Library {
    pub fn new(
        config: Config,
        constants: ItemMap<Constant>,
        globals: ItemMap<Static>,
        types: ItemMap<ItemContainer>,
        functions: Vec<Function>,
    ) -> Library {
        Library {
            config: config,
            constants: constants,
            globals: globals,
            types: types,
            functions: functions,
        }
    }

    pub fn generate(mut self) -> Result<Bindings, Error> {
        self.remove_excluded();
        self.functions.sort_by(|x, y| x.name.cmp(&y.name));
        self.transfer_annotations();
        self.rename_items();
        self.simplify_option_to_ptr();

        if self.config.language == Language::C {
            self.instantiate_monomorphs();
        }

        let mut dependencies = Dependencies::new();

        for function in &self.functions {
            function.add_dependencies_ignoring_generics(
                &GenericParams::default(),
                &self,
                &mut dependencies,
            );
        }

        self.globals.for_all_items(|global| {
            global.add_dependencies(&self, &mut dependencies);
        });

        for name in &self.config.export.include {
            if let Some(items) = self.get_items(name) {
                if !dependencies.items.contains(name) {
                    dependencies.items.insert(name.clone());

                    for item in &items {
                        item.add_dependencies(&self, &mut dependencies);
                    }
                    for item in items {
                        dependencies.order.push(item);
                    }
                }
            }
        }

        dependencies.sort();

        let items = dependencies.order;
        let constants = self.constants.to_vec();
        let globals = self.globals.to_vec();
        let functions = mem::replace(&mut self.functions, Vec::new());

        Ok(Bindings::new(
            self.config.clone(),
            constants,
            globals,
            items,
            functions,
        ))
    }

    pub fn get_items(&self, p: &Path) -> Option<Vec<ItemContainer>> {
        self.types.get_items(p)
    }

    fn remove_excluded(&mut self) {
        let config = &self.config;
        self.functions
            .retain(|x| !config.export.exclude.contains(&x.name));
        self.types
            .filter(|x| config.export.exclude.iter().any(|name| name == x.name()));
        self.globals
            .filter(|x| config.export.exclude.contains(&x.name));
        self.constants
            .filter(|x| config.export.exclude.contains(&x.name));
    }

    fn transfer_annotations(&mut self) {
        let mut annotations = HashMap::new();

        self.types.for_all_items_mut(|x| {
            if let ItemContainer::Typedef(ref mut x) = *x {
                x.transfer_annotations(&mut annotations);
            }
        });

        for (alias_path, annotations) in annotations {
            self.types.for_items_mut(&alias_path, |x| {
                let x_annotations = &mut x.meta_mut().annotations;
                if x_annotations.is_empty() {
                    *x_annotations = annotations.clone();
                } else {
                    warn!(
                        "Can't transfer annotations from typedef to alias ({}) \
                         that already has annotations.",
                        alias_path
                    );
                }
            });
        }
    }

    fn rename_items(&mut self) {
        let config = &self.config;

        self.globals
            .for_all_items_mut(|x| x.rename_for_config(config));
        self.globals.rebuild();

        self.constants
            .for_all_items_mut(|x| x.rename_for_config(config));
        self.constants.rebuild();

        self.types
            .for_all_items_mut(|x| x.rename_for_config(config));
        self.types.rebuild();

        for item in &mut self.functions {
            item.rename_for_config(&self.config);
        }
    }

    fn instantiate_monomorphs(&mut self) {
        // Collect a list of monomorphs
        let mut monomorphs = Monomorphs::default();

        self.add_monomorphs(self, &mut monomorphs);

        // Insert the monomorphs into self
        for monomorph in monomorphs.drain() {
            self.types.try_insert(monomorph);
        }

        // Remove structs and opaque items that are generic
        self.types.filter(|x| x.is_generic());

        // Mangle the paths that remain
        self.mangle_paths(&monomorphs);
    }
}
