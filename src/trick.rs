use crate::card::{Card, CardId, Points};
use crate::player::PlayerId;

#[derive(Clone, Debug)]
pub struct Trick {
    pub card_ids: Vec<Option<CardId>>,
    pub is_empty: bool,
    pub lead_card: Option<Card>,
    pub winning_card: Option<Card>,
    pub winner: Option<PlayerId>,
    pub points: Points, // could be negative in a game like Hearts
}

impl Trick {
    pub fn new(player_count: usize) -> Self {
        let mut card_ids = Vec::new();
        for _ in 0..player_count {
            card_ids.push(None);
        }
        Self {
            card_ids,
            is_empty: true,
            lead_card: None,
            winning_card: None,
            winner: None,
            points: 0,
        }
    }

    pub fn completed(&self) -> bool {
        for id in &self.card_ids {
            if id.is_none() {
                return false;
            }
        }
        true
    }

    pub fn is_eligible(
        &self,
        card: &Card,
        cards_matching_lead: usize,
        trump_broken: bool,
        has_none_trump_card: bool,
    ) -> bool {
        // Assume card is eligible.

        if self.is_empty {
            if card.is_trump && !trump_broken && has_none_trump_card {
                return false;
            }
        } else {
            if let Some(lead_card) = &self.lead_card {
                if cards_matching_lead > 0 {
                    // we have matching suit in hand
                    if card.suit != lead_card.suit {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn add_card(&mut self, p_id: PlayerId, card: &Card) {
        if self.is_empty {
            self.lead_card = Some(card.clone());
            // First card played will win the trick unless another card takes the lead.
            // This is even true for cards that don't set the lead suit.
            self.winning_card = Some(card.clone());
            self.winner = Some(p_id);
            self.is_empty = false;
        } else {
            if self.takes_lead(&card) {
                self.winning_card = Some(card.clone());
                self.winner = Some(p_id);
            }
        }

        self.card_ids[p_id] = Some(card.id);
        self.points += card.points;
    }

    pub fn takes_lead(&self, card: &Card) -> bool {
        if let Some(winning_card) = &self.winning_card {
            if card.suit == winning_card.suit {
                return card.game_rank > winning_card.game_rank;
            } else {
                return card.is_trump
            }
        }
        true // first card played
    }
}
