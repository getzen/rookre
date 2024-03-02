use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use crate::bid::Bid;
use crate::bot_monte::BotMonte;
use crate::bot_random::BotRandom;

use crate::card::{Card, CardId, CardSuit, GameRank};
use crate::game::{Game, PlayerAction};

#[allow(unused_variables)]
pub trait Bot {
    fn make_bid(&self, game: &Game) -> Bid;
    fn choose_trump(&self, game: &Game) -> CardSuit;
    fn play_card(&self, game: &Game) -> CardId;
}

pub struct BotMgr {}

impl BotMgr {
    fn get_bot(bot_kind: BotKind) -> Box<dyn Bot> {
        thread::sleep(Duration::from_millis(10));
        match bot_kind {
            BotKind::Random => Box::new(BotRandom::new()),
            //BotKind::Rule => Box::new(BotRule::new()),
            BotKind::Monte => Box::new(BotMonte::new()),
        }
    }

    pub fn make_bid(game: &Game, sender: Sender<PlayerAction>) {
        let player = &game.active_player();
        let bot = BotMgr::get_bot(player.bot_kind.unwrap());
        let bid = bot.make_bid(game);
        sender
            .send(PlayerAction::MakeBid(bid))
            .expect("BotMessage send error.");
    }

    pub fn choose_trump(game: &Game, sender: Sender<PlayerAction>) {
        let player = &game.active_player();
        let bot = BotMgr::get_bot(player.bot_kind.unwrap());
        let suit = bot.choose_trump(game);
        sender
            .send(PlayerAction::ChooseTrump(Some(suit)))
            .expect("BotMessage send error.");
    }

    pub fn play_card(game: &Game, sender: Sender<PlayerAction>) {
        let player = &game.active_player();
        let bot = BotMgr::get_bot(player.bot_kind.unwrap());
        let p_id = game.active_player;
        let c_id = bot.play_card(game);
        sender
            .send(PlayerAction::PlayCard(p_id, c_id))
            .expect("BotMessage send error.");
    }

    // Utility fns

    #[allow(dead_code)]
    pub fn get_cards(game: &Game, ids: &[CardId]) -> Vec<Card> {
        let mut cards = Vec::new();
        for id in ids {
            cards.push(game.cards.get(*id).unwrap().clone());
        }
        cards
    }

    #[allow(dead_code)]
    pub fn ids_with_suit(ids: &[CardId], suit: CardSuit, game: &Game) -> Vec<CardId> {
        let mut suit_ids = Vec::new();
        for id in ids {
            if game.cards.get(*id).unwrap().suit == suit {
                suit_ids.push(*id);
            }
        }
        suit_ids
    }

    #[allow(dead_code)]
    pub fn ids_without_suit(ids: &[CardId], suit: CardSuit, game: &Game) -> Vec<CardId> {
        let mut suit_ids = Vec::new();
        for id in ids {
            if game.cards.get(*id).unwrap().suit != suit {
                suit_ids.push(*id);
            }
        }
        suit_ids
    }

    // #[allow(dead_code)]
    // pub fn ids_that_take_trick_lead(ids: &[CardId], game: &Game) -> Vec<CardId> {
    //     let mut trick_leaders = Vec::new();
    //     for id in ids {
    //         if game.card_id_takes_trick_lead(id) {
    //             trick_leaders.push(*id);
    //         }
    //     }
    //     trick_leaders
    // }

    #[allow(dead_code)]
    pub fn lowest_rank(ids: &[CardId], game: &Game) -> Option<CardId> {
        let mut lowest_rank = GameRank::MAX;
        let mut lowest_id = None;
        for id in ids {
            let card = game.cards.get(*id).unwrap();
            if card.game_rank < lowest_rank {
                lowest_rank = card.game_rank;
                lowest_id = Some(card.id);
            }
        }
        lowest_id
    }

    #[allow(dead_code)]
    pub fn highest_rank(ids: &[CardId], game: &Game) -> Option<CardId> {
        let mut highest_rank = 0.0;
        let mut higest_id = None;
        for id in ids {
            let card = game.cards.get(*id).unwrap();
            if card.game_rank > highest_rank {
                highest_rank = card.game_rank;
                higest_id = Some(card.id);
            }
        }
        higest_id
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq)]
pub enum BotKind {
    Random,
    //Rule,
    Monte,
}
