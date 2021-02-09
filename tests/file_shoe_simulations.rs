mod utils;

use utils::bin_runner::run_freebj;
use crate::utils::assert_f64_eq;

#[test]
fn ahc_s17_das_d6() {
    let json = run_freebj(&[
        "-n", "1M", "-j8",
        "--ahc", "--s17", "--das", "-d6",
        "--shoe-file", "rc/shoe_1M",
    ]);

    assert_eq!(json["rounds"].as_u64().unwrap(),    1_000_000);
    assert_f64_eq(json["ev"].as_f64().unwrap(),     -0.00684000, 10e-8);
    assert_f64_eq(json["stddev"].as_f64().unwrap(), 1.15675691, 10e-8);

    let rules = json["rules"].as_object().unwrap();
    assert_eq!(rules["game_type"].as_str().unwrap(), "ahc");
    assert_eq!(rules["soft17"].as_str().unwrap(), "s17");
    assert_eq!(rules["das"].as_bool().unwrap(), true);
    assert_eq!(rules["double_down"].as_str().unwrap(), "any_two");
    assert_eq!(rules["surrender"].as_str().unwrap(), "no_surrender");
    assert_eq!(rules["play_ace_pairs"].as_bool().unwrap(), false);
    assert_eq!(rules["max_splits"].as_u64().unwrap(), 4);
    assert_eq!(rules["decks"].as_u64().unwrap(), 6);

    let hands = json["hands"].as_object().unwrap();
    assert_eq!(hands["total"].as_u64().unwrap(),        1_029_080);
    assert_eq!(hands["won"].as_u64().unwrap(),          446_960);
    assert_eq!(hands["lost"].as_u64().unwrap(),         493_448);
    assert_eq!(hands["push"].as_u64().unwrap(),         88_672);
    assert_eq!(hands["busted"].as_u64().unwrap(),       160_048);
    assert_eq!(hands["blackjack"].as_u64().unwrap(),    47_464);
    assert_eq!(hands["doubled"].as_u64().unwrap(),      104_456);
    assert_eq!(hands["split"].as_u64().unwrap(),        54_920);
    assert_eq!(hands["surrender"].as_u64().unwrap(),    0);
    assert_eq!(hands["insured"].as_u64().unwrap(),      0);

    let distrib = json["winning_distrib"].as_object().unwrap();
    assert_eq!(distrib.len(), 15);
    assert_eq!(distrib["-6.0"].as_u64().unwrap(), 40);
    assert_eq!(distrib["-5.0"].as_u64().unwrap(), 56);
    assert_eq!(distrib["-4.0"].as_u64().unwrap(), 576);
    assert_eq!(distrib["-3.0"].as_u64().unwrap(), 2_120);
    assert_eq!(distrib["-2.0"].as_u64().unwrap(), 42_608);
    assert_eq!(distrib["-1.0"].as_u64().unwrap(), 434_000);
    assert_eq!(distrib["+0.0"].as_u64().unwrap(), 88_280);
    assert_eq!(distrib["+1.0"].as_u64().unwrap(), 325_544);
    assert_eq!(distrib["+1.5"].as_u64().unwrap(), 45_136);
    assert_eq!(distrib["+2.0"].as_u64().unwrap(), 58_112);
    assert_eq!(distrib["+3.0"].as_u64().unwrap(), 2_328);
    assert_eq!(distrib["+4.0"].as_u64().unwrap(), 952);
    assert_eq!(distrib["+5.0"].as_u64().unwrap(), 200);
    assert_eq!(distrib["+6.0"].as_u64().unwrap(), 40);
    assert_eq!(distrib["+7.0"].as_u64().unwrap(), 8);
}

