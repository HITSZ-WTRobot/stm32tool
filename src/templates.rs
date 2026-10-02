pub const APP_CPP: &str = include_str!("templates/app.cpp.tmpl");
pub const CLANG_FORMAT: &str = include_str!("templates/clang-format.tmpl");

#[cfg(test)]
mod tests {
    use super::APP_CPP;
    use crate::render::render_string;

    #[test]
    fn cpp_templates_are_embedded() {
        assert!(APP_CPP.contains("extern \"C\""));
    }

    #[test]
    fn cpp_templates_render_successfully() {
        let rendered_app = render_string(APP_CPP, &()).expect("render app.cpp");

        assert!(rendered_app.contains("extern \"C\" void Init"));
    }
}
