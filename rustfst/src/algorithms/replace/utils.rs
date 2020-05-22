use crate::algorithms::replace::config::ReplaceLabelType;
use crate::Label;

/// Returns true if label type on transition results in epsilon input label.
pub fn epsilon_on_input(label_type: ReplaceLabelType) -> bool {
    label_type == ReplaceLabelType::Neither || label_type == ReplaceLabelType::Output
}

/// Returns true if label type on transition results in epsilon input label.
pub fn epsilon_on_output(label_type: ReplaceLabelType) -> bool {
    label_type == ReplaceLabelType::Neither || label_type == ReplaceLabelType::Input
}

#[allow(unused)]
// Necessary when setting the properties.
pub fn replace_transducer(
    call_label_type: ReplaceLabelType,
    return_label_type: ReplaceLabelType,
    call_output_label: Option<Label>,
) -> bool {
    call_label_type == ReplaceLabelType::Input
        || call_label_type == ReplaceLabelType::Output
        || (call_label_type == ReplaceLabelType::Both && call_output_label.is_some())
        || return_label_type == ReplaceLabelType::Input
        || return_label_type == ReplaceLabelType::Output
}
