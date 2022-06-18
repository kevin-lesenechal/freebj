use freebj::card::Card;
use freebj::game_rules::{SurrenderPolicy, DoublePolicy, GameType, Soft17};
use freebj::game_rules::SurrenderPolicy::NoSurrender;
use freebj::game_rules::DoublePolicy::AnyTwo;
use std::process::exit;
use clap::{ArgMatches, crate_version};
use freebj::game_rules::GameType::{Ahc, Enhc};
use freebj::game_rules::Soft17::{S17, H17};
use freebj::strategy::Decision;
use std::collections::VecDeque;
use std::convert::TryFrom;
use regex::Regex;
use freebj::deviation::Deviation;
use std::str::FromStr;

#[derive(Debug)]
pub struct Options {
    /// The number of playing rounds to simulate
    pub rounds:         u64,

    /// The number of processing jobs (threads) the simulator will use
    pub jobs:           u32,

    /// Whether to play American holecard (AHC) or European no-holecard (ENHC)
    pub game_type:      GameType,

    /// Whether to hit dealer soft 17, or stand on dealer soft 17
    pub soft17:         Soft17,

    /// The amount of money the player starts with
    pub start_bankroll: u64,

    /// The maximum number of hands a player can have by splitting pairs
    pub max_splits:     u32,

    /// The number of card decks, typically between 1 and 8
    pub decks:          u32,

    /// Hit split aces, determines whether the player is player is allowed to
    /// play hands resulting of an ace pair splitting
    pub play_split_aces: bool,

    /// Double after split, allows player to double split hands
    pub das:            bool,

    pub surrender:      SurrenderPolicy,
    pub double:         DoublePolicy,
    pub pen_cards:      u32,
    pub hilo_counting:  bool,
    pub bet:            f64,
    pub bet_per_tc:     f64,
    pub bet_neg_tc:     Option<f64>,
    pub bet_max_tc:     Option<f32>,
    pub wongout_under:  Option<f32>,
    pub deviations:     bool,
    pub more_devs:      Vec<Deviation>,
    pub force_tc:       Option<f32>,
    pub holecarding:    bool,
    pub start_cards:    Option<VecDeque<Card>>,
    pub dealer_cards:   Option<VecDeque<Card>>,
    pub override_action: Option<Decision>,
    pub surrender_override: Option<bool>,
    pub verbose:        bool,
    pub dry_run:        bool,
    pub shoe_file:      Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            rounds:         1_000_000,
            jobs:           4,
            game_type:      Ahc,
            soft17:         S17,
            start_bankroll: 1_000_00,
            max_splits:     4,
            decks:          6,
            play_split_aces: false,
            das:            false,
            surrender:      NoSurrender,
            double:         AnyTwo,
            pen_cards:      5 * 52,
            hilo_counting:  false,
            bet:            1.0,
            bet_per_tc:     1.0,
            bet_neg_tc:     None,
            bet_max_tc:     None,
            wongout_under:  None,
            deviations:     false,
            more_devs:      Vec::new(),
            force_tc:       None,
            holecarding:    false,
            start_cards:    None,
            dealer_cards:   None,
            override_action: None,
            surrender_override: None,
            verbose:        false,
            dry_run:        false,
            shoe_file:      None,
        }
    }
}

