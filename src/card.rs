use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CardSuit {
    Club,
    Diamond,
    Heart,
    Spade,
    Joker,
}

#[derive(Clone, Copy, Debug)]
pub enum SelectState {
    Selectable,   // Expands a bit in size when mouse over.
    Unselectable, // Normal size and appearance, just unselectable.
    Dimmed,       // Unselectable and the view should shade in gray to show it.
}

/// The rank showing on the card face.
pub type FaceRank = u8;
/// The rank according to the game rules. Use a maximum of one decimal place, eg 10.5.
pub type GameRank = f32;
pub type CardId = slotmap::DefaultKey;
pub type Points = i16; // allow for negative score in case score system changes

#[derive(Clone, Debug)]
pub struct Card {
    pub id: CardId,
    pub suit: CardSuit,
    pub face_rank: FaceRank,
    pub game_rank: GameRank,
    pub is_trump: bool,
    pub points: Points,
    pub face_up: bool, // face_up here means exposed to all players
    pub select_state: SelectState,
}

impl Card {
    pub fn new(suit: CardSuit, face_rank: FaceRank) -> Self {
        let id = slotmap::DefaultKey::default();
        Self {
            id,
            suit,
            face_rank,
            game_rank: face_rank as f32,
            is_trump: false,
            points: 0,
            face_up: false,
            select_state: SelectState::Unselectable,
        }
    }

    pub fn suit_for_char(c: &char) -> CardSuit {
        match c {
            'c' => CardSuit::Club,
            'd' => CardSuit::Diamond,
            'h' => CardSuit::Heart,
            's' => CardSuit::Spade,
            _ => panic!(),
        }
    }

    /// Used by PartialOrd to determine sort order.
    pub fn sort_order(&self) -> usize {
        let rank = (self.game_rank * 10.0) as usize;
        match self.suit {
            CardSuit::Club => rank,
            CardSuit::Diamond => 200 + rank,
            CardSuit::Heart => 400 + rank,
            CardSuit::Spade => 600 + rank,
            CardSuit::Joker => 800,
        }
    }

    fn rank_string(&self) -> String {
        match self.game_rank as i8 {
            11 => "J".to_string(),
            12 => "Q".to_string(),
            13 => "K".to_string(),
            14 => "A".to_string(),
            _ => {
                let rank = self.game_rank as usize;
                rank.to_string()
            }
        }
    }

    // fn file_string(&self) -> String {
    //     match self.suit {
    //         CardSuit::Club => format!("club_{}", self.rank),
    //         CardSuit::Diamond => format!("diamond_{}", self.rank),
    //         CardSuit::Heart => format!("heart_{}", self.rank),
    //         CardSuit::Spade => format!("spade_{}", self.rank),
    //         CardSuit::Joker => format!("joker"),
    //     }
    // }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_order().cmp(&other.sort_order())
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Card {}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.suit == other.suit && self.face_rank == other.face_rank
    }
}

impl core::fmt::Display for Card {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let rank = self.rank_string();
        match self.suit {
            CardSuit::Spade => write!(f, "{rank}♤"),
            CardSuit::Club => write!(f, "{rank}♧"),
            CardSuit::Diamond => write!(f, "{rank}♦️"),
            CardSuit::Heart => write!(f, "{rank}♥️"),
            CardSuit::Joker => write!(f, "Jk"),
        }
    }
}
