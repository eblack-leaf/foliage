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
