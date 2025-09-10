use std::fmt;

pub struct FormatFn<F>(F)
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result;

impl<F> FormatFn<F>
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    pub fn new(f: F) -> Self {
        Self(f)
    }
}

impl<F> fmt::Debug for FormatFn<F>
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0)(f)
    }
}

impl<F> fmt::Display for FormatFn<F>
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0)(f)
    }
}
