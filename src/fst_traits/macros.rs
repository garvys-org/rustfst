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
    ($fst:expr, $state_id:expr, $f: expr) => {
        for arc in $fst.arcs_iter($state_id).unwrap() {
            write!(
                $f,
                "{}\t{}\t{}\t{}\t{}\n",
                $state_id, &arc.nextstate, &arc.ilabel, &arc.olabel, &arc.weight
            )?;
        }
    };
}

macro_rules! write_fst {
    ($fst:expr, $f:expr) => {
        if let Some(start_state) = $fst.start() {
            // Firstly print the arcs leaving the start state
            display_single_state!($fst, &start_state, $f);

            // Secondly, print the arcs leaving all the other states
            for state_id in $fst.states_iter() {
                if state_id != start_state {
                    display_single_state!($fst, &state_id, $f);
                }
            }

            // Finally, print the final states with their weight
            for final_state in $fst.final_states_iter() {
                write!(
                    $f,
                    "{}\t{}\n",
                    &final_state.state_id, &final_state.final_weight
                )?;
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
                write_fst!(self, f);
                Ok(())
            }
        }
    };
}
