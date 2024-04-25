use std::collections::VecDeque;

use slotmap::SlotMap;

use crate::bot::BotKind;
use crate::card::{Card, CardId, CardSuit, Points, SelectState};
use crate::game::GameAction::*;
use crate::game_options::{GameOptions, PointsAwarded};
use crate::player::{Player, PlayerId, PlayerKind};
use crate::trick::Trick;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PlayerAction {
    DealCards,
    MakeBid(Option<CardSuit>), // None = pass
    MoveCardToNest(CardId),
    TakeCardFromNest(CardId),
    EndNestExchange,
    PlayCard(PlayerId, CardId),
}

#[derive(Clone, Debug, PartialEq)]
pub enum GameAction {
    Setup,
    PrepareForNewHand,
    DealCard(PlayerId, Vec<CardId>),
    DealToNest,
    WaitForBid, // player ui or bot launch
    MoveNestToHand,
    WaitForDiscards, // player ui or bot launch
    MoveCardToDiscard(CardId),
    PauseAfterDiscard,
    EndNestExchange,
    PrepareForNewTrick,
    PrePlayCard,
    WaitForPlayCard(PlayerId), // player ui or bot launch
    PauseAfterPlayCard,
    AwardTrick(Trick),
    EndHand,
    EndGame,
}

const DEAL_PAUSES: [u8; 5] = [20, 24, 28, 32, 36];

#[derive(Clone)]
pub struct Game {
    pub options: GameOptions,

    pub next_action: Option<GameAction>,
    pub actions_taken: VecDeque<GameAction>,

    pub cards: SlotMap<CardId, Card>,
    pub deck: Vec<CardId>,
    pub nest: Vec<CardId>,

    pub player_count: PlayerId,
    pub players: Vec<Player>,
    pub dealer: PlayerId,
    pub active_player: PlayerId,

    pub deal_pause_idx: usize,
    pub deal_count: u8,
    pub dealing_completed: bool,

    pub pass_count: u8,

    pub maker: Option<PlayerId>,
    pub trump_suit: Option<CardSuit>,

    pub trick: Trick,
    pub last_trick_winner: PlayerId,
    pub tricks_played: u8,

    pub game_over: bool,
}

impl Game {
    pub fn new() -> Self {
        // Write over the defaults, if needed.
        let options = GameOptions::new();
        options.write_to_yaml("default.txt");

        // Read as normal.
        let options = GameOptions::read_from_yaml("default.txt");
        let player_count = 4;

        let mut players = Vec::new();
        for p in 0..player_count {
            let mut player = Player::new();

            match p {
                0 => player.bot_kind = None,
                _ => player.bot_kind = Some(BotKind::Random),
            }
            players.push(player);
        }

        let mut action_queue = VecDeque::new();
        action_queue.push_front(GameAction::PrepareForNewHand);

        Self {
            options,
            next_action: Some(Setup),
            actions_taken: VecDeque::new(),
            cards: SlotMap::new(),
            deck: Vec::new(),
            nest: Vec::new(),
            player_count,
            players,
            dealer: 2,

            deal_pause_idx: 0,
            deal_count: 0,
            dealing_completed: false,

            active_player: 0,
            pass_count: 0,
            trump_suit: None,
            maker: None,
            trick: Trick::new(player_count),
            last_trick_winner: 0,
            tricks_played: 0,
            game_over: false,
        }
    }

    pub fn active_player(&self) -> &Player {
        &self.players[self.active_player]
    }

