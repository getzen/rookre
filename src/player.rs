use crate::bot::BotKind;
use crate::card::{CardId, Points, CardSuit};
use crate::trick::Trick;

pub type PlayerId = usize;

#[derive(Clone)]
pub enum PlayerKind {
    Maker,
    Defender,
}

#[derive(Clone)]
pub struct Player {
    pub kind: Option<PlayerKind>,
    pub partner: Option<PlayerId>,
    pub active: bool, // alive in hand?
    pub hand: Vec<CardId>,
    pub bid: Option<CardSuit>,
    pub tricks: Vec<Trick>,
    pub points_this_hand: Points,
    pub score: Points,
    pub bot_kind: Option<BotKind>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            kind: None,
            partner: None,
            active: true,
            hand: Vec::new(),
            bid: None,
            tricks: Vec::new(),
            points_this_hand: 0,
            score: 0,
            bot_kind: None,
        }
    }

    pub fn reset(&mut self) {
        self.kind = None;
        self.active = true;
        self.hand.clear();
        self.bid = None;
        self.tricks.clear();
        self.points_this_hand = 0;
    }

    pub fn add_to_hand(&mut self, id: CardId) {
        self.hand.push(id);
    }

    pub fn remove_from_hand(&mut self, id: &CardId) {
        if let Some(index) = self.hand.iter().position(|i| i == id) {
            self.hand.remove(index);
        }
    }

    pub fn add_to_tricks(&mut self, trick: Trick) {
        self.points_this_hand += trick.points;
        self.tricks.push(trick);
    }

    pub fn last_trick_won_ids(&self) -> Vec<CardId> {
        if let Some(last_trick) = &self.tricks.last() {
            let mut ids = Vec::new();
            for id in &last_trick.card_ids {
                if let Some(id) = id {
                    ids.push(*id);
                }
            }
            ids
        } else {
            panic!();
        }
    }

    pub fn finalize_score(&mut self, score_this_hand: Points) -> Points {
        self.score += score_this_hand;
        self.score
    }
}
