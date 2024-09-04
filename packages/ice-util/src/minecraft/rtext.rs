use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Different representations of a minecraft text component
///
/// See https://zh.minecraft.wiki/w/%E6%96%87%E6%9C%AC%E7%BB%84%E4%BB%B6
#[derive(Clone)]
pub enum Component {
    Object(ComponentObject),
    ObjectList(Vec<ComponentObject>),
    String(String),
    Bool(bool),
    F64(f64),
}

/// Build a minecraft text component into json string
pub fn build_component<T: Into<Component>>(component: T) -> String {
    match component.into() {
        Component::Object(obj) => serde_json::to_string(&obj).unwrap(),
        Component::ObjectList(obj) => serde_json::to_string(&obj).unwrap(),
        Component::String(obj) => obj.to_string(),
        Component::Bool(obj) => obj.to_string(),
        Component::F64(obj) => obj.to_string(),
    }
}

impl Component {
    pub fn new<T: Into<Component>>(v: T) -> Self {
        v.into()
    }
}

impl From<ComponentObject> for Component {
    fn from(obj: ComponentObject) -> Self {
        Self::Object(obj)
    }
}

impl From<Vec<ComponentObject>> for Component {
    fn from(obj: Vec<ComponentObject>) -> Self {
        Self::ObjectList(obj)
    }
}

impl From<String> for Component {
    fn from(obj: String) -> Self {
        Self::String(obj)
    }
}

impl From<bool> for Component {
    fn from(obj: bool) -> Self {
        Self::Bool(obj)
    }
}

impl From<f64> for Component {
    fn from(obj: f64) -> Self {
        Self::F64(obj)
    }
}

/// Minecraft text component
///
/// See https://zh.minecraft.wiki/w/%E6%96%87%E6%9C%AC%E7%BB%84%E4%BB%B6
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ComponentObject {
    #[serde(flatten)]
    content: ComponentContent,
    #[serde(flatten)]
    style: ComponentStyle,
}

impl ComponentObject {
    /// create a [`ComponentContent::Text`]
    pub fn text<T: Display>(text: T) -> Self {
        Self {
            content: ComponentContent::Text {
                text: text.to_string(),
            },
            style: Default::default(),
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.style.color = Some(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.style.bold = Some(true);
        self
    }

    pub fn italic(mut self) -> Self {
        self.style.italic = Some(true);
        self
    }

    pub fn underlined(mut self) -> Self {
        self.style.underlined = Some(true);
        self
    }

    pub fn strikethrough(mut self) -> Self {
        self.style.strikethrough = Some(true);
        self
    }

    pub fn obfuscated(mut self) -> Self {
        self.style.obfuscated = Some(true);
        self
    }

    pub fn click_event(mut self, event: ClickEvent) -> Self {
        self.style.click_event = Some(event);
        self
    }

    pub fn hover_event(mut self, event: HoverEvent) -> Self {
        self.style.hover_event = Some(event);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ComponentContent {
    Text { text: String },
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    underlined: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strikethrough: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    obfuscated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    click_event: Option<ClickEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hover_event: Option<HoverEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action", content = "value")]
pub enum ClickEvent {
    OpenUrl(String),
    OpenFile(String),
    RunCommand(String),
    SuggestCommand(String),
    ChangePage(String),
    CopyToClipboard(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action", content = "value")]
pub enum HoverEvent {
    ShowText(Box<ComponentObject>),
}

/// Represent the available colors in minecraft
///
/// See [格式化代码 - 中文 Minecraft Wiki](https://zh.minecraft.wiki/w/%E6%A0%BC%E5%BC%8F%E5%8C%96%E4%BB%A3%E7%A0%81)
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    /// #000000
    Black,
    /// #0000AA
    DarkBlue,
    /// #00AA00
    DarkGreen,
    /// #00AAAA
    DarkAqua,
    /// #AA0000
    DarkRed,
    /// #AA00AA
    DarkPurple,
    /// #FFAA00
    Gold,
    /// #AAAAAA
    Gray,
    /// #555555
    DarkGray,
    /// #5555FF
    Blue,
    /// #55FF55
    Green,
    /// #55FFFF
    Aqua,
    /// #FF5555
    Red,
    /// #FF55FF
    LightPurple,
    /// #FFFF55
    Yellow,
    /// #FFFFFF
    White,
    // MinecoinGold,
    // MaterialQuartz,
    // MaterialIron,
    // MaterialNetherite,
    // MaterialRedstone,
    // MaterialCopper,
    // MaterialEmerald,
    // MaterialDiamond,
    // MaterialLapis,
    // MaterialAmethyst,
}

impl Color {
    pub fn encode(&self) -> String {
        match self {
            Self::Black => "§0",
            Self::DarkBlue => "§1",
            Self::DarkGreen => "§2",
            Self::DarkAqua => "§3",
            Self::DarkRed => "§4",
            Self::DarkPurple => "§5",
            Self::Gold => "§6",
            Self::Gray => "§7",
            Self::DarkGray => "§8",
            Self::Blue => "§9",
            Self::Green => "§a",
            Self::Aqua => "§b",
            Self::Red => "§c",
            Self::LightPurple => "§d",
            Self::Yellow => "§e",
            Self::White => "§f",
        }
        .to_string()
    }
}

// #[derive(Debug, Serialize, Deserialize)]
// pub enum Style {
//     Obfuscated,
//     Bold,
//     Stroke,
//     Underline,
//     Italic,
//     Reset,
// }

// impl Style {
//     pub fn encode(&self) -> String {
//         match self {
//             Self::Obfuscated => "§k",
//             Self::Bold => "§l",
//             Self::Stroke => "§m",
//             Self::Underline => "§n",
//             Self::Italic => "§o",
//             Self::Reset => "§r",
//         }
//         .to_string()
//     }
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ser_component() {
        let component = ComponentObject {
            content: ComponentContent::Text {
                text: "hello".to_string(),
            },
            style: ComponentStyle {
                color: Some(Color::Aqua),
                bold: Some(true),
                italic: Some(true),
                underlined: Some(false),
                strikethrough: Some(false),
                obfuscated: Some(false),
                click_event: Some(ClickEvent::CopyToClipboard("content".to_string())),
                hover_event: Some(HoverEvent::ShowText(Box::new(ComponentObject {
                    content: ComponentContent::Text {
                        text: "click to copy to clipboard".to_string(),
                    },
                    style: ComponentStyle {
                        color: Some(Color::Blue),
                        bold: Some(false),
                        italic: Some(false),
                        underlined: Some(true),
                        strikethrough: Some(true),
                        obfuscated: Some(true),
                        click_event: None,
                        hover_event: None,
                    },
                }))),
            },
        };
        let component: Component = component.into();
        let s = build_component(component);
        // let component = serde_json::to_string(&component).unwrap();
        println!("{}", s);
    }
}
