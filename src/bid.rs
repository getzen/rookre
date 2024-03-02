use crate::card::CardSuit;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Bid {
    Pass,
    // As in Euchre. Points are the fixed amount needed to make bid, based on points in deck.
    PickItUp(usize),
    Suit(CardSuit, usize),
    Points(usize),
}
