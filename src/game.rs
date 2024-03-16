use std::collections::VecDeque;

use std::sync::mpsc::Sender;

use slotmap::SlotMap;

use crate::bot::BotKind;
use crate::card::{Card, CardId, CardKind, CardSuit, SelectState};
use crate::game::GameAction::*;
use crate::game_options::{
    BiddingKind, DeckKind, GameOptions, NestPointsOption, PartnerKind, PointsAwarded,
};
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
    DealCards,
    DealCard,
    PresentNest, // one or more cards might be revealed
    PreBid,      // player ui or bot launch
    WaitForBid,
    MoveNestToHand,
    PreDiscard,
    WaitForDiscards,
    PreChooseTrump, // player ui or bot launch
    WaitForChooseTrump,
    PrepareForNewTrick,
    PrePlayCard, // player ui or bot launch
    WaitForPlayCard,
    AwardTrick(PlayerId),
    EndHand,
    EndGame,
}

#[derive(Clone)]
pub enum GameMessage {
    UpdateDeck(Game),
    UpdateNest(Game),
    UpdateHand(Game, PlayerId),
    UpdateActivePlayer(Game),
    UpdateDealer(Game),
    GetBid(Game),
    GetDiscard(Game),
    Delay(f32),
}

#[derive(Clone)]
pub struct Game {
    pub options: GameOptions,
    pub action_queue: VecDeque<GameAction>,

    pub cards: SlotMap<CardId, Card>,
    pub deck: Vec<CardId>,
    pub nest: Vec<CardId>,

    pub player_count: usize,
    pub players: Vec<Player>,
    pub dealer: PlayerId,
    pub active_player: PlayerId,

    pub pass_count: u8,
    pub high_bid: Option<CardSuit>,
    pub bid_winner: Option<PlayerId>,

    pub trick: Trick,
    pub last_trick_winner: PlayerId,
    pub tricks_played: usize,
    pub trump_broken: bool,

    pub game_over: bool,

    pub send_messages: bool,
    message_sender: Sender<GameMessage>,
}

impl Game {
    pub fn new(player_count: usize, action_sender: Sender<GameMessage>) -> Self {
        // Write over the defaults, if needed.
        let options = GameOptions::new();
        options.write_to_yaml("default.txt");

        // Read as normal.
        let options = GameOptions::read_from_yaml("default.txt");

        let mut players = Vec::new();
        for p in 0..options.player_count_default {
            let mut player = Player::new();

            match p {
                0 => player.bot_kind = None,
                _ => player.bot_kind = Some(BotKind::Monte),
            }
            players.push(player);
        }

        let mut action_queue = VecDeque::new();
        action_queue.push_front(GameAction::PrepareForNewHand);

        Self {
            options,
            action_queue,
            cards: SlotMap::new(),
            deck: Vec::new(),
            nest: Vec::new(),
            player_count,
            players,
            dealer: 2,
            active_player: 0,
            pass_count: 0,
            high_bid: None,
            bid_winner: None,
            trick: Trick::new(player_count),
            last_trick_winner: 0,
            tricks_played: 0,
            trump_broken: false,
            game_over: false,

            send_messages: true,
            message_sender: action_sender,
        }
    }

    pub fn active_player(&self) -> &Player {
        &self.players[self.active_player]
    }

