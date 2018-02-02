/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::io::Write;

use syn;

use bindgen::config::{Config, Language};
use bindgen::ir::{AnnotationSet, Cfg, CfgWrite, GenericParams, Item, Metadata, Path,
                  TraverseTypes, Type};
use bindgen::writer::{Source, SourceWriter};

/// A type alias that is represented as a C typedef
#[derive(Debug, Clone)]
pub struct Typedef {
    pub name: String,
    pub generic_params: GenericParams,
    pub aliased: Type,
    pub meta: Metadata,
}

impl TraverseTypes for Typedef {
    fn traverse_types<F: FnMut(&Type)>(&self, callback: &mut F) {
        self.aliased.traverse_types(callback);
    }

    fn traverse_types_mut<F: FnMut(&mut Type)>(&mut self, callback: &mut F) {
        self.aliased.traverse_types_mut(callback);
    }
}

impl Typedef {
    pub fn load(item: &syn::ItemType, mod_cfg: &Option<Cfg>) -> Result<Typedef, String> {
        if let Some(x) = Type::load(&item.ty)? {
            Ok(Typedef {
                name: item.ident.to_string(),
                generic_params: GenericParams::new(&item.generics),
                aliased: x,
                meta: Metadata::load(&item.attrs, mod_cfg)?,
            })
        } else {
            Err("Cannot have a typedef of a zero sized type.".to_owned())
        }
    }

    pub fn transfer_annotations(&mut self, out: &mut HashMap<Path, AnnotationSet>) {
        if self.meta.annotations.is_empty() {
            return;
        }

        match self.aliased.get_root_path() {
            Some(alias_path) => {
                if out.contains_key(&alias_path) {
                    warn!(
                        "Multiple typedef's with annotations for {}. Ignoring annotations from {}.",
                        alias_path, self.name
                    );
                    return;
                }

                out.insert(alias_path, self.meta.annotations.clone());
                self.meta.annotations = AnnotationSet::new();
            }
            None => {}
        }
    }
}

impl Item for Typedef {
    fn name(&self) -> &str {
        &self.name
    }

    fn meta(&self) -> &Metadata {
        &self.meta
    }

    fn meta_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn generic_params(&self) -> &GenericParams {
        &self.generic_params
    }

    fn rename_for_config(&mut self, config: &Config) {
        config.export.rename(&mut self.name);
        self.aliased.rename_for_config(config);
    }

    fn set_generic_name(&mut self, new_name: String) {
        self.name = new_name;
        self.generic_params = GenericParams(None);
    }
}

impl Source for Typedef {
    fn write<F: Write>(&self, config: &Config, out: &mut SourceWriter<F>) {
        self.meta.write_before(config, out);

        self.generic_params.write(config, out);

        if config.language == Language::C {
            out.write("typedef ");
            (self.name.clone(), self.aliased.clone()).write(config, out);
        } else {
            write!(out, "using {} = ", self.name);
            self.aliased.write(config, out);
        }
        out.write(";");

        self.meta.write_after(config, out);
    }
}
