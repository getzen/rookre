use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use notan::draw::*;
use notan::math::Vec2;
use notan::prelude::*;

use crate::bot::BotMgr;
use crate::card::{CardId, SelectState};
use crate::card_update::{CardGroup, CardUpdate};
use crate::game::{Game, GameAction, PlayerAction};
use crate::player::PlayerId;
use crate::trick::Trick;
use crate::view::View;
use crate::view_trait::ViewTrait;

pub enum AudioMessage {
    Play,
}

// Global variable. To access: *AUDIO_SENDER.lock().unwrap()
use std::sync::Mutex;
pub static AUDIO_SENDER: Mutex<Option<Sender<AudioMessage>>> = Mutex::new(None);

#[derive(AppState)]
pub struct Controller {
    game: Game,
    view: View,

    player_action_sender: Sender<PlayerAction>,
    player_action_receiver: Receiver<PlayerAction>,

    audio_message_receiver: Receiver<AudioMessage>,
    card_play: Option<AudioSource>,

    card_updates: VecDeque<CardUpdate>,
    game_action_delay: f32,
}

impl Controller {
    pub fn new(assets: &mut Assets, gfx: &mut Graphics) -> Self {
        let (player_action_sender, player_action_receiver) = mpsc::channel();

        *crate::PIXEL_RATIO.lock().unwrap() = gfx.dpi() as f32;

        let font = gfx
            .create_font(include_bytes!("assets/Futura.ttc"))
            .unwrap();
        *crate::FONT.lock().unwrap() = Some(font as Font);

        let (audio_message_sender, audio_message_receiver) = mpsc::channel();
        *AUDIO_SENDER.lock().unwrap() = Some(audio_message_sender);

        let mut game = Game::new();
        game.create_cards();

        // Game clone speed test
        // let now = std::time::Instant::now();
        // let g = game.clone();
        // let elapsed = now.elapsed().as_micros();
        // println!("game clone: {elapsed} micros");

        let view = View::new(
            assets,
            &game.cards,
            player_action_sender.clone(),
            &game,
        );

       

        Self {
            game,
            player_action_sender,
            player_action_receiver,
            view,
            audio_message_receiver,
            card_play: None,

            card_updates: VecDeque::new(),
            game_action_delay: 0.0,
        }
    }

    pub fn event(&mut self, event: Event) {
        match event {
            Event::MouseMove { x, y }
            | Event::MouseDown { button: _, x, y }
            | Event::MouseUp { button: _, x, y } => {
                let screen_pt = Vec2::new(x as f32, y as f32);
                let affine = &notan::math::Affine2::IDENTITY;
                self.view
                    .handle_mouse_event(&event, screen_pt, &affine, true);
            }
            // Event::KeyDown { key }
            _ => {}
        };
    }

