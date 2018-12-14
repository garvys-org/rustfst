#[derive(Debug, Clone, PartialEq)]
pub struct DrawingConfig {
    pub vertical: bool,
    pub width: f32,
    pub height: f32,
    pub title: String,
    pub portrait: bool,
    pub ranksep: f32,
    pub nodesep: f32,
    pub fontsize: u32,
    pub acceptor: bool,
    pub show_weight_one: bool,
}

impl Default for DrawingConfig {
    fn default() -> Self {
        Self {
            vertical: false,
            width: 8.5,
            height: 11.0,
            title: "".to_string(),
            portrait: false,
            ranksep: 0.40,
            nodesep: 0.25,
            fontsize: 14,
            acceptor: false,
            show_weight_one: true,
        }
    }
}
