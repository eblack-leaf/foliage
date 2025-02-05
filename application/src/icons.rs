#[derive(Copy, Clone)]
pub(crate) enum IconHandles {
    Github,
    Terminal,
    Layers,
    BookOpen,
    Code,
    Box,
    ArrowUp,
    X,
    Menu,
    Play,
    Shuffle,
    Repeat,
    SkipLeft,
    SkipRight,
}
impl IconHandles {
    pub(crate) fn value(self) -> i32 {
        self as i32
    }
}
