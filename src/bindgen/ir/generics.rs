use std::ops::Deref;
use std::io::Write;
use std::iter::FromIterator;

use syn;

use bindgen::config::{Config, Language};
use bindgen::writer::{Source, SourceWriter};

#[derive(Default, Debug, Clone)]
pub struct GenericParams(pub Option<Vec<String>>);

impl FromIterator<String> for GenericParams {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        GenericParams(Some(iter.into_iter().collect()))
    }
}

impl GenericParams {
    pub fn new(generics: &syn::Generics) -> Self {
        generics
            .params
            .iter()
            .filter_map(|x| match x {
                &syn::GenericParam::Type(syn::TypeParam { ref ident, .. }) => {
                    Some(ident.to_string())
                }
                _ => None,
            })
            .collect()
    }
}

impl Deref for GenericParams {
    type Target = [String];

    fn deref(&self) -> &[String] {
        match self.0 {
            Some(ref generics) => generics,
            None => &[],
        }
    }
}

impl Source for GenericParams {
    fn write<F: Write>(&self, config: &Config, out: &mut SourceWriter<F>) {
        if let Some(ref generics) = self.0 {
            if !generics.is_empty() && config.language == Language::Cxx {
                out.write("template<");
                for (i, item) in generics.iter().enumerate() {
                    if i != 0 {
                        out.write(", ");
                    }
                    write!(out, "typename {}", item);
                }
                out.write(">");
                out.new_line();
            }
        }
    }
}
