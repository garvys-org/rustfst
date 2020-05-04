/// Struct to configure how the FST should be drawn.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawingConfig {
    /// Draw bottom-to-top instead of left-to-right.
    pub vertical: bool,
    /// Set pair (width, height)
    pub size: Option<(f32, f32)>,
    /// Set figure title.
    pub title: String,
    /// Portrait mode (def: landscape).
    pub portrait: bool,
    /// Set minimum separation between ranks (see dot documentation).
    pub ranksep: Option<f32>,
    /// Set minimum separation between nodes (see dot documentation).
    pub nodesep: Option<f32>,
    /// Set fontsize.
    pub fontsize: u32,
    /// Input in acceptor format.
    pub acceptor: bool,
    /// Print/draw transition weights and final weights equal to Weight::ONE.
    pub show_weight_one: bool,
    /// Print/draw transition weights and final weights.
    pub print_weight: bool,
}

impl Default for DrawingConfig {
    fn default() -> Self {
        Self {
            vertical: false,
            size: None,
            title: "".to_string(),
            portrait: false,
            ranksep: None,
            nodesep: None,
            fontsize: 14,
            acceptor: false,
            show_weight_one: true,
            print_weight: true,
        }
    }
}
