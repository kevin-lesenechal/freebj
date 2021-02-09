pub trait BettingStrategy {
    fn place_bet(&self, true_count: f32) -> f64;
}

pub struct FixedBet(pub f64);

impl BettingStrategy for FixedBet {
    fn place_bet(&self, _true_count: f32) -> f64 { self.0 }
}

pub struct HiloBetting {
    base: f64,
    per_tc: f64,
    neg_null_bet: Option<f64>,
    max_tc: Option<f32>,
    wongout_under: Option<f32>,
}

impl HiloBetting {
    pub fn new(base: f64,
               per_tc: f64,
               neg_null_bet: Option<f64>,
               max_tc: Option<f32>,
               wongout_under: Option<f32>) -> HiloBetting {
        HiloBetting {
            base,
            per_tc,
            neg_null_bet,
            max_tc,
            wongout_under,
        }
    }
}

impl BettingStrategy for HiloBetting {
    fn place_bet(&self, mut true_count: f32) -> f64 {
        true_count = true_count.round();

        if self.wongout_under.is_some()
            && true_count <= self.wongout_under.unwrap() {
            return 0.0;
        } else if self.neg_null_bet.is_some() && true_count <= 0.0 {
            return self.neg_null_bet.unwrap();
        }

        if let Some(max) = self.max_tc {
            true_count = true_count.min(max);
        }

        let bet = self.base + true_count as f64 * self.per_tc;

        bet.max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::betting::{FixedBet, BettingStrategy, HiloBetting};

    #[test]
    fn it_uses_a_fixed_bet() {
        let betting = FixedBet(5.0);

        assert_eq!(betting.place_bet(0.0), 5.0);
        assert_eq!(betting.place_bet(3.0), 5.0);
        assert_eq!(betting.place_bet(-1.0), 5.0);
    }

    #[test]
    fn it_uses_hilo_betting() {
        let betting = HiloBetting::new(7.0, 2.0, None, None, None);

        assert_eq!(betting.place_bet(-5.0), 0.0);
        assert_eq!(betting.place_bet(-1.0), 5.0);
        assert_eq!(betting.place_bet(0.0),  7.0);
        assert_eq!(betting.place_bet(1.0),  9.0);
        assert_eq!(betting.place_bet(2.0),  11.0);
        assert_eq!(betting.place_bet(10.0), 27.0);
    }

    #[test]
    fn it_uses_a_specific_bet_on_negative_or_zero_counts() {
        let betting = HiloBetting::new(10.0, 2.0, Some(5.0), None, None);

        assert_eq!(betting.place_bet(-5.0), 5.0);
        assert_eq!(betting.place_bet(-1.0), 5.0);
        assert_eq!(betting.place_bet(0.0),  5.0);
        assert_eq!(betting.place_bet(1.0),  12.0);
        assert_eq!(betting.place_bet(2.0),  14.0);
        assert_eq!(betting.place_bet(10.0), 30.0);
    }

    #[test]
    fn it_stops_increasing_on_a_maximum_tc() {
        let betting = HiloBetting::new(10.0, 2.0, None, Some(5.0), None);

        assert_eq!(betting.place_bet(-5.0), 0.0);
        assert_eq!(betting.place_bet(-1.0), 8.0);
        assert_eq!(betting.place_bet(0.0),  10.0);
        assert_eq!(betting.place_bet(1.0),  12.0);
        assert_eq!(betting.place_bet(2.0),  14.0);
        assert_eq!(betting.place_bet(10.0), 20.0);
    }

    #[test]
    fn it_wongouts_under_a_specific_tc() {
        let betting = HiloBetting::new(10.0, 2.0, None, None, Some(-4.0));

        assert_eq!(betting.place_bet(-5.0), 0.0);
        assert_eq!(betting.place_bet(-4.0), 0.0);
        assert_eq!(betting.place_bet(-3.0), 4.0);
        assert_eq!(betting.place_bet(-1.0), 8.0);
        assert_eq!(betting.place_bet(0.0),  10.0);
        assert_eq!(betting.place_bet(1.0),  12.0);
        assert_eq!(betting.place_bet(2.0),  14.0);
        assert_eq!(betting.place_bet(10.0), 30.0);
    }

    #[test]
    fn it_uses_hilo_betting_with_all_options() {
        let betting = HiloBetting::new(
            0.0, 10.0, Some(5.0), Some(5.0), Some(-5.0)
        );

        assert_eq!(betting.place_bet(-5.0), 0.0);
        assert_eq!(betting.place_bet(-4.0), 5.0);
        assert_eq!(betting.place_bet(-3.0), 5.0);
        assert_eq!(betting.place_bet(-1.0), 5.0);
        assert_eq!(betting.place_bet(0.0),  5.0);
        assert_eq!(betting.place_bet(1.0),  10.0);
        assert_eq!(betting.place_bet(2.0),  20.0);
        assert_eq!(betting.place_bet(5.0),  50.0);
        assert_eq!(betting.place_bet(6.0),  50.0);
        assert_eq!(betting.place_bet(10.0), 50.0);
    }
}
