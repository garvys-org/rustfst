
/// Struct to configure how the FST will be displayed
#[derive(Debug, Clone, PartialEq)]
pub struct DrawingConfig {
    /// Draw bottom-to-top instead of left-to-right
    pub vertical: bool,
    /// Set width
    pub width: f32,
    /// Set height
    pub height: f32,
    /// Set figure title
    pub title: String,
    /// Portrait mode (def: landscape)
    pub portrait: bool,
    /// Set minimum separation between ranks (see dot documentation)
    pub ranksep: f32,
    /// Set minimum separation between nodes (see dot documentation)
    pub nodesep: f32,
    /// Set fontsize
    pub fontsize: u32,
    /// Input in acceptor format
    pub acceptor: bool,
    /// Print/draw arc weights and final weights equal to Weight::One()
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