impl Options {
    pub fn from_argv() -> Options {
        let mut options: Options = Default::default();

        let matches = clap::App::new("FreeBJ")
            .version(crate_version!())
            .about("Blackjack game and strategy simulator")
            .author("Kévin Lesénéchal")
            .max_term_width(80)
            .help_message("Print help information.")
            .version_message("Print version information.")
            .arg(
                clap::Arg::with_name("action").short("a")
                    .takes_value(true)
                    .help("Always play ACTION as the first action on each \
                       hand, bypassing strategies. Possible actions: +: hit; \
                       =: stand; D: double; V: split; #: surrender.")
            )
            .arg(
                clap::Arg::with_name("rounds").short("n")
                    .takes_value(true)
                    .help("The number of game rounds to simulate.\n\
                        Accepts 'k', 'M', and 'G' suffixes. Example: \"100M\".")
            )
            .arg(
                clap::Arg::with_name("ahc").long("ahc")
                    .help("Use the American holecard game rule.")
            )
            .arg(
                clap::Arg::with_name("enhc").long("enhc")
                    .help("Use the European no-holecard game rule.")
            )
            .arg(
                clap::Arg::with_name("s17").long("s17")
                    .help("Stand on dealer soft 17 (S17).")
            )
            .arg(
                clap::Arg::with_name("h17").long("h17")
                    .help("Hit on dealer soft 17 (H17).")
            )
            .arg(
                clap::Arg::with_name("das").long("das")
                    .help("Allow doubling down on split hands (double after \
                    split - DAS).")
            )
            .arg(
                clap::Arg::with_name("no_das").long("no-das")
                    .help("Disallow doubling down on split hands (double after \
                    split - DAS).")
            )
            .arg(
                clap::Arg::with_name("jobs").short("j")
                    .takes_value(true)
                    .help("The number of processing jobs, should be equal to \
                    the number of CPUs.")
            )
            .arg(
                clap::Arg::with_name("decks").short("d")
                    .takes_value(true)
                    .help("The number of card decks to play.")
            )
            .arg(
                clap::Arg::with_name("penetration").short("p")
                    .takes_value(true)
                    .help("Set the penetration ratio, i.e. the proportion of \
                    cards to deal from the shoe before shuffling. Can be \
                    represented as a percentage (e.g. \"80%\" for 4/5 \
                    penetration), or as a number of card decks (e.g. \"3d\"), \
                    or a number of cards to deal (e.g. \"100\"), or a ratio \
                    (e.g. \"5/6\"). Default: \"80%\".")
            )
            .arg(
                clap::Arg::with_name("max_splits").long("max-splits")
                    .takes_value(true)
                    .help("The maximum number of hands a player can get from \
                    splitting.")
            )
            .arg(
                clap::Arg::with_name("play_aa").long("playAA")
                    .help("Allow the player to play their hand after \
                    splitting aces.")
            )
            .arg(
                clap::Arg::with_name("no_play_aa").long("no-playAA")
                    .help("Disallow the player to play their hand after \
                    splitting aces, the split hands will receive only one card \
                    and then closed.")
            )
            .arg(
                clap::Arg::with_name("esurr").long("esurr")
                    .help("Allow early surrender.")
            )
            .arg(
                clap::Arg::with_name("lsurr").long("lsurr")
                    .help("Allow late surrender.")
            )
            .arg(
                clap::Arg::with_name("no_surr").long("no-surr")
                    .help("Disallow any form of surrender.")
            )
            .arg(
                clap::Arg::with_name("double_any").long("db-any")
                    .help("Allow doubling down on any hand regardless of the \
                    number of cards.")
            )
            .arg(
                clap::Arg::with_name("double_any2").long("db-any2")
                    .help("Allow doubling down on any hand on the first two \
                    cards only.")
            )
            .arg(
                clap::Arg::with_name("double_hard911").long("db-hard-9-11")
                    .help("Allow doubling down on hard hands with a total of \
                    9, 10, or 11 on the first two cards only.")
            )
            .arg(
                clap::Arg::with_name("double_hard1011").long("db-hard-10-11")
                    .help("Allow doubling down on hard hands with a total of \
                    10 or 11 on the first two cards only.")
            )
            .arg(
                clap::Arg::with_name("double_none").long("db-none")
                    .help("Disallow doubling down on all hands.")
            )
            .arg(
                clap::Arg::with_name("holecarding").long("holecarding")
                    .help("Use holecarding strategy where the dealer's \
                    holecard is known to the players. Requires --ahc.")
            )
            .arg(
                clap::Arg::with_name("start_cards").short("c")
                    .takes_value(true)
                    .help("Set the cards each player will start with \
                    separated by commas, there must be at least two cards.\
                    Example: 8,A,10.")
            )
            .arg(
                clap::Arg::with_name("dealer_cards").long("dealer")
                    .takes_value(true)
                    .help("Set the cards the dealer will start with \
                    separated by commas. Example: A.")
            )
            .arg(
                clap::Arg::with_name("bet").long("bet").short("b")
                    .takes_value(true)
                    .help("Specify the base bet; with card counting, this is \
                    the bet placed with a true count of zero.")
            )
            .arg(
                clap::Arg::with_name("bet_per_tc").long("bet-per-tc")
                    .takes_value(true)
                    .help("The amount to increase the bet with for each point \
                    of true count.")
            )
            .arg(
                clap::Arg::with_name("bet_neg_tc").long("bet-neg-tc")
                    .takes_value(true)
                    .help("The bet to place when true count is zero or \
                    negative.")
            )
            .arg(
                clap::Arg::with_name("bet_max_tc").long("bet-max-tc")
                    .takes_value(true)
                    .help("The maximum true count value to take into account \
                    for the betting strategy, TC above won't increase the bet.")
            )
            .arg(
                clap::Arg::with_name("hilo").long("hilo")
                    .help("Count cards using hilo system, this will adapt the \
                    betting strategy but won't enable playing deviations.")
            )
            .arg(
                clap::Arg::with_name("deviations").long("deviations")
                    .help("Enable playing deviations, this requires card \
                    counting.")
            )
            .arg(
                clap::Arg::with_name("add_deviation")
                    .long("add-deviation").short("D")
                    .takes_value(true)
                    .multiple(true)
                    .number_of_values(1)
                    .value_name("DEVIATION")
                    .help("Add a new basic strategy deviation using the \
                    DEVIATION directive; its syntax is \
                    \"<HAND>vs<DEALER>:('<'|'>')<TC><ACTION>\", HAND can \
                    represent a hard total (\"18\"), a soft total (\"A7\"), \
                    or a pair (\"8/8\", \"A/A\", \"T/T\", ...); DEALER is the \
                    dealer's upcard (number or \"A\"); TC is the true count \
                    above/equal ('>') or under/equal ('<') which to apply the \
                    ACTION deviation. Possible actions: +: hit; =: stand; \
                    D: double; V: split; S: surrender. \
                    This option can be repeated to add more deviations; if \
                    --deviations is given, this will override default playing \
                    deviations.\n\
                    Example: \"16vs10:>+1=\" (stand at TC 1 or above with a \
                    hard 16 against a dealer 10).")
            )
            .arg(
                clap::Arg::with_name("force_tc").long("force-tc")
                    .takes_value(true)
                    .help("Force a specific true count value in the shoe for \
                    each round run; this requires reshuffling the shoe before \
                    each round and remove random cards to achieve the desired \
                    true count, causing some performance penalties.")
            )
            .arg(
                clap::Arg::with_name("shoe_file").long("shoe-file")
                    .takes_value(true)
                    .help("Provide a binary file of cards to load into the \
                    card shoe. The file contains bytes from 1 to 10 included.")
            )
            .arg(
                clap::Arg::with_name("dry_run").long("dry-run")
                    .help("Do not perform any actual work; useful to extract \
                    simulation meta information such as game rules.")
            )
            .arg(
                clap::Arg::with_name("verbose").short("v")
                    .help("Print verbose details on each round.")
            )
            .get_matches();

        if let Err(err) = options.hydrate_options(&matches) {
            eprintln!("{}", err);
            eprintln!("{}", matches.usage());
            exit(1);
        }

        options
    }