    ///
    pub fn update(&mut self, app: &mut App) {
        let time_delta = app.timer.delta_f32();

        // Skip processing of game.actions if delay is > 0.0.
        self.game_action_delay -= time_delta;
        self.game_action_delay = self.game_action_delay.max(0.0);
        if self.game_action_delay == 0.0 {

            
            let action_results = self.game.do_action_new();
            for result in &action_results {
                self.view.handle_action_result(result);
            }
            

            if let Some(action) = &self.game.actions_taken.pop_front() {
                match action {
                    GameAction::Setup => {
                        self.game_action_delay = 0.5;
                    }
                    GameAction::PrepareForNewHand => {
                        self.update_deck();
                        self.view
                            .update_dealer(self.game.dealer, self.game.player_count);
                    }
                    GameAction::DealToNest => {
                        self.update_nest(&action);
                    }
                    GameAction::DealCard(p, cards) => {
                        self.update_hand(*p, &cards);
                        self.game_action_delay = 0.1;
                    }
                    //GameAction::PreBid => {},
                    GameAction::WaitForBid => {
                        self.view
                            .update_active_player(self.game.active_player, self.game.player_count);
                        if self.game.active_player_is_bot() {
                            self.spawn_make_bid_bot();
                            self.game_action_delay = 0.5;
                        } else {
                            self.view.get_bid(&self.game);
                        }
                    }
                    GameAction::MoveNestToHand => {
                        self.update_hands();
                        self.update_nest(&action);
                    }
                    GameAction::WaitForDiscards => {
                        if self.game.active_player_is_bot() {
                            //self.spawn_make_bid_bot();
                        } else {
                            self.view.get_discard(&self.game);
                        }
                    }
                    GameAction::MoveCardToDiscard(..) => {
                        self.update_hands();
                        self.update_nest(&action);
                    }
                    GameAction::PauseAfterDiscard => {
                        self.update_hands();
                        self.update_nest(&action);
                        self.game_action_delay = 1.5;
                    }
                    GameAction::EndNestExchange => {
                        self.view.end_discard();
                        self.update_nest(&action);
                    }
                    GameAction::PrepareForNewTrick => {
                        self.update_hands();
                    }
                    GameAction::PrePlayCard => {
                        self.update_hands();
                        self.update_active_trick();
                        self.game_action_delay = 1.0;
                    }
                    GameAction::WaitForPlayCard(p) => {
                        if self.game.active_player_is_bot() {
                            self.spawn_play_card_bot();
                        } else {
                            self.view.get_card_play(*p, &self.game);
                        }
                    }
                    GameAction::PauseAfterPlayCard => {
                        self.game_action_delay = 2.0;
                    }
                    GameAction::AwardTrick(trick) => {
                        self.update_won_trick(trick);
                    },
                    GameAction::EndHand => todo!(),
                    GameAction::EndGame => todo!(),
                }
            }
        }

        // if !self.card_updates.is_empty() {
        //     self.view.update_cards(&mut self.card_updates, time_delta);
        // }

        // Check for PlayerAction messages and call related game functions.
        let received = self.player_action_receiver.try_recv();
        if let Ok(action) = received {
            match action {
                PlayerAction::DealCards => {
                    self.view.deal_button.visible = false;
                }
                PlayerAction::MakeBid(opt_suit) => {
                    self.view.set_trump(opt_suit);
                    self.view.bid_selector.visible = false;
                }
                PlayerAction::PlayCard(_, _) => {
                    self.view.end_card_play();
                }
                PlayerAction::MoveCardToNest(_) => {}
                PlayerAction::TakeCardFromNest(_) => todo!(),
                PlayerAction::EndNestExchange => todo!(),
            }
            self.game.perform_player_action(&action);
            self.game.do_next_action();
        }

        self.view.update(time_delta, app);
        self.update_sounds(app);
    }

    fn update_deck(&mut self) {
        let mut update = CardUpdate {
            group: CardGroup::Deck,
            ..Default::default()
        };
        for (idx, id) in self.game.deck.iter().enumerate() {
            update.id = *id;
            update.group_index = idx;
            if let Some(card) = self.game.cards.get(*id) {
                update.face_up = card.face_up;
                update.select_state = card.select_state;
            }
            self.card_updates.push_back(update.clone());
        }
    }

    fn update_hand(&mut self, p: PlayerId, hand: &[CardId]) {
        let mut update = CardUpdate {
            group: CardGroup::Hand,
            group_len: hand.len(),
            player: p,
            player_len: self.game.player_count,
            player_is_bot: self.game.player_is_bot(p),
            ..Default::default()
        };

        for (idx, id) in hand.iter().enumerate() {
            update.id = *id;
            update.group_index = idx;
            if let Some(card) = self.game.cards.get(*id) {
                update.face_up = card.face_up;
                update.select_state = card.select_state;
            }
            self.card_updates.push_back(update.clone());
        }
    }

    fn update_hands(&mut self) {
        for player_id in 0..self.game.player_count {
            let hand = &self.game.players[player_id].hand;

            let mut update = CardUpdate {
                group: CardGroup::Hand,
                group_len: hand.len(),
                player: player_id,
                player_len: self.game.player_count,
                player_is_bot: self.game.player_is_bot(player_id),
                ..Default::default()
            };

            for (idx, id) in hand.iter().enumerate() {
                update.id = *id;
                update.group_index = idx;
                if let Some(card) = self.game.cards.get(*id) {
                    update.face_up = card.face_up;
                    update.select_state = card.select_state;
                }
                self.card_updates.push_back(update.clone());
            }
        }
    }

