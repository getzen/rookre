use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CardKind {
    Suited,
    Joker,
    Bird,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CardSuit {
    /// Cards with no suit do not set the lead suit when played first.
    /// The next card played does.
    None,
    Club,
    Diamond,
    Heart,
    Spade,
    Star,
    /// Cards with unique suit do set the lead suit, but may be followed
    /// by any suit, including another unique.
    Unique,
}

/// The rank showing on the card face.
pub type FaceRank = usize;
/// The rank according to the game rules. Use a maximum of one decimal place, eg 10.5.
pub type GameRank = f32;
pub type CardPoints = isize;
pub type CardId = slotmap::DefaultKey;

#[derive(Clone, Debug)]
pub struct Card {
    pub kind: CardKind,
    pub suit: CardSuit,
    pub face_rank: FaceRank,
    pub game_rank: GameRank,
    pub face_up: bool, // Not just visual. Bot see face-up nest cards.
    pub is_trump: bool,
    pub points: CardPoints,
    pub id: CardId,
}

impl Card {
    pub fn new(kind: CardKind, suit: CardSuit, face_rank: FaceRank) -> Self {
        let id = slotmap::DefaultKey::default();
        Self {
            kind,
            suit,
            face_rank,
            game_rank: face_rank as f32,
            face_up: false,
            is_trump: false,
            points: 0,
            id,
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
        match self.kind {
            CardKind::Suited => {
                let rank = (self.game_rank * 10.0) as usize;
                match self.suit {
                    CardSuit::Club => rank,
                    CardSuit::Diamond => 200 + rank,
                    CardSuit::Heart => 400 + rank,
                    CardSuit::Spade => 600 + rank,
                    CardSuit::Star => 800 + rank,
                    _ => panic!(),
                }
            }
            CardKind::Joker => 1000,
            CardKind::Bird => 1200,
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
    //         CardSuit::Star => format!("star_{}", self.rank),
    //         CardSuit::Green => format!("green_{}", self.rank),
    //         CardSuit::Blue => format!("blue_{}", self.rank),
    //         CardSuit::Red => format!("red_{}", self.rank),
    //         CardSuit::Black => format!("black_{}", self.rank),
    //         CardSuit::Orange => format!("orange_{}", self.rank),
    //         CardSuit::Joker => format!("joker"),
    //         CardSuit::Bird => format!("bird"),
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
        //self.kind == other.kind
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
            CardSuit::Star => write!(f, "{rank}✸"),
            CardSuit::None => todo!(),
            CardSuit::Unique => write!(f, "Jk"),
        }
    }
}
