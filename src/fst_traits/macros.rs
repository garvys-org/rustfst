macro_rules! add_or_fst {
    ($semiring:tt, $fst_type:ty) => {
        impl<$semiring: 'static + Semiring> Add for $fst_type {
            type Output = Result<$fst_type>;

            fn add(self, rhs: $fst_type) -> Self::Output {
                concat(&self, &rhs)
            }
        }

        impl<$semiring: 'static + Semiring> BitOr for $fst_type {
            type Output = Result<$fst_type>;

            fn bitor(self, rhs: $fst_type) -> Self::Output {
                union(&self, &rhs)
            }
        }
    };
}

macro_rules! display_single_state {
    ($fst:expr, $state_id:expr, $f: expr, $show_weight_one: expr) => {
        for arc in $fst.arcs_iter($state_id).unwrap() {
            if arc.weight.is_one() && !$show_weight_one {
                writeln!(
                    $f,
                    "{}\t{}\t{}\t{}",
                    $state_id, &arc.nextstate, &arc.ilabel, &arc.olabel
                )?;
            }
            else {
                writeln!(
                    $f,
                    "{}\t{}\t{}\t{}\t{}",
                    $state_id, &arc.nextstate, &arc.ilabel, &arc.olabel, &arc.weight
                )?;
            }
        }
    };
}

macro_rules! write_fst {
    ($fst:expr, $f:expr, $show_weight_one: expr) => {
        if let Some(start_state) = $fst.start() {
            // Firstly print the arcs leaving the start state
            display_single_state!($fst, &start_state, $f, $show_weight_one);

            // Secondly, print the arcs leaving all the other states
            for state_id in $fst.states_iter() {
                if state_id != start_state {
                    display_single_state!($fst, &state_id, $f, $show_weight_one);
                }
            }

            // Finally, print the final states with their weight
            for final_state in $fst.final_states_iter() {
                if final_state.final_weight.is_one() && !$show_weight_one {
                    writeln!(
                        $f,
                        "{}",
                        &final_state.state_id
                    )?;
                }
                else {
                    writeln!(
                        $f,
                        "{}\t{}",
                        &final_state.state_id, &final_state.final_weight
                    )?;
                }
            }
        }
    };
}

macro_rules! display_fst_trait {
    ($semiring:tt, $fst_type:ty) => {
        impl<$semiring: 'static + Semiring> fmt::Display for $fst_type
        where
            $semiring::Type: fmt::Display,
        {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write_fst!(self, f, false);
                Ok(())
            }
        }
    };
}

macro_rules! draw_single_state {
    ($fst:expr, $state_id:expr, $f: expr, $config:expr) => {
        write!($f, "{}", $state_id)?;
        write!($f, " [label = \"{}", $state_id)?;
        if let Some(final_weight) = $fst.final_weight($state_id) {
            if $config.show_weight_one || !final_weight.is_one() {
                write!($f, "/{}", final_weight)?;
            }
            write!($f, "\", shape = doublecircle,")?;
        } else {
            write!($f, "\", shape = circle,")?;
        }

        if $fst.is_start($state_id) {
            write!($f, " style = bold,")?;
        } else {
            write!($f, " style = solid,")?;
        }

        writeln!($f, " fontsize = {}]", $config.fontsize)?;

        for arc in $fst.arcs_iter($state_id).unwrap() {
            write!($f, "\t{} -> {}", $state_id, arc.nextstate)?;
            write!($f, " [label = \"{}", arc.ilabel)?;
            if !$config.acceptor {
                write!($f, ":{}", arc.olabel)?;
            }

            if $config.show_weight_one || !arc.weight.is_one() {
                write!($f, "/{}", arc.weight)?;
            }
            writeln!($f, "\", fontsize = {}];", $config.fontsize)?;
        }
    };
}
