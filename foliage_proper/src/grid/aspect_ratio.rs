pub struct AspectRatio {
    pub sm: Option<f32>,
    pub md: Option<f32>,
    pub lg: Option<f32>,
    pub xl: Option<f32>,
}
impl AspectRatio {
    pub fn new() -> Self {
        Self {
            sm: None,
            md: None,
            lg: None,
            xl: None,
        }
    }
    pub fn sm(mut self, sm: f32) -> Self {
        self.sm = Some(sm);
        self
    }
    pub fn md(mut self, md: f32) -> Self {
        self.md = Some(md);
        self
    }
    pub fn lg(mut self, lg: f32) -> Self {
        self.lg = Some(lg);
        self
    }
    pub fn xl(mut self, xl: f32) -> Self {
        self.xl = Some(xl);
        self
    }
}
