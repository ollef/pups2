use std::collections::BinaryHeap;

pub struct Scheduler {
    pub cycle: u64,
    pending: BinaryHeap<PendingEvent>,
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum Event {
    Run(u64),
    VBlankStart,
    GsVBlank,
    VBlankEnd,
}

#[derive(Eq, PartialEq, Debug)]
struct PendingEvent {
    event: Event,
    period: Option<u64>,
    cycle: u64,
}

impl Scheduler {
    const CYCLES_PER_FRAME: u64 = 4920115;
    const VBLANK_START_CYCLE: u64 = 4489019;
    const GS_VBLANK_DELAY: u64 = 65622;

    pub fn new() -> Self {
        let mut pending = BinaryHeap::new();
        pending.push(PendingEvent {
            event: Event::VBlankStart,
            period: Some(Scheduler::CYCLES_PER_FRAME),
            cycle: Self::VBLANK_START_CYCLE,
        });
        pending.push(PendingEvent {
            event: Event::GsVBlank,
            period: Some(Scheduler::CYCLES_PER_FRAME),
            cycle: Self::VBLANK_START_CYCLE + Self::GS_VBLANK_DELAY,
        });
        pending.push(PendingEvent {
            event: Event::VBlankEnd,
            period: Some(Scheduler::CYCLES_PER_FRAME),
            cycle: Self::CYCLES_PER_FRAME,
        });
        Self { cycle: 0, pending }
    }

    pub fn next_event(&mut self) -> Event {
        let next_event_cycle = self.pending.peek().unwrap().cycle;
        if next_event_cycle <= self.cycle {
            let event = self.pending.pop().unwrap();
            if let Some(period) = event.period {
                self.pending.push(PendingEvent {
                    event: event.event,
                    period: Some(period),
                    cycle: event.cycle + period,
                });
            }
            event.event
        } else {
            let run_cycles = (next_event_cycle - self.cycle).min(32);
            Event::Run(run_cycles)
        }
    }

    pub fn tick(&mut self, cycles: u64) {
        self.cycle += cycles;
    }
}

impl Ord for PendingEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cycle.cmp(&self.cycle)
    }
}

impl PartialOrd for PendingEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
