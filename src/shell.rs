use anyhow::{Context, Result};
use std::{env, os::unix::process::parent_id, path::PathBuf};

fn str2envkey(s: &str) -> String {
    // [a-zA-Z_][a-zA-Z0-9_]*
    let mut result = String::new();
    let mut chars = s.chars();
    if let Some(c) = chars.next() {
        if c.is_ascii_alphabetic() || c == '_' {
            result.push(c);
        } else {
            result.push('_');
            result.push(c);
        }
    }
    result.extend(chars.filter(|c| c.is_ascii_alphanumeric() || *c == '_'));
    result
}

fn get_session_script_path() -> PathBuf {
    env::temp_dir()
        .join(env::current_exe().unwrap().file_name().unwrap())
        .join(format!("session{}.sh", parent_id()))
}

pub fn write_session_script(script: &str) -> Result<()> {
    let path = get_session_script_path();

    if !path.parent().unwrap().exists() {
        std::fs::create_dir_all(&path.parent().unwrap()).with_context(|| {
            format!(
                "failed to create session script directory: {}",
                path.display()
            )
        })?;
    }

    std::fs::write(&path, script)
        .with_context(|| format!("failed to write session script: {}", path.display()))?;
    Ok(())
}

pub fn get_setup_script() -> String {
    format!(
        "\
        if [ -z ${loaded_flag} ]; then\n\
            export {loaded_flag}=1\n\
            function git() {{\n\
                source \"{session_script_path}\"\n\
                command git $@\n\
            }}\n\
            function {app_name}() {{\n\
                command {app_path} $@\n\
                source \"{session_script_path}\"\n\
            }}\n\
        fi\n\
        ",
        loaded_flag = format!("{}_LOADED", str2envkey(env!("CARGO_PKG_NAME"))),
        app_path = env::current_exe().unwrap().to_string_lossy(),
        app_name = env::args().next().unwrap(),
        session_script_path = get_session_script_path().to_string_lossy(),
    )
}