    pub fn active_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.active_player]
    }

    pub fn active_hand(&self) -> &Vec<CardId> {
        &self.players[self.active_player].hand
    }

    pub fn active_player_is_bot(&self) -> bool {
        self.active_player().bot_kind.is_some()
    }

    pub fn player_is_bot(&self, player: PlayerId) -> bool {
        self.players[player].bot_kind.is_some()
    }

    pub fn advance_active_player(&mut self) {
        loop {
            self.active_player = (self.active_player + 1) % self.player_count;
            if self.active_player().active {
                break;
            }
        }
        if self.send_messages {
            let msg = GameMessage::UpdateActivePlayer(self.clone());
            self.message_sender.send(msg).unwrap();
        }
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

    fn assign_called_partner(&mut self, caller: PlayerId, card_id: CardId) {
        for p in 0..self.player_count {
            if self.players[p].hand.contains(&card_id) {
                self.players[p].partner = Some(caller);
                self.players[caller].partner = Some(p);
            }
        }
    }

    pub fn create_cards(&mut self) {
        let cards = match self.options.deck_kind {
            DeckKind::Standard52 => self.create_standard_52(),
            DeckKind::Standard53 => self.create_standard_53(),
            DeckKind::Rook56 => self.create_rook_56(),
            DeckKind::Rook57 => self.create_rook_57(),
        };
        self.assign_ids(cards);
    }

    fn assign_ids(&mut self, cards: Vec<Card>) {
        for card in cards {
            let key = self.cards.insert(card);
            //card.id = key;
            if let Some(card) = self.cards.get_mut(key) {
                card.id = key;
            }
        }
    }

    fn create_card_ranks(
        &self,
        from_rank: usize,
        to_rank: usize,
        incl_fifth_suit: bool,
    ) -> Vec<Card> {
        let mut cards = Vec::new();
        for rank in from_rank..=to_rank {
            cards.push(Card::new(CardKind::Suited, CardSuit::Club, rank));
            cards.push(Card::new(CardKind::Suited, CardSuit::Diamond, rank));
            cards.push(Card::new(CardKind::Suited, CardSuit::Heart, rank));
            cards.push(Card::new(CardKind::Suited, CardSuit::Spade, rank));
            if incl_fifth_suit {
                cards.push(Card::new(CardKind::Suited, CardSuit::Star, rank));
            }
        }

        // Remove certain ranks
        for rank in &self.options.remove_ranks {
            cards.retain(|card| card.face_rank != *rank);
        }

        // Remove certain cards
        for (rank, suit_char) in &self.options.remove_cards {
            let remove_suit = Card::suit_for_char(suit_char);
            cards.retain(|card| card.face_rank != *rank || card.suit != remove_suit);
        }

        // Assign card points.
        for (rank, points) in &self.options.face_rank_points {
            for card in &mut cards {
                if card.face_rank == *rank {
                    card.points = *points;
                }
            }
        }

        // Assign any changed game ranks.
        for (face_rank, game_rank) in &self.options.face_rank_to_game_rank_changes {
            for card in &mut cards {
                if card.face_rank == *face_rank {
                    card.game_rank = *game_rank;
                }
            }
        }
        cards
    }

    fn create_standard_52(&self) -> Vec<Card> {
        self.create_card_ranks(2, 14, false)
    }

    fn create_standard_53(&self) -> Vec<Card> {
        let mut cards = self.create_card_ranks(2, 14, false);
        let mut joker = Card::new(CardKind::Joker, CardSuit::Unique, 0);
        joker.game_rank = self.options.bird_joker_rank;
        joker.points = self.options.bird_joker_points;
        cards.push(joker);
        cards
    }

    fn create_rook_56(&self) -> Vec<Card> {
        self.create_card_ranks(1, 14, false)
    }

    fn create_rook_57(&self) -> Vec<Card> {
        let mut cards = self.create_card_ranks(1, 14, false);
        let mut joker = Card::new(CardKind::Bird, CardSuit::Unique, 0);
        joker.game_rank = self.options.bird_joker_rank;
        joker.points = self.options.bird_joker_points;
        cards.push(joker);
        cards
    }

    pub fn prepare_for_new_hand(&mut self) {
        for player in &mut self.players {
            player.reset();
        }

        // Put all the ids in the deck and shuffle.
        self.deck = self.cards.keys().collect();
        fastrand::shuffle(&mut self.deck);
        if self.send_messages {
            let msg = GameMessage::UpdateDeck(self.clone());
            self.message_sender.send(msg).unwrap();
        }

        self.nest.clear();

        self.dealer = (self.dealer + 1) % self.player_count;
        if self.send_messages {
            let msg = GameMessage::UpdateDealer(self.clone());
            self.message_sender.send(msg).unwrap();
        }

        self.active_player = (self.dealer + 1) % self.player_count;
        if self.send_messages {
            let msg = GameMessage::UpdateActivePlayer(self.clone());
            self.message_sender.send(msg).unwrap();
        }

        self.pass_count = 0;

        self.trick = Trick::new(self.player_count);
        self.tricks_played = 0;
        self.trump_broken = !self.options.trump_must_be_broken;

        match self.options.partner_kind {
            PartnerKind::Across => self.assign_across_partners(),
            _ => {}
        }
    }

    fn print_hand(&self, p: PlayerId) {
        let hand = &self.players[p].hand;
        print!("P{p} hand: ");
        for key in hand {
            if let Some(card) = self.cards.get(*key) {
                print!("{card} ");
            }
        }
        println!("");
    }

    fn print_nest(&self) {
        print!("Nest: ");
        for key in &self.nest {
            if let Some(card) = self.cards.get(*key) {
                print!("{card} ");
            }
        }
        println!("");
    }

    /// Deals the given number of cards to each player.
    pub fn deal_cards(&mut self, count: usize) {
        // Start with the player to the dealer's left.
        let mut deal_to = (self.dealer + 1) % self.player_count;

        for _ in 0..(count * self.player_count) {
            if let Some(id) = self.deck.pop() {
                self.players[deal_to].add_to_hand(id);

                if self.send_messages {
                    let msg = GameMessage::UpdateHand(self.clone(), deal_to);
                    self.message_sender.send(msg).unwrap();
                }
            }
            deal_to = (deal_to + 1) % self.player_count;
        }

        // Sort human hands.
        for p in 0..self.player_count {
            if !self.player_is_bot(p) {
                self.sort_hand(p);
                if self.send_messages {
                    let msg = GameMessage::UpdateHand(self.clone(), p);
                    self.message_sender.send(msg).unwrap();
                }
            }
        }

        // Remaining cards to nest
        self.nest.append(&mut self.deck);
        // Flip top card
        if let Some(id) = self.nest.last() {
            if let Some(card) = self.cards.get_mut(*id) {
                card.face_up = true;
            }
        }
        if self.send_messages {
            let msg = GameMessage::UpdateNest(self.clone());
            self.message_sender.send(msg).unwrap();
        }

        for i in 0..self.options.nest_face_up {
            if let Some(card) = self.cards.get_mut(self.nest[i]) {
                card.face_up = true;
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

    pub fn set_select_state(&mut self, state: SelectState, ids: &[CardId]) {
        for id in ids {
            if let Some(card) = self.cards.get_mut(*id) {
                card.select_state = state;
            }
        }
    }

    pub fn available_trump_suits(&self) -> Vec<CardSuit> {
        let top_nest_id = self.nest.last().unwrap();
        let card = self.cards.get(*top_nest_id).unwrap();

        if self.pass_count < self.player_count as u8 {
            // Still round one of bidding.
            return vec![card.suit];
        } else {
            // Second round of bidding.
            let all_suits = vec![
                CardSuit::Club,
                CardSuit::Diamond,
                CardSuit::Heart,
                CardSuit::Spade,
            ];
            let mut suits = Vec::new();
            for suit in all_suits {
                if suit == card.suit {
                    continue;
                } else {
                    suits.push(suit);
                }
            }
            return suits;
        }
    }

    // /// Calculate the minimum and maximum bids based on the card points available.
    // pub fn bid_min_max_increment(&self) -> (usize, usize, usize) {
    //     let mut max = 0;
    //     for card in self.cards.values() {
    //         max += card.points as usize;
    //     }

    //     let min = match &self.high_bid {
    //         Some(high_bid) => match high_bid {
    //             Bid::Pass => max / 2,
    //             Bid::Points(points) => points + self.options.bid_increment,
    //             _ => panic!(),
    //         },
    //         None => max / 2,
    //     };
    //     (min, max, self.options.bid_increment)
    // }

    pub fn make_bid(&mut self, bid: Option<CardSuit>) {
        self.active_player_mut().bid = bid;
        match bid {
            Some(suit) => {
                self.bid_winner = Some(self.active_player);
                self.set_trump(suit);
                self.high_bid = bid;
                if self.pass_count < self.player_count as u8 {
                    // Do card exchange.
                    self.action_queue.push_back(MoveNestToHand);
                    if self.send_messages {
                        let msg = GameMessage::GetDiscard(self.clone());
                        self.message_sender.send(msg).unwrap();
                    }
                } else {
                    // Skip card exchange.
                    self.action_queue.push_back(PrepareForNewTrick)
                }
            }
            None => {
                self.pass_count += 1;
                println!("pass count: {}", self.pass_count);
                self.advance_active_player();
                self.action_queue.push_back(WaitForBid);
            }
        }
    }

    // fn active_bidders_remaining(&self) -> usize {
    //     let mut active_count = 0;
    //     match self.options.bidding_kind {
    //         BiddingKind::Euchre => {
    //             for p in &self.players {
    //                 match p.bid {
    //                     Some(bid) => match bid {
    //                         Bid::Pass => active_count += 1,
    //                         Bid::Suit(_) => {
    //                             return 0;
    //                         }
    //                     },
    //                     None => active_count += 1,
    //                 }
    //             }
    //         }
    //         _ => {
    //             panic!("BiddingKind not implemented.")
    //         }
    //     }
    //     active_count
    // }

    pub fn assign_makers_and_defenders(&mut self) {
        let maker = self.bid_winner.unwrap();
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
        let p = self.bid_winner.unwrap();
        if let Some(id) = self.nest.pop() {
            self.players[p].hand.push(id);
        }
        self.sort_hand(p);
        if self.send_messages {
            let msg = GameMessage::UpdateHand(self.clone(), p);
            self.message_sender.send(msg).unwrap();
        }
    }

    pub fn discard_to_nest(&mut self, ids: Vec<CardId>) {
        let p = self.bid_winner.unwrap();
        let winner = &mut self.players[p];
        for id in ids {
            winner.remove_from_hand(&id);
            self.nest.push(id);
        }
        if self.send_messages {
            let msg = GameMessage::UpdateHand(self.clone(), p);
            self.message_sender.send(msg).unwrap();
            let msg = GameMessage::UpdateNest(self.clone());
            self.message_sender.send(msg).unwrap();
        }
    }

    pub fn undiscard_from_nest(&mut self, id: &CardId) {
        let player_id = self.bid_winner.unwrap();
        self.nest.retain(|i| i != id);
        let winner = &mut self.players[player_id];
        winner.add_to_hand(*id);
        self.sort_hand(player_id);
    }

    pub fn set_trump(&mut self, suit: CardSuit) {
        // Mark the cards matching trump.
        for card in self.cards.values_mut() {
            if card.suit == suit || card.kind == CardKind::Joker || card.kind == CardKind::Bird {
                card.suit = suit;
                card.is_trump = true;
            }
        }
    }

    pub fn get_playable_card_ids(&self) -> Vec<CardId> {
        let mut ids = Vec::new();
        let card_count_matching_lead = self.card_count_matching_lead();
        let has_non_trump_card = self.hand_contains_non_trump_card();

        for id in self.active_hand() {
            let card = self.cards.get(*id).unwrap();
            if self.trick.is_eligible(
                card,
                card_count_matching_lead,
                self.trump_broken,
                has_non_trump_card,
            ) {
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

    fn hand_contains_non_trump_card(&self) -> bool {
        for id in self.active_hand() {
            let card = self.cards.get(*id).unwrap();
            if !card.is_trump {
                return true;
            }
        }
        false
    }

    pub fn play_card_id(&mut self, id: &CardId) {
        self.active_player_mut().remove_from_hand(id);
        let card = self.cards.get(*id).unwrap();
        self.trick.add_card(self.active_player, card);
        if card.is_trump {
            self.trump_broken = true;
        }
        self.advance_active_player();
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

    pub fn nest_points(&self) -> isize {
        let mut points = 0;
        for id in &self.deck {
            let card = self.cards.get(*id).unwrap();
            points += card.points;
        }
        points
    }

    fn award_nest(&mut self) {
        let nest_pts = self.nest_points();
        for (id, player) in self.players.iter_mut().enumerate() {
            if id == self.last_trick_winner {
                let pts = match self.options.nest_points_awarded {
                    NestPointsOption::CardPoints => nest_pts,
                    NestPointsOption::Fixed(p) => p,
                };
                println!("Nest points awarded : {pts}");
                player.points_this_hand += pts;
            }
        }
    }

    fn makers_and_defenders_points(&self) -> (isize, isize) {
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

    pub fn makers_and_defenders_score(&self) -> (isize, isize) {
        let (makers_pts, defenders_pts) = self.makers_and_defenders_points();
        let points_needed = 70; ////////// read from GameOptions instead

        let makers_score;
        let defenders_score;

        if makers_pts >= points_needed as isize {
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

    // fn send_message(&self, message: GameMessage) {
    //     if self.send_messages {
    //         //println!("message: {:?}", message);
    //         self.message_sender.send(message).unwrap();
    //     }
    // }

    pub fn do_next_action(&mut self) {
        if let Some(action) = self.action_queue.pop_front() {
            match action {
                Setup => {
                    self.create_cards();
                    self.action_queue.push_back(PrepareForNewHand);
                    self.do_next_action();
                }
                PrepareForNewHand => {
                    self.prepare_for_new_hand();
                }
                DealCards => {
                    self.deal_cards(self.options.hand_size);
                    self.action_queue.push_back(PresentNest);
                    self.do_next_action();
                }
                PresentNest => {
                    self.action_queue.push_back(WaitForBid);
                    self.do_next_action();
                }
                // PreBid => {
                //     self.action_queue.push_back(WaitForBid);
                // }
                WaitForBid => {
                    if self.send_messages {
                        let msg = GameMessage::GetBid(self.clone());
                        self.message_sender.send(msg).unwrap();
                    }
                    println!("game: WaitForBid");
                }
                MoveNestToHand => {
                    println!("game: MoveNestToHand");
                    self.move_nest_card_to_hand();
                    self.action_queue.push_back(WaitForDiscards);
                    self.do_next_action();
                }
                // PreDiscard => {
                //     self.action_queue.push_back(WaitForDiscards);
                // }
                WaitForDiscards => {
                    println!("game::WaitForDiscards");
                }
                PreChooseTrump => {}
                WaitForChooseTrump => {}
                PrepareForNewTrick => {
                    self.prepare_for_new_trick();
                    self.action_queue.push_back(WaitForPlayCard);
                    self.do_next_action();
                }
                // PrePlayCard => {
                //     self.action_queue.push_back(WaitForPlayCard);
                //     self.do_next_action();
                // }
                WaitForPlayCard => {
                    println!("game: WaitForPlayCard");
                }
                AwardTrick(p_id) => {
                    self.award_trick();
                    self.action_queue.push_back(PrepareForNewTrick);
                    self.do_next_action();
                }
                EndHand => {
                    self.award_nest();
                    println!("========= End of Hand ========")
                }
                EndGame => todo!(),
                _ => {}
            }
        }
    }

    pub fn perform_player_action(&mut self, player_action: &PlayerAction) {
        match player_action {
            PlayerAction::DealCards => {
                self.action_queue.push_back(DealCards);
            }
            PlayerAction::MakeBid(bid) => self.make_bid(*bid),
            PlayerAction::MoveCardToNest(id) => {
                println!("MoveCardToNest");
                self.discard_to_nest(vec![*id]);
            }
            PlayerAction::TakeCardFromNest(id) => {
                println!("TakeCardFroNest");
                self.undiscard_from_nest(id);
            }
            PlayerAction::EndNestExchange => {
                self.action_queue.push_back(PreChooseTrump);
            }

            PlayerAction::PlayCard(_, c_id) => {
                self.play_card_id(c_id);
                if self.trick_completed() {
                    let winner = self.trick.winner.unwrap();
                    self.action_queue.push_back(AwardTrick(winner));
                } else {
                    self.action_queue.push_back(PrePlayCard);
                }
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
