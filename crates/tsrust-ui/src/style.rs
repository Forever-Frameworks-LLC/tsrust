#[derive(Debug, Clone, Copy)]
pub(crate) enum Utility {
    Row,
    Column,
    Gap(f32),
    Padding(f32),
    FontSize(f32),
    Bold,
    Background([u8; 3]),
    Foreground([u8; 3]),
    Rounded(f32),
}
impl Utility {
    pub(crate) fn parse(token: &str) -> Option<Self> {
        Some(match token {
            "flex-row" => Self::Row,
            "flex-col" => Self::Column,
            "gap-2" => Self::Gap(8.),
            "gap-4" => Self::Gap(16.),
            "p-2" => Self::Padding(8.),
            "p-4" => Self::Padding(16.),
            "text-base" => Self::FontSize(16.),
            "text-xl" => Self::FontSize(24.),
            "font-bold" => Self::Bold,
            "bg-background" => Self::Background([255, 255, 255]),
            "bg-primary" => Self::Background([37, 99, 235]),
            "text-foreground" => Self::Foreground([23, 32, 51]),
            "text-primary-foreground" => Self::Foreground([255, 255, 255]),
            "rounded" => Self::Rounded(8.),
            _ => return None,
        })
    }
    pub(crate) fn css(self) -> String {
        match self {
            Self::Row => "flex-direction:row".into(),
            Self::Column => "flex-direction:column".into(),
            Self::Gap(v) => format!("gap:{v}px"),
            Self::Padding(v) => format!("padding:{v}px"),
            Self::FontSize(v) => format!("font-size:{v}px"),
            Self::Bold => "font-weight:700".into(),
            Self::Background([r, g, b]) => format!("background:#{r:02x}{g:02x}{b:02x}"),
            Self::Foreground([r, g, b]) => format!("color:#{r:02x}{g:02x}{b:02x}"),
            Self::Rounded(v) => format!("border-radius:{v}px"),
        }
    }
    pub(crate) fn category(self) -> u8 {
        match self {
            Self::Row | Self::Column => 0,
            Self::Gap(_) => 1,
            Self::Padding(_) => 2,
            Self::FontSize(_) => 3,
            Self::Bold => 4,
            Self::Background(_) => 5,
            Self::Foreground(_) => 6,
            Self::Rounded(_) => 7,
        }
    }
}
