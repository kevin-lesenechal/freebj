use std::ops::Add;
use bitflags::_core::ops::AddAssign;

#[derive(Debug)]
pub struct RunningStats {
    min: f64,
    max: f64,
    count: usize,
    m: f64,
    s: f64,
}

impl Default for RunningStats {
    fn default() -> Self {
        RunningStats {
            min: f64::NAN,
            max: f64::NAN,
            count: 0,
            m: 0.0,
            s: 0.0,
        }
    }
}

impl RunningStats {
    /// Pushes and updates the running stats with a new value.
    ///
    /// * `value` - The new value to push
    pub fn push(&mut self, value: f64) {
        self.count += 1;

        // Numerically stable mean and variance calculation, see Donald Knuth,
        // "The Art of Computer Programming", vol. 2, p. 232, 3rd edition.
        if self.count == 1 {
            self.m = value;
            self.s = 0.0;
        } else {
            let new_m = self.m + (value - self.m) / (self.count as f64);
            self.s += (value - self.m) * (value - new_m);
            self.m = new_m;
        }

        if self.min.is_nan() || value < self.min {
            self.min = value;
        }
        if self.max.is_nan() || value > self.max {
            self.max = value;
        }
    }

    /// Returns the number of values that where pushed.
    pub fn count(&self) -> usize { self.count }

    /// Returns the lowest value pushed, NaN if none were pushed.
    pub fn min(&self) -> f64 { self.min }

    /// Returns the highest value pushed, NaN if none were pushed.
    pub fn max(&self) -> f64 { self.max }

    /// Returns the mean of all values pushed, NaN if none were pushed.
    pub fn mean(&self) -> f64 { self.m }

    /// Returns the variance of all values pushed, NaN if none were pushed, 0.0
    /// if only one was.
    pub fn variance(&self) -> f64 {
        match self.count {
            0 => f64::NAN,
            1 => 0.0,
            n => self.s / (n - 1) as f64
        }
    }

    /// Returns the standard deviation of all values pushed, NaN if none were
    /// pushed, 0.0 if only one was.
    pub fn stddev(&self) -> f64 {
        self.variance().sqrt()
    }
}

impl Add for RunningStats {
    type Output = RunningStats;

    fn add(self, rhs: Self) -> Self::Output {
        // Numerically stable, see Tony F. Chan, "Updating Formulae and a
        // Pairwise Algorithm for Computing Sample Variances."
        let m = (self.count as f64 * self.m + rhs.count as f64 * rhs.m)
                / (self.count + rhs.count) as f64;
        // Numerically unstable if rhs.m ~= self.m and both are large
        let delta = rhs.m - self.m;
        let s = self.s + rhs.s + delta * delta
            * (self.count * rhs.count) as f64 / (self.count + rhs.count) as f64;

        RunningStats {
            count: self.count + rhs.count,
            min: self.min.min(rhs.min),
            max: self.max.max(rhs.max),
            m,
            s,
        }
    }
}

impl AddAssign for RunningStats {
    fn add_assign(&mut self, rhs: Self) {
        // Numerically unstable if rhs.m ~= self.m and both are large
        let delta = rhs.m - self.m;

        self.min = self.min.min(rhs.min);
        self.max = self.max.max(rhs.max);
        // Numerically stable, see Tony F. Chan, "Updating Formulae and a
        // Pairwise Algorithm for Computing Sample Variances."
        self.m = (self.count as f64 * self.m + rhs.count as f64 * rhs.m)
                 / (self.count + rhs.count) as f64;
        self.s += rhs.s + delta * delta
            * (self.count * rhs.count) as f64 / (self.count + rhs.count) as f64;
        self.count += rhs.count;
    }
}

#[cfg(test)]
mod tests {
    use crate::running_stats::RunningStats;
    use crate::test_utils::assert_f64_eq;

