use crate::Label;

/// This specifies what labels to output on the call or return transition.
#[derive(PartialOrd, PartialEq, Copy, Clone, Debug, Eq)]
pub enum ReplaceLabelType {
    /// Epsilon labels on both input and output.
    Neither,
    /// Non-epsilon labels on input and epsilon on output.
    Input,
    /// Epsilon on input and non-epsilon on output.
    Output,
    #[allow(unused)]
    /// Non-epsilon labels on both input and output.
    Both,
}

#[derive(PartialOrd, PartialEq, Clone, Debug, Eq)]
pub struct ReplaceFstOptions {
    /// Index of root rule for expansion.
    pub root: Label,
    /// How to label call transition.
    pub call_label_type: ReplaceLabelType,
    /// How to label return transition.
    pub return_label_type: ReplaceLabelType,
    /// Specifies output label to put on call transition; if `None`, use existing label
    /// on call transition. Otherwise, use this field as the output label.
    pub call_output_label: Option<Label>,
    /// Specifies label to put on return transition.
    pub return_label: Label,
}

impl ReplaceFstOptions {
    pub fn new(root: Label, epsilon_on_replace: bool) -> Self {
        Self {
            root,
            call_label_type: if epsilon_on_replace {
                ReplaceLabelType::Neither
            } else {
                ReplaceLabelType::Input
            },
            return_label_type: ReplaceLabelType::Neither,
            call_output_label: if epsilon_on_replace { Some(0) } else { None },
            return_label: 0,
        }
    }
}
