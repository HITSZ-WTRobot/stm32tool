use serde::Serialize;
use std::fs;
use std::path::Path;
use tinytemplate::TinyTemplate;
use tracing::{debug, warn};

pub fn render_file<T: Serialize>(
    path: &str,
    template: &str,
    ctx: &T,
    force: bool,
) -> std::io::Result<()> {
    let output_path = Path::new(path);
    debug!(path, force, "Preparing to render file");

    if output_path.exists() && !force {
        warn!("Skip existing {}", path);
        return Ok(());
    }

    if let Some(parent) = output_path.parent() {
        debug!(parent = %parent.display(), "Ensuring parent directory exists");
        fs::create_dir_all(parent)?;
    }

    debug!(path, template_bytes = template.len(), "Rendering template");
    let content = render_string(template, ctx)?;

    debug!(path, bytes = content.len(), "Writing rendered file");
    fs::write(path, content)?;
    Ok(())
}

pub fn render_string<T: Serialize>(template: &str, ctx: &T) -> std::io::Result<String> {
    debug!(
        template_bytes = template.len(),
        "Rendering template into string"
    );
    let mut tt = TinyTemplate::new();
    tt.add_template("tpl", template).unwrap();

    let content = tt.render("tpl", ctx).unwrap();
    debug!(bytes = content.len(), "Rendered template string");

    Ok(content)
}
