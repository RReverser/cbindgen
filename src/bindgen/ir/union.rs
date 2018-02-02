/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::Write;

use syn;

use bindgen::config::{Config, Language};
use bindgen::ir::{Cfg, CfgWrite, Documentation, GenericParams, Item, Metadata, Repr,
                  TraverseTypes, Type};
use bindgen::ir::SynFieldHelpers;
use bindgen::rename::{IdentifierType, RenameRule};
use bindgen::utilities::{find_first_some, IterHelpers};
use bindgen::writer::{ListType, Source, SourceWriter};

#[derive(Debug, Clone)]
pub struct Union {
    pub name: String,
    pub generic_params: GenericParams,
    pub fields: Vec<(String, Type, Documentation)>,
    pub tuple_union: bool,
    pub meta: Metadata,
}

impl TraverseTypes for Union {
    fn traverse_types<F: FnMut(&Type)>(&self, callback: &mut F) {
        for &(_, ref ty, _) in &self.fields {
            ty.traverse_types(callback);
        }
    }

    fn traverse_types_mut<F: FnMut(&mut Type)>(&mut self, callback: &mut F) {
        for &mut (_, ref mut ty, _) in &mut self.fields {
            ty.traverse_types_mut(callback);
        }
    }
}

impl Union {
    pub fn load(item: &syn::ItemUnion, mod_cfg: &Option<Cfg>) -> Result<Union, String> {
        if Repr::load(&item.attrs)? != Repr::C {
            return Err("Union is not marked #[repr(C)].".to_owned());
        }

        let (fields, tuple_union) = {
            let out = item.fields
                .named
                .iter()
                .try_skip_map(|x| x.as_ident_and_type())?;
            (out, false)
        };

        Ok(Union {
            name: item.ident.to_string(),
            generic_params: GenericParams::new(&item.generics),
            fields: fields,
            tuple_union: tuple_union,
            meta: Metadata::load(&item.attrs, mod_cfg)?,
        })
    }
}

impl Item for Union {
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
        for &mut (_, ref mut ty, _) in &mut self.fields {
            ty.rename_for_config(config);
        }

        let rules = [
            self.meta.annotations.parse_atom::<RenameRule>("rename-all"),
            config.structure.rename_fields,
        ];

        if let Some(o) = self.meta.annotations.list("field-names") {
            let mut overriden_fields = Vec::new();

            for (i, &(ref name, ref ty, ref doc)) in self.fields.iter().enumerate() {
                if i >= o.len() {
                    overriden_fields.push((name.clone(), ty.clone(), doc.clone()));
                } else {
                    overriden_fields.push((o[i].clone(), ty.clone(), doc.clone()));
                }
            }

            self.fields = overriden_fields;
        } else if let Some(r) = find_first_some(&rules) {
            self.fields = self.fields
                .iter()
                .map(|x| {
                    (
                        r.apply_to_snake_case(&x.0, IdentifierType::StructMember),
                        x.1.clone(),
                        x.2.clone(),
                    )
                })
                .collect();
        } else if self.tuple_union {
            // If we don't have any rules for a tuple union, prefix them with
            // an underscore so it still compiles
            for &mut (ref mut name, ..) in &mut self.fields {
                name.insert(0, '_');
            }
        }
    }

    fn set_generic_name(&mut self, new_name: String) {
        self.name = new_name;
        self.generic_params = GenericParams(None);
    }
}

impl Source for Union {
    fn write<F: Write>(&self, config: &Config, out: &mut SourceWriter<F>) {
        self.meta.write_before(config, out);

        self.generic_params.write(config, out);

        if config.language == Language::C {
            out.write("typedef union");
        } else {
            write!(out, "union {}", self.name);
        }
        out.open_brace();

        if config.documentation {
            out.write_vertical_source_list(&self.fields, ListType::Cap(";"));
        } else {
            out.write_vertical_source_list(
                &self.fields
                    .iter()
                    .map(|&(ref name, ref ty, _)| (name.clone(), ty.clone()))
                    .collect(),
                ListType::Cap(";"),
            );
        }

        if config.language == Language::C {
            out.close_brace(false);
            write!(out, " {};", self.name);
        } else {
            out.close_brace(true);
        }

        self.meta.write_after(config, out);
    }
}
