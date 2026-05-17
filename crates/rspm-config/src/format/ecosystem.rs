//! `ecosystem.config.js` loader backed by Boa.

use std::path::Path;

use boa_engine::{Context, Source};
use rspm_core::{Result, RspmError};

use crate::loader::parse_json_document;
use crate::schema::ConfigDocument;

/// Loads a JavaScript ecosystem config document.
pub fn load(path: &Path) -> Result<ConfigDocument> {
    let src = std::fs::read_to_string(path)?;
    parse_ecosystem(&src)
}

/// Evaluates a PM2-style `module.exports = { apps: [...] }` config.
///
/// ```
/// let js = "module.exports = { apps: [{ name: 'api', script: 'api.js' }] };";
/// let document = rspm_config::format::ecosystem::parse_ecosystem(js).expect("valid js");
/// assert_eq!(document.into_apps().len(), 1);
/// ```
pub fn parse_ecosystem(js_src: &str) -> Result<ConfigDocument> {
    let wrapped = format!(
        "var module = {{ exports: {{}} }}; var exports = module.exports;\n{}\nJSON.stringify(module.exports);",
        js_src
    );
    let mut context = Context::default();
    let value = context
        .eval(Source::from_bytes(&wrapped))
        .map_err(|err| RspmError::Config(format!("ecosystem js eval failed: {err}")))?;
    let json = value
        .to_string(&mut context)
        .map_err(|err| RspmError::Config(format!("ecosystem stringify failed: {err}")))?
        .to_std_string_escaped();
    parse_json_document(&json)
}