    fn hydrate_options(&mut self, matches: &ArgMatches) -> Result<(), String> {
        if let Some(action_str) = matches.value_of("action") {
            match action_str {
                "+" => self.override_action = Some(Decision::Hit),
                "=" => self.override_action = Some(Decision::Stand),
                "D" => self.override_action = Some(Decision::Double),
                "V" => self.override_action = Some(Decision::Split),
                "#" => self.surrender_override = Some(true),
                _ => return Err("-a: invalid action".into()),
            }
            if self.override_action.is_some() {
                self.surrender_override = Some(false);
            }
            if self.surrender_override.unwrap_or(false) {
                self.override_action = None;
            }
        }

        if let Some(rounds_str) = matches.value_of("rounds") {
            self.rounds = match parse_suffix_int(rounds_str) {
                Ok(n) if n > 0 => n,
                _ => return Err("--rounds: invalid number of rounds".into())
            };
        }

        if let Some(decks) = matches.value_of("decks") {
            self.decks = match decks.parse() {
                Ok(n) if n > 0 => n,
                _ => return Err("--decks: invalid number of decks".into()),
            };
        }

        if let Some(pen_cards) = matches.value_of("penetration") {
            self.pen_cards = parse_penetration(pen_cards, self.decks)
                .map_err(|_| "-p: invalid penetration")?;
        } else {
            self.pen_cards = parse_penetration("80%", self.decks).unwrap();
        }

        if let Some(jobs) = matches.value_of("jobs") {
            self.jobs = match jobs.parse() {
                Ok(n) if n > 0 => n,
                _ => return Err("--jobs: invalid number of jobs".into()),
            };
        }

        if let Some(max_splits) = matches.value_of("max_splits") {
            self.max_splits = match max_splits.parse() {
                Ok(n) if n > 0 => n,
                _ => return Err("--max-splits: invalid number of hands".into()),
            };
        }

        if matches.is_present("ahc") && matches.is_present("enhc") {
            return Err("--ahc and --enhc are mutually exclusive".into());
        } else if matches.is_present("enhc") {
            self.game_type = Enhc;
        } else if matches.is_present("ahc") {
            self.game_type = Ahc;
        }

        if matches.is_present("s17") && matches.is_present("h17") {
            return Err("--s17 and --h17 are mutually exclusive".into());
        } else if matches.is_present("s17") {
            self.soft17 = S17;
        } else if matches.is_present("h17") {
            self.soft17 = H17;
        }

        if matches.is_present("play_aa") && matches.is_present("no_play_aa") {
            return Err("--playAA and --no-playAA are mutually exclusive".into());
        } else if matches.is_present("play_aa") {
            self.play_split_aces = true;
        } else if matches.is_present("no_play_aa") {
            self.play_split_aces = false;
        }

        if matches.is_present("das") && matches.is_present("no_das") {
            return Err("--das and --no-das are mutually exclusive".into());
        } else if matches.is_present("das") {
            self.das = true;
        } else if matches.is_present("no_das") {
            self.das = false;
        }

        if matches.is_present("double_any") as u32
           + matches.is_present("double_any2") as u32
           + matches.is_present("double_hard911") as u32
           + matches.is_present("double_hard1011") as u32
           + matches.is_present("double_none") as u32 > 1 {
            return Err("--db-any, --db-any2, --db-hard-9-11, --db-hard-10-11, \
            and --db-none are mutually exclusive".into());
        } else if matches.is_present("double_any") {
            self.double = DoublePolicy::AnyHand;
        } else if matches.is_present("double_any2") {
            self.double = DoublePolicy::AnyTwo;
        } else if matches.is_present("double_hard911") {
            self.double = DoublePolicy::Hard9To11;
        } else if matches.is_present("double_hard1011") {
            self.double = DoublePolicy::Hard10To11;
        } else if matches.is_present("double_none") {
            self.double = DoublePolicy::NoDouble;
        }

        if matches.is_present("esurr") as u32
           + matches.is_present("lsurr") as u32
           + matches.is_present("no_surr") as u32 > 1 {
            return Err("--esurr, --lsurr, and --no-surr are mutually \
            exclusive".into());
        } else if matches.is_present("esurr") {
            self.surrender = SurrenderPolicy::EarlySurrender;
        } else if matches.is_present("lsurr") {
            if self.game_type == GameType::Enhc {
                return Err("--lsurr: incompatible with --enhc".into());
            }
            self.surrender = SurrenderPolicy::LateSurrender;
        } else if matches.is_present("no_surr") {
            self.surrender = SurrenderPolicy::NoSurrender;
        }

        if matches.is_present("holecarding") {
            if self.game_type != GameType::Ahc {
                return Err("--holecarding: requires --ahc".into());
            }
            self.holecarding = true;
        }

        if let Some(start_cards) = matches.value_of("start_cards") {
            let cards = parse_card_list(start_cards)
                .map_err(|_| "-c: invalid card list")?;
            if cards.len() < 2 {
                return Err("-c: there must be at least two cards".into());
            }
            self.start_cards = Some(cards);
        }

        if let Some(dealer_cards) = matches.value_of("dealer_cards") {
            let cards = parse_card_list(dealer_cards)
                .map_err(|e| format!("--dealer: invalid card list: {}", e))?;
            self.dealer_cards = Some(cards);
        }

        self.hilo_counting = matches.is_present("hilo");

        if let Some(bet) = matches.value_of("bet") {
            self.bet = match bet.parse() {
                Ok(n) if n > 0.0 => n,
                _ => return Err("--bet: invalid bet".into()),
            };
        }

        if let Some(bet_per_tc) = matches.value_of("bet_per_tc") {
            if !self.hilo_counting {
                return Err("--bet-per-tc: requires card counting".into());
            }
            self.bet_per_tc = match bet_per_tc.parse() {
                Ok(n) if n >= 0.0 => n,
                _ => return Err("--bet-per-tc: invalid increment".into()),
            };
        }

        if let Some(bet_neg_tc) = matches.value_of("bet_neg_tc") {
            if !self.hilo_counting {
                return Err("--bet-neg-tc: requires card counting".into());
            }
            self.bet_neg_tc = match bet_neg_tc.parse() {
                Ok(n) if n >= 0.0 => Some(n),
                _ => return Err("--bet-neg-tc: invalid bet".into()),
            };
        }

        if let Some(bet_max_tc) = matches.value_of("bet_max_tc") {
            if !self.hilo_counting {
                return Err("--bet-max-tc: requires card counting".into());
            }
            self.bet_neg_tc = match bet_max_tc.parse() {
                Ok(n) if n >= 0.0 => Some(n),
                _ => return Err("--bet-max-tc: invalid true count".into()),
            };
        }

        if matches.is_present("deviations") {
            if !self.hilo_counting {
                return Err("--deviations: requires card counting".into());
            }
            self.deviations = true;
        }

        if let Some(iter) = matches.values_of("add_deviation") {
            if !self.hilo_counting {
                return Err("-D, --add-deviation: requires card counting".into());
            }
            for dev in iter {
                self.more_devs.push(
                    Deviation::from_str(dev)
                        .map_err(|e| format!("-D, --add-deviation: {}", e))?
                );
            }
        }

        if let Some(tc) = matches.value_of("force_tc") {
            self.force_tc = Some(tc.parse()
                .map_err(|_| "--force-tc: invalid true count")?);
        }

        self.shoe_file = matches.value_of("shoe_file").map(|s| s.to_string());

        self.dry_run = matches.is_present("dry_run");
        self.verbose = matches.is_present("verbose");

        Ok(())
    }
}

