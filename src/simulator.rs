use std::ops::AddAssign;
use std::collections::BTreeMap;

use crate::hand_stats::HandStats;
use crate::shoe::CardShoe;
use crate::round_factory::RoundFactory;
use crate::running_stats::RunningStats;

pub struct Simulator<'a>
{
    round_count: u64,
    shoe: Box<dyn CardShoe>,
    round_factory: &'a RoundFactory<'a>,
    force_tc: Option<f32>,
    adjust_rc: Option<i32>,
    verbose: bool,
    print_progress: bool,
}

#[derive(Debug, Default)]
pub struct SimulationResult {
    pub winnings: RunningStats,
    pub hand_stats: HandStats,
    pub winning_distrib: BTreeMap<i32, u64>,
}

impl AddAssign for SimulationResult {
    fn add_assign(&mut self, rhs: Self) {
        self.winnings += rhs.winnings;
        self.hand_stats += rhs.hand_stats;

        for (&k, &v) in rhs.winning_distrib.iter() {
            *self.winning_distrib.entry(k).or_insert(0) += v;
        }
    }
}

impl<'a> Simulator<'a>
{
    pub fn new(round_count: u64,
               shoe: Box<dyn CardShoe>,
               round_factory: &'a RoundFactory<'a>,
               force_tc: Option<f32>,
               adjust_rc: Option<i32>,
               verbose: bool,
               print_progress: bool) -> Simulator<'a> {
        Simulator {
            round_count,
            shoe,
            round_factory,
            force_tc,
            adjust_rc,
            verbose,
            print_progress,
        }
    }

    pub fn run(mut self) -> SimulationResult {
        let mut winnings = RunningStats::default();
        let mut hand_stats = HandStats::default();
        let mut winning_distrib = BTreeMap::new();

        for round_i in 0..self.round_count {
            if let Some(force_tc) = self.force_tc {
                self.shoe.force_true_count(force_tc);
            }
            let rc = self.shoe.running_count();
            let tc = self.shoe.true_count();

            let (_, result) = self.round_factory.make(&mut *self.shoe).run();

            if let Some(rel_rc) = self.adjust_rc {
                self.shoe.adjust_running_count(rel_rc);
            }

            let num_result = result.player_results[0];
            winnings.push(num_result);
            hand_stats += result.hand_stats;

            let hash_key = (num_result * 2.0).round() as i32;
            *winning_distrib.entry(hash_key).or_insert(0) += 1;

            if self.print_progress {
                Self::update_progress(round_i + 1, self.round_count);
            }
            if self.verbose {
                eprintln!("rc = {:+}, tc = {:+.1}", rc, tc);
                //eprintln!("{:?}", round);
                eprintln!("res = {:+.1}\n", num_result);
            }
        }

        SimulationResult {
            winnings,
            hand_stats,
            winning_distrib,
        }
    }

    fn update_progress(done: u64, total: u64) {
        if total < 100 || done % (total / 100) == 0 {
            let percent = (done as f64 / total as f64 * 100.0).round() as u32;

            if percent == 100 {
                eprintln!("100%");
            } else if percent % 5 == 0 {
                eprint!("{}%", percent);
            } else {
                eprint!(".");
            }
        } else if done == total {
            eprintln!("100%");
        }
    }
}