    pub fn active_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.active_player]
    }

    pub fn active_hand(&self) -> &Vec<CardId> {
        &self.players[self.active_player as usize].hand
    }

    pub fn active_player_is_bot(&self) -> bool {
        self.active_player().bot_kind.is_some()
    }

    pub fn player_is_bot(&self, player: PlayerId) -> bool {
        self.players[player].bot_kind.is_some()
    }

    pub fn advance_active_player(&mut self) {
        self.active_player = (self.active_player + 1) % self.player_count;
        println!("Active P:{}", self.active_player);
    }

    pub fn assign_across_partners(&mut self) {
        if self.player_count != 4 {
            panic!("Can't assign partners for {} players.", self.player_count);
        }
        self.players[0].partner = Some(2);
        self.players[1].partner = Some(3);
        self.players[2].partner = Some(0);
        self.players[3].partner = Some(1);
        println!("partners assigned");
    }

    // fn assign_called_partner(&mut self, caller: PlayerId, card_id: CardId) {
    //     for p in 0..self.player_count {
    //         if self.players[p].hand.contains(&card_id) {
    //             self.players[p].partner = Some(caller);
    //             self.players[caller].partner = Some(p);
    //         }
    //     }
    // }

    pub fn create_cards(&mut self) {
        let mut cards = self.create_card_ranks(5, 14);

        // Remove 6s.
        cards.retain(|card| card.face_rank != 6);

        // Assign card points.
        let rank_points = vec![(5, 5), (10, 10), (14, 15)];
        for (rank, points) in rank_points {
            for card in &mut cards {
                if card.face_rank == rank {
                    card.points = points;
                }
            }
        }

        // Add Jokers. In this game the rank and value of the two Jokers depend on
        // which one is played first, so we'll set the values for the first-played
        // and then change the second Joker after the first is played.
        for _ in 0..2 {
            let mut joker = Card::new(CardSuit::Joker, 15);
            joker.points = 0;
            cards.push(joker);
        }

        // Assign IDs.
        for card in cards {
            let key = self.cards.insert(card);
            if let Some(card) = self.cards.get_mut(key) {
                card.id = key;
            }
        }
    }

    fn create_card_ranks(&self, from_rank: u8, to_rank: u8) -> Vec<Card> {
        let mut cards = Vec::new();
        for rank in from_rank..=to_rank {
            cards.push(Card::new(CardSuit::Club, rank));
            cards.push(Card::new(CardSuit::Diamond, rank));
            cards.push(Card::new(CardSuit::Heart, rank));
            cards.push(Card::new(CardSuit::Spade, rank));
        }
        cards
    }

    pub fn prepare_for_new_hand(&mut self) {
        for player in &mut self.players {
            player.reset();
        }

        // Put all the ids in the deck and shuffle.
        self.deck = self.cards.keys().collect();
        fastrand::shuffle(&mut self.deck);

        self.nest.clear();

        self.dealer = (self.dealer + 1) % self.player_count;

        self.active_player = (self.dealer + 1) % self.player_count;

        self.pass_count = 0;

        self.trick = Trick::new(self.player_count);
        self.tricks_played = 0;

        self.assign_across_partners();
    }

    /// Deals a single card to active_player. Flips it face up and sorts
    /// the hand if the player is human.
    pub fn deal_card(&mut self) {
        let p = self.active_player;
        if let Some(id) = self.deck.pop() {
            self.players[p].add_to_hand(id);
            if !self.active_player_is_bot() {
                if let Some(card) = self.cards.get_mut(id) {
                    card.face_up = true;
                }
                self.sort_hand(p);
            }
        }
    }

    pub fn sort_hand(&mut self, p: PlayerId) {
        // Get the cards for the hand.
        let mut sorted_cards = Vec::new();
        for id in &self.players[p].hand {
            sorted_cards.push(self.cards.get(*id).unwrap());
        }
        sorted_cards.sort();

        // Reassign the hand ids to match the sorted hand_cards ids.
        self.players[p].hand.clear();
        for card in &sorted_cards {
            self.players[p].add_to_hand(card.id);
        }
    }

    // pub fn set_select_state(&mut self, state: SelectState, ids: &[CardId]) {
    //     for id in ids {
    //         if let Some(card) = self.cards.get_mut(*id) {
    //             card.select_state = state;
    //         }
    //     }
    // }

    pub fn make_bid(&mut self, bid: Option<CardSuit>) {
        match bid {
            Some(suit) => {
                self.maker = Some(self.active_player);
                self.set_trump(suit);
            }
            None => {
                self.pass_count += 1;
                println!("pass count: {}", self.pass_count);
                self.advance_active_player();
            }
        }
    }

    pub fn assign_makers_and_defenders(&mut self) {
        let maker = self.maker.unwrap();
        let maker_partner = self.players[maker].partner;

        for (id, player) in self.players.iter_mut().enumerate() {
            if id == maker {
                player.kind = Some(PlayerKind::Maker);
            } else {
                player.kind = Some(PlayerKind::Defender);
            }
            if let Some(maker_partner) = maker_partner {
                if id == maker_partner {
                    player.kind = Some(PlayerKind::Maker);
                } else {
                    player.kind = Some(PlayerKind::Defender);
                }
            }
        }
    }

    pub fn nest_cards(&self) -> Vec<&Card> {
        let mut cards = Vec::new();
        for id in &self.nest {
            cards.push(self.cards.get(*id).unwrap());
        }
        cards
    }

    pub fn move_nest_card_to_hand(&mut self) {
        let p = self.maker.unwrap();
        for _ in 0..self.options.nest_size {
            if let Some(id) = self.nest.pop() {
                self.cards.get_mut(id).unwrap().face_up = true;
                self.players[p].hand.push(id);
            }
        }
        self.sort_hand(p);
    }

    pub fn eligible_discards(&self) -> Vec<CardId> {
        let mut ids = Vec::new();
        for id in &self.active_player().hand {
            if let Some(card) = self.cards.get(*id) {
                if card.suit == CardSuit::Joker {
                    continue;
                }
                ids.push(*id);
            }
        }
        ids
    }

    pub fn discard_to_nest(&mut self, discards: &[CardId]) {
        self.mark_select_state(discards, SelectState::Unselectable);
        for id in discards {
            self.active_player_mut().remove_from_hand(id);
            self.nest.push(*id);
            if let Some(card) = self.cards.get_mut(*id) {
                card.face_up = false;
            }
        }
    }

    pub fn undiscard_from_nest(&mut self, id: &CardId) {
        let player_id = self.maker.unwrap();
        self.nest.retain(|i| i != id);
        let winner = &mut self.players[player_id];
        winner.add_to_hand(*id);
        self.sort_hand(player_id);
    }

    /// Mark the cards matching trump.
    pub fn set_trump(&mut self, suit: CardSuit) {
        self.trump_suit = Some(suit);
        for card in self.cards.values_mut() {
            if card.suit == suit || card.suit == CardSuit::Joker {
                card.is_trump = true;
            }
        }
    }

    pub fn get_playable_card_ids(&self) -> Vec<CardId> {
        let mut ids = Vec::new();
        let card_count_matching_lead = self.card_count_matching_lead();

        for id in self.active_hand() {
            let card = self.cards.get(*id).unwrap();
            //println!("checking id: {:?}", id);
            if self
                .trick
                .is_eligible(card, card_count_matching_lead)
            {
                ids.push(*id);
            }
        }
        if ids.is_empty() {
            println!(
                "No playable ids out of {} cards for p:{}",
                self.active_hand().len(),
                self.active_player
            );
            println!("Lead card: {:?}", self.trick.lead_card);
            println!("# matching lead: {card_count_matching_lead}");
            for id in self.active_hand() {
                let card = self.cards.get(*id).unwrap();
                println!("Hand card: {:?}", card);
            }
        } else {
            println!("P: {}. Playable: {}", self.active_player, ids.len());
        }
        ids
    }

    fn card_count_matching_lead(&self) -> usize {
        let mut count = 0;
        if let Some(lead_card) = &self.trick.lead_card {
            for id in self.active_hand() {
                let card = self.cards.get(*id).unwrap();
                if card.suit == lead_card.suit {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn play_card_id(&mut self, id: &CardId) {
        // Turn off selectability for all cards in hand.
        for id in self.active_hand().clone() {
            if let Some(card) = self.cards.get_mut(id) {
                card.select_state = SelectState::Unselectable;
            }
        }

        self.active_player_mut().remove_from_hand(id);
        let card = self.cards.get_mut(*id).unwrap();
        self.trick.add_card(self.active_player, card);
    }

    pub fn trick_completed(&self) -> bool {
        self.trick.completed()
    }

    pub fn award_trick(&mut self) {
        let winner = self.trick.winner.unwrap();
        self.players[winner].add_to_tricks(self.trick.clone());
        self.last_trick_winner = winner;
        self.tricks_played += 1;
    }

    pub fn prepare_for_new_trick(&mut self) {
        if let Some(trick_winner) = self.trick.winner {
            self.active_player = trick_winner;
        }
        if !self.active_player().active {
            self.advance_active_player();
        }
        self.trick = Trick::new(self.player_count);
    }

    pub fn hand_completed(&self) -> bool {
        self.tricks_played == self.options.hand_size
    }

    pub fn nest_points(&self) -> Points {
        let mut points = 0;
        for id in &self.deck {
            let card = self.cards.get(*id).unwrap();
            points += card.points;
        }
        points
    }

    fn award_nest(&mut self) {
        let pts = self.nest_points() + self.options.nest_points_bonus;
        for (id, player) in self.players.iter_mut().enumerate() {
            if id == self.last_trick_winner {
                player.points_this_hand += pts;
                println!("Nest points awarded : {pts}");
            }
        }
    }

    fn makers_and_defenders_points(&self) -> (Points, Points) {
        let mut makers_pts = 0;
        let mut defenders_pts = 0;
        for p in &self.players {
            match &p.kind {
                Some(kind) => match kind {
                    PlayerKind::Maker => makers_pts += p.points_this_hand,
                    PlayerKind::Defender => defenders_pts += p.points_this_hand,
                },
                None => {}
            }
        }
        (makers_pts, defenders_pts)
    }

    pub fn makers_and_defenders_score(&self) -> (Points, Points) {
        let (makers_pts, defenders_pts) = self.makers_and_defenders_points();
        let points_needed = 70; ////////// read from GameOptions instead

        let makers_score;
        let defenders_score;

        if makers_pts >= points_needed {
            // Bid successful
            makers_score = match self.options.makers_points_awarded_for_win {
                PointsAwarded::Fixed(p) => p,
                PointsAwarded::PointsTakenWithMultiplier(x) => makers_pts * x,
            };
            defenders_score = match self.options.makers_points_awarded_for_loss {
                PointsAwarded::Fixed(p) => p,
                PointsAwarded::PointsTakenWithMultiplier(x) => defenders_pts * x,
            };
        } else {
            // Bid failed
            makers_score = match self.options.makers_points_awarded_for_loss {
                PointsAwarded::Fixed(p) => p,
                PointsAwarded::PointsTakenWithMultiplier(x) => makers_pts * x,
            };
            defenders_score = match self.options.makers_points_awarded_for_win {
                PointsAwarded::Fixed(p) => p,
                PointsAwarded::PointsTakenWithMultiplier(x) => defenders_pts * x,
            };
        }
        (makers_score, defenders_score)
    }

    fn mark_select_state(&mut self, card_ids: &[CardId], state: SelectState) {
        for id in card_ids {
            if let Some(card) = self.cards.get_mut(*id) {
                card.select_state = state;
            }
        }
    }

    /// After dealing or after a bid received, determine the next action.
    fn set_deal_or_bid_action(&mut self, dealing: bool) {
        // Default action
        self.next_action = Some(DealCard(0, Vec::new()));

        if dealing {
            // Have we reached a deal pause?
            if self.deal_count == DEAL_PAUSES[self.deal_pause_idx] {
                self.deal_pause_idx += 1;
                if self.deal_pause_idx == DEAL_PAUSES.len() {
                    self.dealing_completed = true;
                    if self.maker.is_some() {
                        self.next_action = Some(MoveNestToHand);
                    }
                } else if self.maker.is_none() {
                    self.next_action = Some(WaitForBid);
                }
            }
        } else {
            // bidding
            match &self.maker {
                Some(_) => {
                    if self.dealing_completed {
                        self.next_action = Some(MoveNestToHand);
                    }
                }
                None => {
                    if self.pass_count == self.player_count as u8 {
                        self.pass_count = 0;
                    } else {
                        self.next_action = Some(WaitForBid);
                    }
                }
            };
        }
    }

    pub fn do_next_action(&mut self) {
        if let Some(mut action) = self.next_action.take() {
            // self.next_action is now None

            match action {
                Setup => {
                    self.create_cards();
                    self.next_action = Some(PrepareForNewHand);
                }
                PrepareForNewHand => {
                    self.prepare_for_new_hand();
                }
                DealToNest => {
                    println!("game: DealToNest");
                    // Remaining cards to nest
                    for _ in 0..self.options.nest_size {
                        if let Some(id) = self.deck.pop() {
                            self.nest.push(id);
                        }
                    }
                    // Flip nest cards
                    for i in 0..self.options.nest_face_up {
                        let idx = self.nest.len() - 1 - i as usize;
                        if let Some(card) = self.cards.get_mut(self.nest[idx]) {
                            card.face_up = true;
                        }
                    }
                    self.next_action = Some(DealCard(self.active_player, Vec::new()));
                }
                DealCard(_, _) => {
                    self.deal_card();
                    self.deal_count += 1;

                    // Update the action with the new hand (after the card was dealt), since this
                    // will be sent to the Controller.
                    action = DealCard(self.active_player, self.active_hand().clone());
                    self.advance_active_player();
                    self.set_deal_or_bid_action(true);
                }
                WaitForBid => {
                    println!("game: WaitForBid");
                }
                MoveNestToHand => {
                    println!("game: MoveNestToHand");
                    self.move_nest_card_to_hand();
                    self.next_action = Some(WaitForDiscards);
                }
                WaitForDiscards => {
                    println!("game::WaitForDiscards");
                    for id in self.eligible_discards() {
                        if let Some(card) = self.cards.get_mut(id) {
                            card.select_state = SelectState::Selectable
                        }
                    }
                }
                MoveCardToDiscard(id) => {
                    self.discard_to_nest(&vec![id]);
                    if self.nest.len() == self.options.nest_size as usize {
                        self.next_action = Some(PauseAfterDiscard);
                    }
                }
                PauseAfterDiscard => {
                    self.next_action = Some(EndNestExchange);
                }
                EndNestExchange => {
                    let ids = self.active_hand().clone();
                    self.mark_select_state(&ids, SelectState::Unselectable);
                    self.next_action = Some(PrepareForNewTrick)
                }
                PrepareForNewTrick => {
                    self.prepare_for_new_trick();
                    self.next_action = Some(PrePlayCard);
                }
                PrePlayCard => {
                    self.next_action = Some(WaitForPlayCard(self.active_player));
                }
                WaitForPlayCard(..) => {                  
                    for id in self.get_playable_card_ids() {
                        if let Some(card) = self.cards.get_mut(id) {
                            card.select_state = SelectState::Selectable
                        }
                    }
                    println!("game: WaitForPlayCard");
                }
                PauseAfterPlayCard => {
                    if self.trick_completed() {
                        let winner = self.trick.winner.unwrap();
                        self.next_action = Some(AwardTrick(self.trick.clone()));
                    } else {
                        self.advance_active_player();
                        self.next_action = Some(PrePlayCard);
                    }
                }
                AwardTrick(_) => {
                    println!("game: AwardTrick");
                    self.award_trick();
                    self.next_action = Some(PrepareForNewTrick);
                }
                EndHand => {
                    self.award_nest();
                    println!("========= End of Hand ========")
                }
                EndGame => todo!(),
            }
            self.actions_taken.push_back(action);
        }
        if self.next_action.is_some() {
            self.do_next_action();
        }
    }

    pub fn perform_player_action(&mut self, player_action: &PlayerAction) {
        match player_action {
            PlayerAction::DealCards => {
                self.next_action = Some(DealToNest);
            }
            PlayerAction::MakeBid(bid) => {
                self.make_bid(*bid);
                self.set_deal_or_bid_action(false);
            }
            PlayerAction::MoveCardToNest(id) => {
                println!("MoveCardToNest");
                self.next_action = Some(MoveCardToDiscard(*id));
            }
            PlayerAction::TakeCardFromNest(id) => {
                println!("TakeCardFroNest");
                self.undiscard_from_nest(id);
            }
            PlayerAction::EndNestExchange => {
                self.next_action = Some(EndNestExchange);
            }

            PlayerAction::PlayCard(_p, c_id) => {
                println!("game: PlayCard: {:?}", c_id);
                self.play_card_id(c_id);
                self.next_action = Some(PauseAfterPlayCard);
            }
        }
    }
}

// Pop, perform, and return action in queue. Add next action.
// pub fn update(&mut self, time_delta: f32) -> Option<GameAction> {
//     if let Some(action) = self.action_queue.pop_front() {
//         match action {
//             Setup => {
//                 self.create_cards();
//                 self.action_queue.push_back(PrepareForNewHand(Vec::new()));
//             }
//             PrepareForNewHand(_) => {
//                 self.prepare_for_new_hand();
//                 self.action_queue.push_back(DealCards);
//             }
//             DealCards => {
//                 self.deal_cards(self.options.hand_size);
//                 self.action_queue.push_back(PresentNest);
//             }
//             PresentNest => {
//                 self.action_queue.push_back(PreBid);
//             }
//             PreBid => {
//                 self.action_queue.push_back(WaitForBid);
//             }
//             WaitForBid => {}
//             MoveNestToHand => {
//                 self.move_nest_cards_to_hand();
//                 self.action_queue.push_back(PreDiscard);
//             }
//             PreDiscard => {
//                 self.action_queue.push_back(WaitForDiscards);
//             }
//             WaitForDiscards => {}
//             PreChooseTrump => {
//                 self.action_queue.push_back(WaitForChooseTrump);
//             }
//             WaitForChooseTrump => {} // just wait
//             PrepareForNewTrick => {
//                 self.prepare_for_new_trick();
//                 self.action_queue.push_back(PrePlayCard);
//             }
//             PrePlayCard => {
//                 self.action_queue.push_back(WaitForPlayCard);
//             }
//             WaitForPlayCard => {} // just wait
//             AwardTrick(p_id) => {
//                 self.award_trick();
//                 self.action_queue.push_back(PrepareForNewTrick);
//             }
//             EndHand => {
//                 self.award_nest();
//                 println!("========= End of Hand ========")
//             }
//             EndGame => todo!(),
//             Delay(mut time) => {
//                 time -= time_delta;
//                 if time > 0.0 {
//                     self.action_queue.push_front(GameAction::Delay(time));
//                 }
//             }
//             _ => {}
//         }
//         return Some(action);
//     }
//     None
// }
