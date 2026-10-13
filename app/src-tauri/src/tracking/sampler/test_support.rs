use super::*;
use crate::platform::ActivityProbe;
use crate::time::test_support::FakeClock;
use crate::tracking::test_support::{FakeProbe, RecordingSink};

pub const T0: i64 = 1_800_000_000;

pub struct Rig {
    pub sampler: Sampler,
    pub probe: Arc<FakeProbe>,
    pub clock: Arc<FakeClock>,
    pub sink: Arc<RecordingSink>,
    pub store: Arc<Store>,
}

pub fn rig() -> Rig {
    let probe = Arc::new(FakeProbe::new());
    let clock = Arc::new(FakeClock::at(T0));
    let sink = Arc::new(RecordingSink::default());
    let store = Arc::new(Store::open_in_memory().unwrap());
    let observer = Observer::new(Arc::clone(&probe) as Arc<dyn ActivityProbe>);
    let sampler = Sampler::new(
        Arc::clone(&store),
        observer,
        Arc::clone(&clock) as Arc<dyn Clock>,
        Arc::clone(&sink) as Arc<dyn SampleSink>,
    );
    Rig {
        sampler,
        probe,
        clock,
        sink,
        store,
    }
}

impl Rig {
    pub fn tick_every_two_seconds(&mut self, times: usize) {
        for _ in 0..times {
            self.sampler.tick();
            self.clock.advance(2);
        }
    }

    pub fn rows(&self) -> Vec<(i64, i64, String, Option<String>)> {
        self.store
            .with_conn(|c| {
                let mut s = c.prepare(
                    "SELECT started_at, ended_at, app_id, domain FROM sessions ORDER BY id",
                )?;
                let rows = s.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
                Ok(rows.collect::<Result<_, _>>()?)
            })
            .unwrap()
    }
}
