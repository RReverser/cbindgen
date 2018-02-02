/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::Write;

use syn;

use bindgen::config::{Config, Language};
use bindgen::ir::{AnnotationSet, Cfg, CfgWrite, Documentation, GenericParams, Item, Metadata,
                  Path, TraverseTypes, Type};
use bindgen::writer::{Source, SourceWriter};

#[derive(Debug, Clone)]
pub struct OpaqueItem {
    pub name: Path,
    pub generic_params: GenericParams,
    pub meta: Metadata,
}

impl TraverseTypes for OpaqueItem {
    fn traverse_types<F: FnMut(&Type)>(&self, _callback: &mut F) {}

    fn traverse_types_mut<F: FnMut(&mut Type)>(&mut self, _callback: &mut F) {}
}

impl OpaqueItem {
    pub fn new(
        name: String,
        generics: &syn::Generics,
        attrs: &[syn::Attribute],
        mod_cfg: &Option<Cfg>,
    ) -> OpaqueItem {
        OpaqueItem {
            name: name,
            generic_params: GenericParams::new(generics),
            meta: Metadata {
                cfg: Cfg::append(mod_cfg, Cfg::load(attrs)),
                annotations: AnnotationSet::load(attrs).unwrap_or(AnnotationSet::new()),
                documentation: Documentation::load(attrs),
            },
        }
    }
}

impl Item for OpaqueItem {
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
    }

    fn set_generic_name(&mut self, new_name: String) {
        self.name = new_name;
        self.generic_params = GenericParams(None);
    }
}

impl Source for OpaqueItem {
    fn write<F: Write>(&self, config: &Config, out: &mut SourceWriter<F>) {
        self.meta.write_before(config, out);

        self.generic_params.write(config, out);

        if config.language == Language::C {
            write!(out, "typedef struct {} {};", self.name, self.name);
        } else {
            write!(out, "struct {};", self.name);
        }

        self.meta.write_after(config, out);
    }
}
