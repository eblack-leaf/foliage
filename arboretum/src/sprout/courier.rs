//! The mock backend: answers the app already knows, delivered as if from far away.

use foliage::{Grove, Grow, Pollen, Timing, Tween};

use crate::genart::Seed;

/// Which request an answer is to.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ticket(u32);

/// A courier between the page and a backend that is not there.
///
/// Every answer is worked out the moment it is asked for, by an app's own pure `api` -- the part
/// a real backend would replace -- and handed to the courier, which keeps it for a while and
/// delivers it later, as a request across a network would arrive. The wait is jittered from a
/// seed, so a demo's latency has the irregular feel of a real one and is the same on every run.
/// A failure is an answer like any other, an `Err`; nothing here knows the difference.
pub struct Courier<A> {
    next: u32,
    jitter: Seed,
    waiting: Vec<(Ticket, Tween, A)>,
}

impl<A> Courier<A> {
    /// A courier whose waits are jittered from `seed`.
    pub fn new(seed: u64) -> Self {
        Self {
            next: 0,
            jitter: Seed::new(seed),
            waiting: Vec::new(),
        }
    }

    /// Sends `answer` on its way, to arrive in about `ms` -- a third either side.
    pub fn send(&mut self, grove: &mut Grove, ms: u64, answer: A) -> Ticket {
        let ticket = Ticket(self.next);
        self.next += 1;
        let spread = self.jitter.between(0.67, 1.33);
        let wait = ((ms as f32) * spread).max(1.0) as u64;
        let timer = grove.timer(Timing::ms(wait));
        self.waiting.push((ticket, timer, answer));
        ticket
    }

    /// What arrived this frame, in the order it was sent.
    pub fn frame(&mut self, pollen: &Pollen) -> Vec<(Ticket, A)> {
        let mut arrived = Vec::new();
        let mut n = 0;
        while n < self.waiting.len() {
            if pollen.finished(self.waiting[n].1) {
                let (ticket, _, answer) = self.waiting.remove(n);
                arrived.push((ticket, answer));
            } else {
                n += 1;
            }
        }
        arrived
    }

    /// Gives up waiting for `ticket`'s answer.
    pub fn cancel(&mut self, grove: &mut Grove, ticket: Ticket) {
        if let Some(n) = self.waiting.iter().position(|(t, _, _)| *t == ticket) {
            let (_, timer, _) = self.waiting.remove(n);
            grove.stop(timer);
        }
    }

    /// Gives up on everything still on its way.
    pub fn cancel_all(&mut self, grove: &mut Grove) {
        for (_, timer, _) in self.waiting.drain(..) {
            grove.stop(timer);
        }
    }

    /// Whether anything is still on its way.
    pub fn busy(&self) -> bool {
        !self.waiting.is_empty()
    }

    /// Whether `ticket`'s answer is still on its way.
    pub fn waiting(&self, ticket: Ticket) -> bool {
        self.waiting.iter().any(|(t, _, _)| *t == ticket)
    }
}
