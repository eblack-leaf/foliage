#[derive(Clone, Default)]
pub struct Interpolations {
    pub(crate) scalars: Vec<Interpolation>,
}

impl Interpolations {
    pub fn new() -> Self {
        Self { scalars: vec![] }
    }
    pub fn with(mut self, s: f32, e: f32) -> Self {
        self.scalars.push(Interpolation::new(s, e));
        self
    }
    pub fn read(&mut self, i: usize) -> Option<f32> {
        self.scalars.get_mut(i)?.current_value()
    }
    pub fn read_percent(&mut self, i: usize) -> Option<f32> {
        self.scalars.get_mut(i)?.percent()
    }
}

#[derive(Copy, Clone)]
pub struct Interpolation {
    pub(crate) start: f32,
    pub(crate) finish: f32,
    pub(crate) diff: f32,
    pub(crate) current_value: Option<f32>,
}

impl Interpolation {
    pub fn new(s: f32, e: f32) -> Self {
        Self {
            start: s,
            finish: e,
            diff: e - s,
            current_value: None,
        }
    }
    pub fn current_value(&mut self) -> Option<f32> {
        self.current_value.take()
    }
    pub fn percent(&self) -> Option<f32> {
        self.current_value.and_then(|v| Option::from(v / self.diff))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_computes_diff_as_finish_minus_start() {
        let i = Interpolation::new(10.0, 30.0);
        assert_eq!(i.diff, 20.0);
        let i = Interpolation::new(30.0, 10.0);
        assert_eq!(i.diff, -20.0, "diff is signed -- a decreasing interpolation has a negative diff");
    }

    #[test]
    fn current_value_is_none_until_something_writes_it() {
        let mut i = Interpolation::new(0.0, 10.0);
        assert_eq!(i.current_value(), None);
    }

    #[test]
    fn current_value_is_consumed_by_reading_it_once() {
        // `current_value()` is a `.take()` -- it's a one-shot mailbox (the runner writes a
        // fresh value once per tick), not a durable getter. A second read without another
        // write in between must come back empty, or a slow/duplicate reader would see a
        // stale value repeated instead of silently having nothing new to report.
        let mut i = Interpolation::new(0.0, 10.0);
        i.current_value = Some(5.0);
        assert_eq!(i.current_value(), Some(5.0));
        assert_eq!(i.current_value(), None, "the second read must not see the same value again");
    }

    #[test]
    fn percent_divides_the_current_value_by_diff_without_consuming_it() {
        let mut i = Interpolation::new(0.0, 10.0);
        i.current_value = Some(2.5);
        assert_eq!(i.percent(), Some(0.25));
        // unlike `current_value()`, `percent()` takes `&self` -- it must be safe to call
        // repeatedly, and must not interfere with a later `current_value()` read.
        assert_eq!(i.percent(), Some(0.25));
        assert_eq!(i.current_value(), Some(2.5), "percent() must not have consumed it");
    }

    #[test]
    fn percent_is_none_when_nothing_has_been_written_yet() {
        let i = Interpolation::new(0.0, 10.0);
        assert_eq!(i.percent(), None);
    }

    #[test]
    fn interpolations_reads_are_indexed_and_out_of_bounds_is_none_not_a_panic() {
        let mut interps = Interpolations::new().with(0.0, 10.0).with(0.0, 100.0);
        interps.scalars[0].current_value = Some(5.0);
        interps.scalars[1].current_value = Some(50.0);

        assert_eq!(interps.read(0), Some(5.0));
        assert_eq!(interps.read(1), Some(50.0));
        assert_eq!(interps.read(2), None, "no third scalar was ever added");
    }

    #[test]
    fn interpolations_read_percent_matches_the_underlying_scalars_percent() {
        let mut interps = Interpolations::new().with(0.0, 4.0);
        interps.scalars[0].current_value = Some(1.0);
        assert_eq!(interps.read_percent(0), Some(0.25));
    }
}
