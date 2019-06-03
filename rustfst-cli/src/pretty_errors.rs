use failure::Fail;

pub struct ExitFailure(failure::Error);

/// Prints a list of causes for this Error, along with any backtrace
/// information collected by the Error (if RUST_BACKTRACE=1).
impl std::fmt::Debug for ExitFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let fail = self.0.as_fail();

        writeln!(f, "{}", &fail)?;

        let mut x: &Fail = fail;
        while let Some(cause) = x.cause() {
            writeln!(f, " -> caused by: {}", &cause)?;
            x = cause;
        }
        if let Some(backtrace) = x.backtrace() {
            writeln!(f, "{:?}", backtrace)?;
        }

        Ok(())
    }
}

impl<T: Into<failure::Error>> From<T> for ExitFailure {
    fn from(t: T) -> Self {
        ExitFailure(t.into())
    }
}