fn parse_suffix_int(str: &str) -> Result<u64, String> {
    let suffix = str.chars().last().ok_or("Empty parameter")?;

    let scale = match suffix {
        'k' => 1_000,
        'M' => 1_000_000,
        'G' => 1_000_000_000,
        '0'..='9' => 1,
        _ => return Err("Unknown suffix".to_string()),
    };

    let str_slice = if scale > 1 { &str[0..str.len() - 1] } else { &str[..] };
    let base = str_slice.parse::<u64>().map_err(|e| e.to_string())?;

    Ok(base * scale)
}

fn parse_card_list(str: &str) -> Result<VecDeque<Card>, &'static str> {
    let mut vec = VecDeque::new();

    for part in str.split(',') {
        vec.push_back(Card::try_from(part)?);
    }

    Ok(vec)
}

fn parse_penetration(arg: &str, decks: u32) -> Result<u32, &'static str> {
    let percent_regex = Regex::new(r"^(\d+)%$").unwrap();
    let ratio_regex = Regex::new(r"^(\d+)/(\d+)$").unwrap();
    let decks_regex = Regex::new(r"^(\d+)d$").unwrap();

    let pen_cards;

    if percent_regex.is_match(arg) {
        let c = percent_regex.captures_iter(arg).next().unwrap();
        let percent: f64 = c[1].parse().map_err(|_| "Invalid penetration")?;
        pen_cards = (percent / 100.0 * (decks as f64 * 52.0)).round() as u32;
    } else if ratio_regex.is_match(arg) {
        let c = ratio_regex.captures_iter(arg).next().unwrap();
        let a: u32 = c[1].parse().map_err(|_| "Invalid penetration")?;
        let b: u32 = c[2].parse().map_err(|_| "Invalid penetration")?;
        if b == 0 {
            return Err("Invalid penetration");
        }
        let ratio = a as f64 / b as f64;
        pen_cards = (ratio * (decks as f64 * 52.0)).round() as u32;
    } else if decks_regex.is_match(arg) {
        let c = decks_regex.captures_iter(arg).next().unwrap();
        pen_cards = c[1].parse::<u32>()
            .map_err(|_| "Invalid penetration")? * 52;
    } else {
        pen_cards = arg.parse().map_err(|_| "Invalid penetration")?;
    }

    if pen_cards > decks * 52 {
        return Err("Penetration cannot exceed 100 %");
    } else if pen_cards == 0 {
        return Err("Invalid penetration");
    }

    Ok(pen_cards)
}

