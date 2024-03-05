use crate::card::CardSuit;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Bid {
    Pass,
    Suit(CardSuit),
}
