pub(crate) fn float_approx_equal(w1: f32, w2: f32, delta: f32) -> bool {
    (w1 - w2).abs() <= delta
}
