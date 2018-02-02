/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::Write;
use std::mem;

use syn;

use bindgen::config::{Config, Language};
use bindgen::ir::{Cfg, Item, Metadata, TraverseTypes, Type};
use bindgen::writer::{Source, SourceWriter};

#[derive(Debug, Clone)]
pub struct LiteralExpr(String);

impl LiteralExpr {
    pub fn load(expr: &syn::Expr) -> Result<LiteralExpr, String> {
        match expr {
            &syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(ref value),
                ..
            }) => Ok(LiteralExpr(format!("u8\"{}\"", value.value()))),
            &syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Byte(ref value),
                ..
            }) => Ok(LiteralExpr(format!("{}", value.value()))),
            &syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Char(ref value),
                ..
            }) => Ok(LiteralExpr(format!("{}", value.value()))),
            &syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(ref value),
                ..
            }) => match value.suffix() {
                syn::IntSuffix::Usize
                | syn::IntSuffix::U8
                | syn::IntSuffix::U16
                | syn::IntSuffix::U32
                | syn::IntSuffix::U64
                | syn::IntSuffix::U128
                | syn::IntSuffix::None => Ok(LiteralExpr(format!("{}", value.value()))),
                syn::IntSuffix::Isize
                | syn::IntSuffix::I8
                | syn::IntSuffix::I16
                | syn::IntSuffix::I32
                | syn::IntSuffix::I64
                | syn::IntSuffix::I128 => unsafe {
                    Ok(LiteralExpr(format!(
                        "{}",
                        mem::transmute::<u64, i64>(value.value())
                    )))
                },
            },
            &syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Float(ref value),
                ..
            }) => Ok(LiteralExpr(format!("{}", value.value()))),
            &syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Bool(ref value),
                ..
            }) => Ok(LiteralExpr(format!("{}", value.value))),
            _ => Err("Unsupported literal expression.".to_owned()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub name: String,
    pub ty: Type,
    pub value: LiteralExpr,
    pub meta: Metadata,
}

impl TraverseTypes for Constant {
    fn traverse_types<F: FnMut(&Type)>(&self, callback: &mut F) {
        self.ty.traverse_types(callback);
    }

    fn traverse_types_mut<F: FnMut(&mut Type)>(&mut self, callback: &mut F) {
        self.ty.traverse_types_mut(callback);
    }
}

impl Constant {
    pub fn load(
        name: String,
        item: &syn::ItemConst,
        mod_cfg: &Option<Cfg>,
    ) -> Result<Constant, String> {
        let ty = Type::load(&item.ty)?;

        if ty.is_none() {
            return Err("Cannot have a zero sized const definition.".to_owned());
        }

        let ty = ty.unwrap();

        if !ty.is_primitive_or_ptr_primitive() {
            return Err("Cannot have a non primitive const definition.".to_owned());
        }

        Ok(Constant {
            name: name,
            ty: ty,
            value: LiteralExpr::load(&item.expr)?,
            meta: Metadata::load(&item.attrs, mod_cfg)?,
        })
    }
}

impl Item for Constant {
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
        config.export.rename(&mut self.name);
    }
}

impl Source for Constant {
    fn write<F: Write>(&self, config: &Config, out: &mut SourceWriter<F>) {
        if config.constant.allow_static_const && config.language == Language::Cxx {
            if let Type::ConstPtr(..) = self.ty {
                out.write("static ");
            } else {
                out.write("static const ");
            }
            self.ty.write(config, out);
            write!(out, " {} = {};", self.name, self.value.0)
        } else {
            write!(out, "#define {} {}", self.name, self.value.0)
        }
    }
}
