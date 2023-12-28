#[derive(Default)]
pub(crate) struct HandleGenerator {
    segment: i32,
    holes: Vec<i32>,
}
impl HandleGenerator {
    pub(crate) fn generate(&mut self) -> i32 {
        
        if !self.holes.is_empty() {
            self.holes.pop().unwrap()
        } else {
            let h = self.segment;
            self.segment += 1;
            h
        }
    }
    #[allow(unused)]
    pub(crate) fn release(&mut self, handle: i32) {
        self.holes.push(handle);
    }
}
