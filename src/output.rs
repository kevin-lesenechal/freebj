use std::collections::BTreeMap;
use serde::{Serialize, Serializer};
use serde::ser::SerializeMap;
use freebj::game_rules::GameRules;
use freebj::simulator::SimulationResult;

pub struct ProgramResult<'a> {
    pub rounds: u64,
    pub rules: &'a GameRules,
    pub simulation: SimulationResult,
}

struct WinningDistrib<'a> {
    pub distrib: &'a BTreeMap<i32, u64>,
}

impl<'a> WinningDistrib<'a> {
    pub fn new(distrib: &'a BTreeMap<i32, u64>) -> WinningDistrib {
        WinningDistrib {
            distrib,
        }
    }
}

impl Serialize for WinningDistrib<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        let mut map = serializer.serialize_map(None)?;

        for (&k, v) in self.distrib.iter() {
            map.serialize_entry(&format!("{:+.1}", k as f64 / 2.0), v)?;
        }

        map.end()
    }
}

impl Serialize for ProgramResult<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("rounds", &self.rounds)?;
        map.serialize_entry("rules", self.rules)?;
        map.serialize_entry("ev", &self.simulation.winnings.mean())?;
        map.serialize_entry("stddev", &self.simulation.winnings.stddev())?;
        let distrib = WinningDistrib::new(&self.simulation.winning_distrib);
        map.serialize_entry("winning_distrib", &distrib)?;
        map.serialize_entry("hands", &self.simulation.hand_stats)?;

        map.end()
    }
}
