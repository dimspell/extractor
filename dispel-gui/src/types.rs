use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteMode {
    Sprite,
    Animation,
}

impl SpriteMode {
    pub const ALL: &'static [SpriteMode] = &[SpriteMode::Sprite, SpriteMode::Animation];
}

impl fmt::Display for SpriteMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpriteMode::Sprite => write!(f, "Sprite"),
            SpriteMode::Animation => write!(f, "Animation"),
        }
    }
}
