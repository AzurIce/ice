use ice_util::minecraft::rtext::{self, ComponentObject};
use rhai::{def_package, plugin::*, Array};

#[allow(non_upper_case_globals)]
#[rhai::export_module]
pub mod color {
    use ice_util::minecraft::rtext::Color;

    pub const Black: Color = Color::Black;
    pub const DarkBlue: Color = Color::DarkBlue;
    pub const DarkGreen: Color = Color::DarkGreen;
    pub const DarkAqua: Color = Color::DarkAqua;
    pub const DarkRed: Color = Color::DarkRed;
    pub const DarkPurple: Color = Color::DarkPurple;
    pub const Gold: Color = Color::Gold;
    pub const Gray: Color = Color::Gray;
    pub const DarkGray: Color = Color::DarkGray;
    pub const Blue: Color = Color::Blue;
    pub const Green: Color = Color::Green;
    pub const Aqua: Color = Color::Aqua;
    pub const Red: Color = Color::Red;
    pub const LightPurple: Color = Color::LightPurple;
    pub const Yellow: Color = Color::Yellow;
    pub const White: Color = Color::White;
}

#[rhai::export_module]
#[allow(unused)]
mod module {
    use ice_util::minecraft::rtext;

    pub type Component = rtext::Component;
    pub type Color = rtext::Color;
}

def_package! {
    pub MinecraftRtextPackage(module) {
        combine_with_exported_module!(module, "minecraft_rtext", module);
    } |> |engine| {
        engine.register_static_module("Color", exported_module!(color).into());

        engine.register_fn("build_component", rtext::build_component::<String>)
            .register_fn("build_component", rtext::build_component::<bool>)
            .register_fn("build_component", rtext::build_component::<f64>)
            .register_fn("build_component", rtext::build_component::<ComponentObject>)
            .register_fn("build_component", |arr: Array| {
                let vec = arr.into_iter().map(|v| {
                    if v.is_string() {
                        ComponentObject::text(v.cast::<String>())
                    } else if let Ok(v) = v.as_bool() {
                        ComponentObject::text(v)
                    } else if let Ok(v) = v.as_float() {
                        ComponentObject::text(v)
                    } else if let Ok(v) = v.as_int() {
                        ComponentObject::text(v)
                    } else if let Ok(v) = v.try_cast_result::<ComponentObject>() {
                        v
                    } else {
                        panic!("invalid type")
                    }
                }).collect();
                rtext::build_component::<Vec<ComponentObject>>(vec)
            });

        engine.register_fn("Rtext", rtext::ComponentObject::text::<String>)
            .register_fn("Rtext", rtext::ComponentObject::text::<bool>)
            .register_fn("Rtext", rtext::ComponentObject::text::<f64>)
            .register_fn("Rtext", rtext::ComponentObject::text::<i64>);

        engine.register_fn("color", rtext::ComponentObject::color)
            .register_fn("bold", rtext::ComponentObject::bold)
            .register_fn("italic", rtext::ComponentObject::italic)
            .register_fn("underlined", rtext::ComponentObject::underlined)
            .register_fn("strikethrough", rtext::ComponentObject::strikethrough)
            .register_fn("obfuscated", rtext::ComponentObject::obfuscated);
    }
}
