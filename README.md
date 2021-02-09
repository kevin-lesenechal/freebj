#  FreeBJ Blackjack simulator #

FreeBJ is a free and open source Blackjack simulator used to make statistical
analysis of game strategy. It supports a variety of game rules including any
valid combination of:

  * European no holecard (ENHC) or American holecard (AHC);
  * Hit soft 17 or stand on soft 17;
  * Any number of card decks;
  * Double after split (DAS) or not;
  * Double on 10-11 only, 9-11 only, or any two cards;
  * Hit split aces or not;
  * Maximum number of split hands;
  * Early and late surrender;

FreeBJ is able to count cards using the hi-lo system and adapt its bet based on
a programmable betting strategy. The simulator also supports playing deviations
and one can dynamically add their own deviations.

FreeBJ is a command-line interface program that prints on its standard output a
JSON of all the statistics gathered; the program can also output on demande a
CSV of bankroll samples to have a look on the evolution of your capital
throughout the playing session.

## Installing ##

The easiest way to install FreeBJ is through Rust's Cargo:

    $ cargo install freebj

Refer to [crates.io](https://crates.io/) for more information on how to install
and use Cargo for your operating system and distribution.

## Typical usage ##

FreeBJ can be invoked without any arguments, it will then run a simulation of
one million rounds with default rules:

  * Americal holecard (AHC);
  * Stand on soft 17 (S17);
  * No DAS;
  * 6 decks of cards;
  * No surrender;

Card counting is disabled by default.

    $ freebj
    {
      "rounds": 1000000,
      "rules": {
        "game_type": "ahc",
        "soft17": "s17",
        "das": false,
        "bj_pays": 1.5,
        "double_down": "any_two",
        "surrender": "no_surrender",
        "play_ace_pairs": false,
        "max_splits": 4,
        "decks": 6,
        "penetration_cards": 250
      },
      "ev": -0.006965999999999973,
      "stddev": 1.134225621872368,
      "winning_distrib": {
        "-4.0": 78,
        "-3.0": 565,
        "-2.0": 42360,
        "-1.0": 436266,
        "+0.0": 88616,
        "+1.0": 326537,
        "+1.5": 44988,
        "+2.0": 59870,
        "+3.0": 612,
        "+4.0": 108
      },
      "hands": {
        "total": 1024402,
        "won": 445247,
        "lost": 491025,
        "push": 88130,
        "busted": 161041,
        "blackjack": 47147,
        "doubled": 95252,
        "split": 46241,
        "insured": 0,
        "surrender": 0
      }
    }

For more information about the JSON output and all the options available, please
refer to the manpage [freebj(1)](doc/freebj.1).

## More usage examples ##

Run a simulation of 10 billions rounds spread across 16 threads:

    $ freebj -n 10G -j 16

Play a european game with early surrender, DAS, hit soft 17, and 4 card decks:

    $ freebj --enhc --esurr --das --h17 -d4

Always stard the rounds with an ace and 5 for the players and an 8 as the dealer
upcard and always double-down (instead of hitting):

    $ freebj -c A,5 --dealer=8 -aD

Enable hi-lo card counting and default playing deviations (first 20 deviations),
use a default betting strategy (bet 1.0 on TC 0, increase bet by 1 for each TC
point, do not play negative TC):

    $ freebj --hilo --deviations

Enable hi-lo, no default deviations, but add a specific user playing deviation
consisting of doubling-down with a TC ≥ 3 with a hard 12 against a dealer 6:

    $ freebj --hilo -D "12vs6:>3D"

Enable hi-lo with no playing devations and set a custom betting strategy where
the player bets 10.00 $ on a TC 0, increases (resp. decreases) their bet by
5.00 $ on each poisitive TC point (resp. negative TC point). A maximum TC of 6
is set to limit risk:

| TC | BET     |
|---:|--------:|
| −3 |  0.00 $ |
| −2 |  0.00 $ |
| −1 |  5.00 $ |
|  0 | 10.00 $ |
| +1 | 15.00 $ |
| +2 | 20.00 $ |
| +3 | 25.00 $ |
| +4 | 30.00 $ |
| +5 | 35.00 $ |
| +6 | 40.00 $ |
| +7 | 40.00 $ |

    $ freebj --hilo -b 10.0 --bet-per-tc=5.0 --bet-max-tc=6

## Blackjack research with FreeBJ ##

FreeBJ can be viewed as a research tool to study Blackjack and its strategies.
Other research tools can be built upon `freebj` (so-called "study" scripts);
none are provided in this source tree, but many are available by the same author
along their output data in the [`freebj_study`] repository.

A website is also available for plushing these results:

<https://freebj.lesenechal.fr>

[`freebj_study`]: https://github.com/kevin-lesenechal/freebj_study