#[test]
fn ahc_s17_das_d6_esurr() {
    let json = run_freebj(&[
        "-n", "1M", "-j8",
        "--ahc", "--s17", "--das", "-d6", "--esurr",
        "--shoe-file", "rc/shoe_1M",
    ]);

    assert_eq!(json["rounds"].as_u64().unwrap(),    1_000_000);
    assert_f64_eq(json["ev"].as_f64().unwrap(),     0.00227600, 10e-8);
    assert_f64_eq(json["stddev"].as_f64().unwrap(), 1.12169429, 10e-8);

    let rules = json["rules"].as_object().unwrap();
    assert_eq!(rules["game_type"].as_str().unwrap(), "ahc");
    assert_eq!(rules["soft17"].as_str().unwrap(), "s17");
    assert_eq!(rules["das"].as_bool().unwrap(), true);
    assert_eq!(rules["double_down"].as_str().unwrap(), "any_two");
    assert_eq!(rules["surrender"].as_str().unwrap(), "early_surrender");
    assert_eq!(rules["play_ace_pairs"].as_bool().unwrap(), false);
    assert_eq!(rules["max_splits"].as_u64().unwrap(), 4);
    assert_eq!(rules["decks"].as_u64().unwrap(), 6);

    let hands = json["hands"].as_object().unwrap();
    assert_eq!(hands["total"].as_u64().unwrap(),        1_026_904);
    assert_eq!(hands["won"].as_u64().unwrap(),          427_040);
    assert_eq!(hands["lost"].as_u64().unwrap(),         518_272);
    assert_eq!(hands["push"].as_u64().unwrap(),         81_592);
    assert_eq!(hands["busted"].as_u64().unwrap(),       111_720);
    assert_eq!(hands["blackjack"].as_u64().unwrap(),    47_888);
    assert_eq!(hands["doubled"].as_u64().unwrap(),      103_464);
    assert_eq!(hands["split"].as_u64().unwrap(),        50_672);
    assert_eq!(hands["surrender"].as_u64().unwrap(),    104_880);
    assert_eq!(hands["insured"].as_u64().unwrap(),      0);

    let distrib = json["winning_distrib"].as_object().unwrap();
    assert_eq!(distrib.len(), 16);
    assert_eq!(distrib["-6.0"].as_u64().unwrap(), 48);
    assert_eq!(distrib["-5.0"].as_u64().unwrap(), 120);
    assert_eq!(distrib["-4.0"].as_u64().unwrap(), 424);
    assert_eq!(distrib["-3.0"].as_u64().unwrap(), 2_056);
    assert_eq!(distrib["-2.0"].as_u64().unwrap(), 40_536);
    assert_eq!(distrib["-1.0"].as_u64().unwrap(), 357_464);
    assert_eq!(distrib["-0.5"].as_u64().unwrap(), 104_880);
    assert_eq!(distrib["+0.0"].as_u64().unwrap(), 81_488);
    assert_eq!(distrib["+1.0"].as_u64().unwrap(), 306_048);
    assert_eq!(distrib["+1.5"].as_u64().unwrap(), 45_528);
    assert_eq!(distrib["+2.0"].as_u64().unwrap(), 58_152);
    assert_eq!(distrib["+3.0"].as_u64().unwrap(), 2_008);
    assert_eq!(distrib["+4.0"].as_u64().unwrap(), 960);
    assert_eq!(distrib["+5.0"].as_u64().unwrap(), 240);
    assert_eq!(distrib["+6.0"].as_u64().unwrap(), 40);
    assert_eq!(distrib["+7.0"].as_u64().unwrap(), 8);
}

