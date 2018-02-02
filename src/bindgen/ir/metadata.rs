use std::io::Write;

use syn;

use bindgen::config::Config;
use bindgen::ir::{AnnotationSet, Cfg, CfgWrite, Documentation};
use bindgen::writer::{Source, SourceWriter};

#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub cfg: Option<Cfg>,
    pub annotations: AnnotationSet,
    pub documentation: Documentation,
}

impl Metadata {
    pub fn load(attrs: &[syn::Attribute], mod_cfg: &Option<Cfg>) -> Result<Metadata, String> {
        Ok(Metadata {
            cfg: Cfg::append(mod_cfg, Cfg::load(attrs)),
            annotations: AnnotationSet::load(attrs)?,
            documentation: Documentation::load(attrs),
        })
    }
}

impl CfgWrite for Metadata {
    fn write_before<F: Write>(&self, config: &Config, out: &mut SourceWriter<F>) {
        self.cfg.write_before(config, out);
        self.documentation.write(config, out);
    }

    fn write_after<F: Write>(&self, config: &Config, out: &mut SourceWriter<F>) {
        self.cfg.write_after(config, out);
    }
}