    const VALUES: [f64; 100] = [
        0.4507922233844060, 0.4989752893634920, 0.2277752695003650,
        0.4123462909379520, 0.2608634099327220, 0.7667903131557050,
        0.2829521738624980, 0.7898329692347570, 0.7688003254241100,
        0.0803415244442340, 0.5660270128941850, 0.2277831247913630,
        0.2401538347069750, 0.0121276049228674, 0.0259112659783077,
        0.5495983855945680, 0.1471912812318120, 0.4580352029837580,
        0.9666060606487490, 0.1786156601836170, 0.2233557908371940,
        0.9819644248935740, 0.9228042489577100, 0.2381715207775710,
        0.0230396288784812, 0.1134510814824610, 0.9094194299014320,
        0.0618817817548163, 0.2082918853914680, 0.5456986343314730,
        0.9420427585144190, 0.1017731781552220, 0.3140208815992360,
        0.7262757459436520, 0.2563860452445060, 0.0813806312241444,
        0.5767312668978590, 0.0031346110480915, 0.9419136450737430,
        0.0978592271129655, 0.2441601006793430, 0.5200799463770840,
        0.4861129285804470, 0.3755069534164010, 0.0629322142458583,
        0.4720059951869870, 0.6175725137999360, 0.4778020114487870,
        0.9791417872070990, 0.0498875741617424, 0.1725802766996810,
        0.9043257408626310, 0.6978811638523010, 0.8365977763694370,
        0.8680287105458120, 0.3364556836051190, 0.7516059031831270,
        0.7589503660110920, 0.5033502792539610, 0.3250336060694770,
        0.6218390785673870, 0.2774029853504470, 0.6572999173554310,
        0.9695627365980480, 0.9821433593768460, 0.0350376131560575,
        0.1435948218120560, 0.1111744113655650, 0.6471174345012750,
        0.6050482041405330, 0.2513505499914340, 0.4530157663041030,
        0.4752670312476880, 0.9318441346093180, 0.4405146809518730,
        0.2600848924131810, 0.9319406781650730, 0.1170901144028410,
        0.4383537905966750, 0.7006769443241730, 0.9667015564876570,
        0.2250573965697970, 0.8090146064558700, 0.4748670623177500,
        0.7740150302969210, 0.7032719332950950, 0.0053210847533632,
        0.7170538626877930, 0.3594542399721990, 0.7675298735800510,
        0.1719193945094220, 0.6621933697540800, 0.9757796978666670,
        0.0787437634153985, 0.3786470423075410, 0.3771037496662970,
        0.5022683062647580, 0.6580143153172250, 0.4795733723827800,
        0.7929097706007020,
    ];

    #[test]
    fn it_is_empty() {
        let stats = RunningStats::default();

        assert_eq!(stats.count(), 0);
        assert!(stats.min().is_nan());
        assert!(stats.max().is_nan());
        assert!(stats.variance().is_nan());
        assert!(stats.stddev().is_nan());
    }

    #[test]
    fn it_pushes_one_value() {
        let mut stats = RunningStats::default();

        stats.push(42.0);

        assert_eq!(stats.count(),    1);
        assert_eq!(stats.min(),      42.0);
        assert_eq!(stats.max(),      42.0);
        assert_eq!(stats.variance(), 0.0);
        assert_eq!(stats.stddev(),   0.0);
    }

    #[test]
    fn it_pushes_100_values() {
        let mut stats = RunningStats::default();

        for &value in VALUES.iter() {
            stats.push(value);
        }

        assert_eq!(stats.count(),       100);
        assert_eq!(stats.min(),         0.0031346110480915);
        assert_eq!(stats.max(),         0.9821433593768460);
        assert_f64_eq(stats.mean(),     0.4755092574648416, 10e-16);
        assert_f64_eq(stats.variance(), 0.09138556180391591, 10e-16);
        assert_f64_eq(stats.stddev(),   0.3023004495595663, 10e-16);
    }

    #[test]
    fn it_sums_running_stats() {
        let mut stats1 = RunningStats::default();
        for &value in &VALUES[0..40] { stats1.push(value); }
        let mut stats2 = RunningStats::default();
        for &value in &VALUES[40..70] { stats2.push(value); }
        let mut stats3 = RunningStats::default();
        for &value in &VALUES[70..90] { stats3.push(value); }
        let mut stats4 = RunningStats::default();
        for &value in &VALUES[90..100] { stats4.push(value); }

        let stats = stats1 + stats2 + stats3 + stats4;

        assert_eq!(stats.count(),       100);
        assert_eq!(stats.min(),         0.0031346110480915);
        assert_eq!(stats.max(),         0.9821433593768460);
        assert_f64_eq(stats.mean(),     0.4755092574648416, 10e-16);
        assert_f64_eq(stats.variance(), 0.09138556180391591, 10e-16);
        assert_f64_eq(stats.stddev(),   0.3023004495595663, 10e-16);
    }

    #[test]
    fn it_sums_assign_running_stats() {
        let mut stats = RunningStats::default();
        for &value in &VALUES[0..40] { stats.push(value); }
        let mut stats2 = RunningStats::default();
        for &value in &VALUES[40..70] { stats2.push(value); }
        let mut stats3 = RunningStats::default();
        for &value in &VALUES[70..90] { stats3.push(value); }
        let mut stats4 = RunningStats::default();
        for &value in &VALUES[90..100] { stats4.push(value); }

        stats += stats2;
        stats += stats3;
        stats += stats4;

        assert_eq!(stats.count(),       100);
        assert_eq!(stats.min(),         0.0031346110480915);
        assert_eq!(stats.max(),         0.9821433593768460);
        assert_f64_eq(stats.mean(),     0.4755092574648416, 10e-16);
        assert_f64_eq(stats.variance(), 0.09138556180391591, 10e-16);
        assert_f64_eq(stats.stddev(),   0.3023004495595663, 10e-16);
    }
}