#[test]
fn ahc_h17_das_d6() {
    let json = run_freebj(&[
        "-n", "1M", "-j8",
        "--ahc", "--h17", "--das", "-d6",
        "--shoe-file", "rc/shoe_1M",
    ]);

    assert_eq!(json["rounds"].as_u64().unwrap(),    1_000_000);
    assert_f64_eq(json["ev"].as_f64().unwrap(),     -0.01069200, 10e-8);
    assert_f64_eq(json["stddev"].as_f64().unwrap(), 1.16202454, 10e-8);

    let rules = json["rules"].as_object().unwrap();
    assert_eq!(rules["game_type"].as_str().unwrap(), "ahc");
    assert_eq!(rules["soft17"].as_str().unwrap(), "h17");
    assert_eq!(rules["das"].as_bool().unwrap(), true);
    assert_eq!(rules["double_down"].as_str().unwrap(), "any_two");
    assert_eq!(rules["surrender"].as_str().unwrap(), "no_surrender");
    assert_eq!(rules["play_ace_pairs"].as_bool().unwrap(), false);
    assert_eq!(rules["max_splits"].as_u64().unwrap(), 4);
    assert_eq!(rules["decks"].as_u64().unwrap(), 6);

    let hands = json["hands"].as_object().unwrap();
    assert_eq!(hands["total"].as_u64().unwrap(),        1_028_664);
    assert_eq!(hands["won"].as_u64().unwrap(),          445_016);
    assert_eq!(hands["lost"].as_u64().unwrap(),         495_504);
    assert_eq!(hands["push"].as_u64().unwrap(),         88_144);
    assert_eq!(hands["busted"].as_u64().unwrap(),       159_976);
    assert_eq!(hands["blackjack"].as_u64().unwrap(),    47_048);
    assert_eq!(hands["doubled"].as_u64().unwrap(),      108_584);
    assert_eq!(hands["split"].as_u64().unwrap(),        54_096);
    assert_eq!(hands["surrender"].as_u64().unwrap(),    0);
    assert_eq!(hands["insured"].as_u64().unwrap(),      0);

    let distrib = json["winning_distrib"].as_object().unwrap();
    assert_eq!(distrib.len(), 15);
    assert_eq!(distrib["-6.0"].as_u64().unwrap(), 32);
    assert_eq!(distrib["-5.0"].as_u64().unwrap(), 48);
    assert_eq!(distrib["-4.0"].as_u64().unwrap(), 512);
    assert_eq!(distrib["-3.0"].as_u64().unwrap(), 2_192);
    assert_eq!(distrib["-2.0"].as_u64().unwrap(), 44_416);
    assert_eq!(distrib["-1.0"].as_u64().unwrap(), 434_608);
    assert_eq!(distrib["+0.0"].as_u64().unwrap(), 87_728);
    assert_eq!(distrib["+1.0"].as_u64().unwrap(), 322_000);
    assert_eq!(distrib["+1.5"].as_u64().unwrap(), 44_824);
    assert_eq!(distrib["+2.0"].as_u64().unwrap(), 59_960);
    assert_eq!(distrib["+3.0"].as_u64().unwrap(), 2_408);
    assert_eq!(distrib["+4.0"].as_u64().unwrap(), 992);
    assert_eq!(distrib["+5.0"].as_u64().unwrap(), 232);
    assert_eq!(distrib["+6.0"].as_u64().unwrap(), 40);
    assert_eq!(distrib["+7.0"].as_u64().unwrap(), 8);
}

#[test]
fn ahc_s17_nodas_d6() {
    let json = run_freebj(&[
        "-n", "1M", "-j8",
        "--ahc", "--s17", "--no-das", "-d6",
        "--shoe-file", "rc/shoe_1M",
    ]);

    assert_eq!(json["rounds"].as_u64().unwrap(),    1_000_000);
    assert_f64_eq(json["ev"].as_f64().unwrap(),     -0.01004000, 10e-8);
    assert_f64_eq(json["stddev"].as_f64().unwrap(), 1.13434055, 10e-8);

    let rules = json["rules"].as_object().unwrap();
    assert_eq!(rules["game_type"].as_str().unwrap(), "ahc");
    assert_eq!(rules["soft17"].as_str().unwrap(), "s17");
    assert_eq!(rules["das"].as_bool().unwrap(), false);
    assert_eq!(rules["double_down"].as_str().unwrap(), "any_two");
    assert_eq!(rules["surrender"].as_str().unwrap(), "no_surrender");
    assert_eq!(rules["play_ace_pairs"].as_bool().unwrap(), false);
    assert_eq!(rules["max_splits"].as_u64().unwrap(), 4);
    assert_eq!(rules["decks"].as_u64().unwrap(), 6);

    let hands = json["hands"].as_object().unwrap();
    assert_eq!(hands["total"].as_u64().unwrap(),        1_025_104);
    assert_eq!(hands["won"].as_u64().unwrap(),          444_416);
    assert_eq!(hands["lost"].as_u64().unwrap(),         491_992);
    assert_eq!(hands["push"].as_u64().unwrap(),         88_696);
    assert_eq!(hands["busted"].as_u64().unwrap(),       160_288);
    assert_eq!(hands["blackjack"].as_u64().unwrap(),    47_304);
    assert_eq!(hands["doubled"].as_u64().unwrap(),      95_192);
    assert_eq!(hands["split"].as_u64().unwrap(),        47_632);
    assert_eq!(hands["surrender"].as_u64().unwrap(),    0);
    assert_eq!(hands["insured"].as_u64().unwrap(),      0);

    let distrib = json["winning_distrib"].as_object().unwrap();
    assert_eq!(distrib.len(), 10);
    assert_eq!(distrib["-4.0"].as_u64().unwrap(), 64);
    assert_eq!(distrib["-3.0"].as_u64().unwrap(), 520);
    assert_eq!(distrib["-2.0"].as_u64().unwrap(), 43_160);
    assert_eq!(distrib["-1.0"].as_u64().unwrap(), 436_272);
    assert_eq!(distrib["+0.0"].as_u64().unwrap(), 89_240);
    assert_eq!(distrib["+1.0"].as_u64().unwrap(), 325_432);
    assert_eq!(distrib["+1.5"].as_u64().unwrap(), 45_040);
    assert_eq!(distrib["+2.0"].as_u64().unwrap(), 59_536);
    assert_eq!(distrib["+3.0"].as_u64().unwrap(), 640);
    assert_eq!(distrib["+4.0"].as_u64().unwrap(), 96);
}

