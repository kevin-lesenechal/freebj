extern crate clap;
extern crate regex;

mod options;
mod output;

use crate::options::Options;
use crate::output::ProgramResult;

use freebj::round_factory::RoundFactory;
use freebj::game_rules::{GameRules, SurrenderPolicy};
use freebj::basic_strategy::BasicStrategy;
use freebj::betting::{FixedBet, HiloBetting, BettingStrategy};
use freebj::smp_simulator::SmpSimulator;
use std::process::exit;
use std::path::Path;
use freebj::shoe::CardShoe;
use freebj::shoe::file_shoe::FileShoe;
use freebj::shoe::standard_shoe::StandardShoe;
use std::collections::VecDeque;
use freebj::card::Card;

fn main() {
    let options = Options::from_argv();

    let game_rules = GameRules {
        game_type: options.game_type,
        soft17: options.soft17,
        das: options.das,
        bj_pays: 1.5,
        double_down: options.double,
        surrender: options.surrender,
        play_ace_pairs: options.play_split_aces,
        max_splits: options.max_splits,
        decks: options.decks,
        penetration_cards: options.pen_cards,
    };

    if options.surrender_override.unwrap_or(false)
        && game_rules.surrender == SurrenderPolicy::NoSurrender {
        eprintln!("Unable to always surrender");
        exit(2);
    }

    let mut strategy = BasicStrategy::new(options.hilo_counting);
    if options.deviations {
        strategy.set_default_deviations();
    }
    for dev in options.more_devs {
        strategy.add_deviation(dev);
    }

    let betting: Box<dyn BettingStrategy + Sync>;
    if options.hilo_counting {
        betting = Box::new(HiloBetting::new(
            options.bet,
            options.bet_per_tc,
            options.bet_neg_tc,
            options.bet_max_tc,
            options.wongout_under,
        ));
    } else {
        betting = Box::new(FixedBet(options.bet));
    }

    let adjust_rc = get_rc_adjust(options.hilo_counting,
                                  &options.start_cards,
                                  &options.dealer_cards);

    let round_factory = RoundFactory::new(
        &game_rules,
        &strategy,
        &*betting,
        1,
        options.holecarding,
        options.override_action,
        options.surrender_override,
        options.start_cards.unwrap_or_default(),
        options.dealer_cards.unwrap_or_default(),
    );

    let shoe_factory: Box<dyn Fn() -> Box<dyn CardShoe + Send>>;
    if let Some(shoe_file) = options.shoe_file {
        shoe_factory = Box::new(move || -> Box<dyn CardShoe + Send> {
            Box::new(FileShoe::new(Path::new(&shoe_file)).unwrap())
        });
    } else {
        let num_decks = options.decks;
        let pen_cards = options.pen_cards;
        shoe_factory = Box::new(move || -> Box<dyn CardShoe + Send> {
            Box::new(StandardShoe::shuffled(num_decks, pen_cards))
        });
    }

    let real_num_rounds;
    if options.dry_run {
        real_num_rounds = 0;
    } else {
        real_num_rounds = options.rounds;
    }

    let simulator = SmpSimulator::new(
        real_num_rounds,
        round_factory,
        shoe_factory,
        options.force_tc,
        adjust_rc,
        options.jobs,
        options.verbose,
    );

    let result = ProgramResult {
        rounds: options.rounds,
        rules: &game_rules,
        simulation: simulator.run(),
    };

    let json = serde_json::to_string_pretty(&result).unwrap();
    println!("{}", json);
}

/// Calculate how much the running count (RC) must be adjusted based on start
/// cards for players and dealer.
///
/// # Parameters
///
///  * `hilo_counting` - Whether hi-lo card counting is enabled, if false no
///                      adjustement is required;
///  * `start_cards` - Players' starting cards;
///  * `dealer_cards` - Dealer's starting cards.
///
/// The adjustement counts one for each high card (ace and ten), and minus one
/// for each low card (2 to 6 included). `Some(0)` is never returned.
///
/// FIXME: return `i32` only
fn get_rc_adjust(hilo_counting: bool,
                 start_cards: &Option<VecDeque<Card>>,
                 dealer_cards: &Option<VecDeque<Card>>) -> Option<i32> {
    if hilo_counting {
        let mut rel_rc = 0;
        if let Some(cards) = start_cards {
            for c in cards.iter() {
                if c.0 == 1 || c.0 == 10 {
                    rel_rc += 1;
                } else if c.0 < 7 {
                    rel_rc -= 1;
                }
            }
        }
        if let Some(cards) = dealer_cards {
            for c in cards.iter() {
                if c.0 == 1 || c.0 == 10 {
                    rel_rc += 1;
                } else if c.0 < 7 {
                    rel_rc -= 1;
                }
            }
        }
        if rel_rc != 0 {
            Some(rel_rc)
        } else {
            None
        }
    } else {
        None
    }
}
