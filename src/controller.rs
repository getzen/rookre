use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use notan::draw::*;
use notan::math::Vec2;
use notan::prelude::*;

use crate::bot::BotMgr;
use crate::card_update::{CardGroup, CardUpdate};
use crate::game::{Game, GameAction, GameMessage, PlayerAction};
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

    game_message_receiver: Receiver<GameMessage>,

    audio_message_receiver: Receiver<AudioMessage>,
    card_play: Option<AudioSource>,

    // NEW
    card_updates: VecDeque<CardUpdate>,
}

impl Controller {
    pub fn new(gfx: &mut Graphics) -> Self {
        let (game_message_sender, game_message_receiver) = mpsc::channel();
        let (player_action_sender, player_action_receiver) = mpsc::channel();

        let mut game = Game::new(game_message_sender);
        game.create_cards();
        game.do_next_action();

        // Game clone speed test
        // let now = std::time::Instant::now();
        // let g = game.clone();
        // let elapsed = now.elapsed().as_micros();
        // println!("game clone: {elapsed} micros");

        let view = View::new(gfx, &game.cards, player_action_sender.clone());

        let (audio_message_sender, audio_message_receiver) = mpsc::channel();
        *AUDIO_SENDER.lock().unwrap() = Some(audio_message_sender);

        Self {
            game,
            game_message_receiver,
            player_action_sender,
            player_action_receiver,
            view,
            audio_message_receiver,
            card_play: None,

            card_updates: VecDeque::new(),
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

        if let Some(action) = &self.game.actions_taken.pop_front() {
            match action {
                GameAction::Setup => {},
                GameAction::PrepareForNewHand => {
                    self.update_deck();
                    self.view.update_dealer(self.game.dealer, self.game.player_count);
                },
                GameAction::DealCards => {
                    self.update_hands();
                    self.update_nest();
                },
                GameAction::PresentNest => {
                    self.update_hands();
                    self.update_nest();
                },
                //GameAction::PreBid => {},
                GameAction::WaitForBid => {
                    self.view.update_active_player(self.game.active_player, self.game.player_count);
                    if self.game.active_player_is_bot() {
                        self.spawn_make_bid_bot();
                    } else {
                        self.view.get_bid(&self.game);
                    }
                },
                GameAction::MoveNestToHand => {
                    self.update_hands();
                    self.update_nest();
                },
                //GameAction::PreDiscard => {},
                GameAction::WaitForDiscards => {
                    if self.game.active_player_is_bot() {
                        //self.spawn_make_bid_bot();
                    } else {
                        self.view.get_discard(&self.game);
                    }
                },
                GameAction::PreChooseTrump => {},
                GameAction::WaitForChooseTrump => {},
                GameAction::PrepareForNewTrick => todo!(),
                GameAction::PrePlayCard => todo!(),
                GameAction::WaitForPlayCard => todo!(),
                GameAction::AwardTrick(_) => todo!(),
                GameAction::EndHand => todo!(),
                GameAction::EndGame => todo!(),
            }
        }

        if !self.card_updates.is_empty() {
            self.view.update_cards(&mut self.card_updates);
        }

        // Check for GameMessages. Pass the messages to the view,
        // possibly with delay added afterward.
        let received = self.game_message_receiver.try_recv();
        if let Ok(message) = received {
            match message {
                //GameMessage::UpdateActivePlayer(..) => self.view.queue_message(message),
                //GameMessage::UpdateDealer(..) => self.view.queue_message(message),
                // GameMessage::GetBid(..) => {
                //     if self.game.active_player_is_bot() {
                //         self.spawn_make_bid_bot();
                //         self.view.queue_message(GameMessage::Delay(1.0));
                //     } else {
                //         self.view.queue_message(message);
                //     }
                // }
                // GameMessage::GetDiscard(..) => {
                    
                // }
                // GameMessage::Delay(_) => todo!(),
                _ => {},
            }
        }

        // Check for PlayerAction messages and call related game functions.
        let received = self.player_action_receiver.try_recv();
        if let Ok(action) = received {
            match action {
                PlayerAction::DealCards => {
                    self.view.deal_button.visible = false;
                }
                PlayerAction::MakeBid(..) => {
                    self.view.bid_selector.visible = false;
                }
                PlayerAction::PlayCard(_, _) => todo!(),
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

    fn update_nest(&mut self) {
        let mut update = CardUpdate {
            group: CardGroup::NestExchange,
            group_len: self.game.nest.len(),
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
