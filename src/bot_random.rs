use crate::bid::Bid;
use crate::bot::Bot;
use crate::card::{CardId, CardSuit};
use crate::game::Game;

#[derive(Clone)]
pub struct BotRandom {}

impl BotRandom {
    pub fn new() -> Self {
        Self {}
    }
}

impl Bot for BotRandom {
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

    // Play a random playable card.
    fn play_card(&self, game: &Game) -> CardId {
        let ids = game.get_playable_card_ids();
        let rand_idx = fastrand::usize(0..ids.len());
        ids[rand_idx]
    }
}