    fn update_nest(&mut self, game_action: &GameAction) {
        let group = match game_action {
            GameAction::Setup
            | GameAction::PrepareForNewHand
            | GameAction::DealCard(..)
            | GameAction::DealToNest
            | GameAction::WaitForBid
            | GameAction::MoveNestToHand
            | GameAction::WaitForDiscards
            | GameAction::MoveCardToDiscard(..)
            | GameAction::PauseAfterDiscard => CardGroup::NestExchange,
            _ => CardGroup::NestAside,
        };
        let mut update = CardUpdate {
            group: group,
            group_len: self.game.options.nest_size as usize,
            ..Default::default()
        };
        for (idx, id) in self.game.nest.iter().enumerate() {
            update.id = *id;
            update.group_index = idx;
            if let Some(card) = self.game.cards.get(*id) {
                update.face_up = card.face_up;
                update.select_state = card.select_state;
            }
            self.card_updates.push_back(update.clone());
        }
    }

    fn update_active_trick(&mut self) {
        let mut update = CardUpdate {
            group: CardGroup::TrickActive,
            player_len: self.game.player_count,
            ..Default::default()
        };
        for (p, opt_id) in self.game.trick.card_ids.iter().enumerate() {
            if let Some(id) = opt_id {
                update.id = *id;
                update.player = p;
                if let Some(card) = self.game.cards.get(*id) {
                    update.face_up = card.face_up;
                }
                self.card_updates.push_back(update.clone());
            }
        }
    }

    fn update_won_trick(&mut self, trick: &Trick) {
        let mut update = CardUpdate {
            group: CardGroup::TrickAside,
            player_len: self.game.player_count,
            player: trick.winner.unwrap(),
            ..Default::default()
        };

        for opt_id in &trick.card_ids {
            if let Some(id) = opt_id {
                update.id = *id;
                update.face_up = false;
                self.card_updates.push_back(update.clone());
            }
        }
    }

    // Turn the bot loose on the world.
    fn spawn_make_bid_bot(&self) {
        if !self.game.active_player_is_bot() {
            return;
        }
        let game_clone = self.game.clone();
        let sender = self.player_action_sender.clone();
        std::thread::spawn(move || {
            BotMgr::make_bid(&game_clone, sender);
        });
    }

    // Turn the bot loose on the world.
    // fn spawn_choose_trump_bot(&self) {
    //     if !self.game.active_player_is_bot() {
    //         return;
    //     }
    //     let game_clone = self.game.clone();
    //     let sender = self.player_action_sender.clone();
    //     std::thread::spawn(move || {
    //         BotMgr::choose_trump(&game_clone, sender);
    //     });
    // }

    // Turn the bot loose on the world.
    fn spawn_play_card_bot(&self) {
        let game_clone = self.game.clone();
        let sender = self.player_action_sender.clone();
        std::thread::spawn(move || {
            BotMgr::play_card(&game_clone, sender);
        });
    }

    pub fn draw(&mut self, gfx: &mut Graphics) {
        let mut draw = gfx.create_draw();
        draw.clear(crate::view::TABLE_COLOR);

        let affine = notan::math::Affine2::IDENTITY;
        self.view.draw(&mut draw, &affine);

        gfx.render(&draw);
    }

    // To send sound message:
    // if let Some(sender) = &*AUDIO_SENDER.lock().unwrap() {
    // sender
    // .send(AudioMessage::Play)
    // .expect("Message send error.");
    fn update_sounds(&mut self, app: &mut App) {
        let received = self.audio_message_receiver.try_recv();
        if let Ok(message) = received {
            match message {
                AudioMessage::Play => {
                    if self.card_play.is_none() {
                        // self.card_play = Some(
                        // app.audio
                        //     .create_source(include_bytes!("assets/card_play.mp3"))
                        //     .unwrap(),
                        // );
                    }
                    if let Some(source) = &self.card_play {
                        app.audio.play_sound(source, 1.0, false);
                    }
                }
            }
        }
    }
}
