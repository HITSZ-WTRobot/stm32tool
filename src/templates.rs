pub const APP_CPP: &str = include_str!("templates/app.cpp.tmpl");
pub const ARENA_CPP: &str = include_str!("templates/arena.cpp.tmpl");
pub const CLANG_FORMAT: &str = include_str!("templates/clang-format.tmpl");

#[cfg(test)]
mod tests {
    use super::{APP_CPP, ARENA_CPP};
    use crate::render::render_string;

    #[test]
    fn cpp_templates_are_embedded() {
        assert!(APP_CPP.contains("extern \"C\""));
        assert!(ARENA_CPP.contains("StaticArena<64 * 1024>"));
    }

    #[test]
    fn cpp_templates_render_successfully() {
        let rendered_app = render_string(APP_CPP, &()).expect("render app.cpp");
        let rendered_arena = render_string(ARENA_CPP, &()).expect("render arena.cpp");

        assert!(rendered_app.contains("extern \"C\" void Init"));
        assert!(rendered_arena.contains("operator new"));
    }
}
