//! Environment expansion helpers.

use rspm_core::types::EnvMap;

/// Expands `${NAME}` references using the provided environment map first, then process env.
///
/// ```
/// let mut env = rspm_core::types::EnvMap::new();
/// env.insert("PORT".into(), "3000".into());
/// assert_eq!(rspm_config::env_expand::expand(":${PORT}", &env), ":3000");
/// ```
pub fn expand(value: &str, env: &EnvMap) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'{') {
            let _ = chars.next();
            let mut name = String::new();
            for next in chars.by_ref() {
                if next == '}' {
                    break;
                }
                name.push(next);
            }
            let replacement = env
                .get(&name)
                .cloned()
                .or_else(|| std::env::var(&name).ok())
                .unwrap_or_default();
            output.push_str(&replacement);
        } else {
            output.push(ch);
        }
    }

    output
}