#[cfg(test)]
mod tests {
    use crate::options::{parse_suffix_int, parse_card_list, parse_penetration};
    use std::collections::VecDeque;
    use freebj::card::Card;

    fn make_card_list(cards: &[u8]) -> VecDeque<Card> {
        let mut v = VecDeque::new();
        for &c in cards {
            v.push_back(Card(c));
        }
        v
    }

    #[test]
    fn it_parses_suffix_int() {
        assert_eq!(parse_suffix_int("0"), Ok(0));
        assert_eq!(parse_suffix_int("42"), Ok(42));
        assert_eq!(parse_suffix_int("12k"), Ok(12_000));
        assert_eq!(parse_suffix_int("388M"), Ok(388_000_000));
        assert_eq!(parse_suffix_int("9G"), Ok(9_000_000_000));
    }

    #[test]
    fn it_parses_card_list() {
        assert_eq!(parse_card_list("8,6"),
                   Ok(make_card_list(&[8, 6])));
        assert_eq!(parse_card_list("2"),
                   Ok(make_card_list(&[2])));
        assert_eq!(parse_card_list("A,8,3"),
                   Ok(make_card_list(&[1, 8, 3])));

        assert_eq!(parse_card_list("A,12,3"), Err("Invalid card"));
        assert_eq!(parse_card_list("A,,3"), Err("Invalid card"));
        assert_eq!(parse_card_list("A,8,"), Err("Invalid card"));
        assert_eq!(parse_card_list("pp,8,"), Err("Invalid card"));
        assert_eq!(parse_card_list("10, 8,2"), Err("Invalid card"));
        assert_eq!(parse_card_list(""), Err("Invalid card"));
        assert_eq!(parse_card_list(","), Err("Invalid card"));
    }

