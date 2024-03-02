use crate::bid::Bid;
use crate::bot::Bot;
use crate::bot_random::BotRandom;
use crate::card::{Card, CardId, CardSuit};
use crate::game::Game;
use crate::player::PlayerKind;

/// A MonteCarlo bot. Only card play is MonteCarlo'd at this point.
/// Bidding and discarding are stil rule based.
#[derive(Clone)]
pub struct BotMonte {}

impl BotMonte {
    pub fn new() -> Self {
        Self {}
    }

    #[allow(dead_code)]
    fn suit_strength(&self, hand: &Vec<Card>, suit: &CardSuit) -> f32 {
        let mut points = 0.0;
        for card in hand {
            if card.suit != *suit {
                continue;
            }
            points += card.game_rank;
        }
        points
    }
}

impl Bot for BotMonte {
    // Pass
    fn make_bid(&self, _game: &Game) -> Bid {
        Bid::Pass
    }

    // Choose a random suit.
    fn choose_trump(&self, _game: &Game) -> CardSuit {
        let suits = [
            CardSuit::Club,
            CardSuit::Diamond,
            CardSuit::Heart,
            CardSuit::Spade,
        ];
        let rand_idx = fastrand::usize(0..suits.len());
        suits[rand_idx]
    }

    // Use a MonteCarlo simulation to pick the best card.
    fn play_card(&self, game: &Game) -> CardId {
        let monte_player = game.active_player;

        let playable_ids = game.get_playable_card_ids();
        if playable_ids.len() == 1 {
            return playable_ids[0];
        }

        // Create a vec with all the cards we don't know about.
        let mut hidden_cards = Vec::new();

        for id in &game.deck {
            hidden_cards.push(*id);
        }

        for p in 0..game.players.len() {
            if p == game.active_player {
                continue;
            }
            // Push a copy of player's hand id. We don't want to clear the hand using
            // append() since we need to know the hand len later.
            for id in &game.players[p].hand {
                hidden_cards.push(*id);
            }
        }

        let simulations = 1000;

        let mut id_score = Vec::new();
        for _ in 0..playable_ids.len() {
            id_score.push(0);
        }

        let random_bot = BotRandom::new();

        for _ in 0..simulations {
            let mut sim_game = game.clone();
            let mut cards = hidden_cards.clone();
            fastrand::shuffle(&mut cards);

            // Assign random cards to all players but the active player.
            // TODO: Keep certain suits from being assigned if previous
            // play has shown the player is out of that suit.
            for (p, player) in sim_game.players.iter_mut().enumerate() {
                if p == monte_player {
                    continue;
                }
                let card_count = player.hand.len();
                player.hand.clear();
                for _ in 0..card_count {
                    player.add_to_hand(cards.pop().unwrap());
                }
            }

            // Run the simulation on each playable card id.
            for (i, id) in playable_ids.iter().enumerate() {
                //println!("=== id {} of {}", i, playable_ids.len());
                let mut monte_game = sim_game.clone();

                // for idx in 0..monte_game.players.len() {
                //     println!("{}: {:?}", idx, monte_game.players[idx].partner);
                // }

                monte_game.play_card_id(id);

                while !monte_game.hand_completed() {
                    while !monte_game.trick_completed() {
                        // Randomly play subsequent cards.
                        //println!("active players: {}", monte_game.active_player_count());
                        //println!("rando ids: {}", monte_game.get_playable_card_ids().len());
                        let c_id = random_bot.play_card(&monte_game);
                        monte_game.play_card_id(&c_id);
                    }

                    monte_game.award_trick();
                    monte_game.prepare_for_new_trick();
                }

                let (makers_score, defenders_score) = monte_game.makers_and_defenders_score();
                id_score[i] += match &monte_game.players[monte_player].kind {
                    Some(kind) => match kind {
                        PlayerKind::Maker => makers_score,
                        PlayerKind::Defender => defenders_score,
                    },
                    None => panic!(),
                };
            }
        }

        // Select the card id with the highest score.
        let mut highest_score = 0;
        let mut best_id = slotmap::DefaultKey::default();
        for (idx, score) in id_score.iter().enumerate() {
            println!("p: {monte_player}, idx: {}, score: {}", idx, score);
            if *score >= highest_score {
                highest_score = *score;
                best_id = playable_ids[idx];
            }
        }
        println!("------");
        best_id
    }
}
