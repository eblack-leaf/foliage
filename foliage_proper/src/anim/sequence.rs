use crate::time::TimeDelta;
use crate::Component;

#[derive(Copy, Clone, Default)]
pub(crate) struct SequenceTimeRange {
    pub(crate) start: TimeDelta,
    pub(crate) finish: TimeDelta,
}

#[derive(Component, Default, Copy, Clone)]
pub(crate) struct Sequence {
    pub(crate) animations_to_finish: i32,
}
pub(crate) struct AnimationTime {
    accumulated_time: TimeDelta, // use these two to get linear % => use Bézier curve 0-1 to get actual %
    total_time: TimeDelta,
    pub(crate) delay: TimeDelta,
}

impl AnimationTime {
    pub(crate) fn time_delta(&mut self, fd: TimeDelta) -> f32 {
        self.accumulated_time += fd;
        let delta = self.accumulated_time.as_millis() as f32 / self.total_time.as_millis() as f32;
        delta.clamp(0.0, 1.0)
    }
}

impl From<SequenceTimeRange> for AnimationTime {
    fn from(value: SequenceTimeRange) -> Self {
        AnimationTime {
            accumulated_time: Default::default(),
            total_time: value.finish - value.start,
            delay: value.start,
        }
    }
}
