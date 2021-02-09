use crate::simulator::{Simulator, SimulationResult};
use crate::shoe::CardShoe;
use crate::round_factory::RoundFactory;

pub struct SmpSimulator<'a> {
    round_count: u64,
    round_factory: RoundFactory<'a>,
    shoe_factory: Box<dyn Fn() -> Box<dyn CardShoe + Send>>,
    force_tc: Option<f32>,
    adjust_rc: Option<i32>,
    num_threads: u32,
    verbose: bool,
}

impl<'a> SmpSimulator<'a> {
    pub fn new(round_count: u64,
               round_factory: RoundFactory<'a>,
               shoe_factory: Box<dyn Fn() -> Box<dyn CardShoe + Send>>,
               force_tc: Option<f32>,
               adjust_rc: Option<i32>,
               num_threads: u32,
               verbose: bool) -> SmpSimulator {
        SmpSimulator {
            round_count,
            round_factory,
            shoe_factory,
            force_tc,
            adjust_rc,
            num_threads,
            verbose,
        }
    }

    pub fn run(self) -> SimulationResult {
        let per_thread = self.round_count / self.num_threads as u64;
        let rest = self.round_count % self.num_threads as u64;

        let mut result = SimulationResult::default();

        crossbeam::scope(|scope| {
            let mut threads = Vec::new();

            for i in 0..self.num_threads {
                let shoe = (self.shoe_factory)();
                let round_factory = &self.round_factory;
                let force_tc = self.force_tc;
                let adjust_rc = self.adjust_rc;
                let verbose = self.verbose;

                threads.push(scope.spawn(move |_| {
                    let simulator = Simulator::new(
                        per_thread + (if i == 0 { rest } else { 0 }),
                        shoe,
                        round_factory,
                        force_tc,
                        adjust_rc,
                        verbose,
                        i == 0,
                    );
                    simulator.run()
                }));
            }

            for thread in threads {
                result += thread.join().unwrap();
            }
        }).unwrap();

        result
    }
}