#[test]
fn enhc_s17_das_d6() {
    let json = run_freebj(&[
        "-n", "1M", "-j8",
        "--enhc", "--s17", "--das", "-d6",
        "--shoe-file", "rc/shoe_1M",
    ]);

    assert_eq!(json["rounds"].as_u64().unwrap(),    1_000_000);
    assert_f64_eq(json["ev"].as_f64().unwrap(),     -0.00860400, 10e-8);
    assert_f64_eq(json["stddev"].as_f64().unwrap(), 1.13752418, 10e-8);

    let rules = json["rules"].as_object().unwrap();
    assert_eq!(rules["game_type"].as_str().unwrap(), "enhc");
    assert_eq!(rules["soft17"].as_str().unwrap(), "s17");
    assert_eq!(rules["das"].as_bool().unwrap(), true);
    assert_eq!(rules["double_down"].as_str().unwrap(), "any_two");
    assert_eq!(rules["surrender"].as_str().unwrap(), "no_surrender");
    assert_eq!(rules["play_ace_pairs"].as_bool().unwrap(), false);
    assert_eq!(rules["max_splits"].as_u64().unwrap(), 4);
    assert_eq!(rules["decks"].as_u64().unwrap(), 6);

    let hands = json["hands"].as_object().unwrap();
    assert_eq!(hands["total"].as_u64().unwrap(),        1_026_992);
    assert_eq!(hands["won"].as_u64().unwrap(),          446_408);
    assert_eq!(hands["lost"].as_u64().unwrap(),         493_480);
    assert_eq!(hands["push"].as_u64().unwrap(),         87_104);
    assert_eq!(hands["busted"].as_u64().unwrap(),       179_240);
    assert_eq!(hands["blackjack"].as_u64().unwrap(),    47_000);
    assert_eq!(hands["doubled"].as_u64().unwrap(),      88_960);
    assert_eq!(hands["split"].as_u64().unwrap(),        50_648);
    assert_eq!(hands["surrender"].as_u64().unwrap(),    0);
    assert_eq!(hands["insured"].as_u64().unwrap(),      0);

    let distrib = json["winning_distrib"].as_object().unwrap();
    assert_eq!(distrib.len(), 15);
    assert_eq!(distrib["-7.0"].as_u64().unwrap(), 8);
    assert_eq!(distrib["-6.0"].as_u64().unwrap(), 40);
    assert_eq!(distrib["-5.0"].as_u64().unwrap(), 104);
    assert_eq!(distrib["-4.0"].as_u64().unwrap(), 488);
    assert_eq!(distrib["-3.0"].as_u64().unwrap(), 2_032);
    assert_eq!(distrib["-2.0"].as_u64().unwrap(), 35_264);
    assert_eq!(distrib["-1.0"].as_u64().unwrap(), 442_976);
    assert_eq!(distrib["+0.0"].as_u64().unwrap(), 86_816);
    assert_eq!(distrib["+1.0"].as_u64().unwrap(), 333_576);
    assert_eq!(distrib["+1.5"].as_u64().unwrap(), 44_648);
    assert_eq!(distrib["+2.0"].as_u64().unwrap(), 50_392);
    assert_eq!(distrib["+3.0"].as_u64().unwrap(), 2_496);
    assert_eq!(distrib["+4.0"].as_u64().unwrap(), 912);
    assert_eq!(distrib["+5.0"].as_u64().unwrap(), 192);
    assert_eq!(distrib["+6.0"].as_u64().unwrap(), 56);
}
