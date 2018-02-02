/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::Write;

use syn;

use bindgen::config::Config;
use bindgen::ir::{Cfg, Item, Metadata, TraverseTypes, Type};
use bindgen::writer::{Source, SourceWriter};

#[derive(Debug, Clone)]
pub struct Static {
    pub name: String,
    pub ty: Type,
    pub mutable: bool,
    pub meta: Metadata,
}

impl Static {
    pub fn load(item: &syn::ItemStatic, mod_cfg: &Option<Cfg>) -> Result<Static, String> {
        let ty = Type::load(&item.ty)?;

        if ty.is_none() {
            return Err("Cannot have a zero sized static definition.".to_owned());
        }

        Ok(Static {
            name: item.ident.to_string(),
            ty: ty.unwrap(),
            mutable: item.mutability.is_some(),
            meta: Metadata::load(&item.attrs, mod_cfg)?,
        })
    }
}

impl TraverseTypes for Static {
    fn traverse_types<F: FnMut(&Type)>(&self, callback: &mut F) {
        self.ty.traverse_types(callback);
    }

    fn traverse_types_mut<F: FnMut(&mut Type)>(&mut self, callback: &mut F) {
        self.ty.traverse_types_mut(callback);
    }
}

impl Item for Static {
    fn name(&self) -> &str {
        &self.name
    }

    fn meta(&self) -> &Metadata {
        &self.meta
    }

    fn meta_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn rename_for_config(&mut self, config: &Config) {
        self.ty.rename_for_config(config);
    }
}

impl Source for Static {
    fn write<F: Write>(&self, config: &Config, out: &mut SourceWriter<F>) {
        out.write("extern ");
        if let Type::ConstPtr(..) = self.ty {
        } else {
            if !self.mutable {
                out.write("const ");
            }
        }
        self.ty.write(config, out);
        write!(out, " {};", self.name);
    }
}