    #[test]
    fn it_parses_penetration() {
        assert_eq!(parse_penetration("100", 6), Ok(100));
        assert_eq!(parse_penetration("100", 2), Ok(100));
        assert_eq!(parse_penetration("5/6", 6), Ok(260));
        assert_eq!(parse_penetration("5/6", 3), Ok(130));
        assert_eq!(parse_penetration("100%", 6), Ok(312));
        assert_eq!(parse_penetration("80%", 6), Ok(250));
        assert_eq!(parse_penetration("54%", 3), Ok(84));
        assert_eq!(parse_penetration("4d", 6), Ok(208));
        assert_eq!(parse_penetration("4d", 4), Ok(208));

        assert_eq!(parse_penetration("aaa", 6), Err("Invalid penetration"));
        assert_eq!(parse_penetration("-12", 6), Err("Invalid penetration"));
        assert_eq!(parse_penetration("foo%", 6), Err("Invalid penetration"));
        assert_eq!(parse_penetration("0%", 6), Err("Invalid penetration"));
        assert_eq!(parse_penetration("-50%", 6), Err("Invalid penetration"));
        assert_eq!(parse_penetration("%", 6), Err("Invalid penetration"));
        assert_eq!(parse_penetration("6/a", 6), Err("Invalid penetration"));
        assert_eq!(parse_penetration("/2", 6), Err("Invalid penetration"));
        assert_eq!(parse_penetration("6/0", 6), Err("Invalid penetration"));
        assert_eq!(parse_penetration("-5/6", 6), Err("Invalid penetration"));
        assert_eq!(parse_penetration("53", 1), Err("Penetration cannot exceed 100 %"));
        assert_eq!(parse_penetration("5d", 4), Err("Penetration cannot exceed 100 %"));
        assert_eq!(parse_penetration("101%", 4), Err("Penetration cannot exceed 100 %"));
        assert_eq!(parse_penetration("7/6", 4), Err("Penetration cannot exceed 100 %"));
    }
}
