use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use notan::draw::*;
use notan::math::Vec2;
use notan::prelude::*;

use crate::bot::BotMgr;
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
}

impl Controller {
    pub fn new(gfx: &mut Graphics) -> Self {
        let player_count = 4;

        let (game_message_sender, game_message_receiver) = mpsc::channel();
        let (player_action_sender, player_action_receiver) = mpsc::channel();

        let mut game = Game::new(player_count, game_message_sender);
        game.create_cards();
        game.assign_across_partners();
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
        }
    }

    pub fn event(&mut self, event: Event) {
        match event {
            Event::MouseMove { x, y }
            | Event::MouseDown { button: _, x, y }
            | Event::MouseUp { button: _, x, y } => {
                let screen_pt = Vec2::new(x as f32, y as f32);
                let affine = &notan::math::Affine2::IDENTITY;
                self.view.handle_mouse_event(&event, screen_pt, &affine, true);
            }
            // Event::KeyDown { key }
            _ => {}
        };
    }

    ///
    pub fn update(&mut self, app: &mut App) {
        let time_delta = app.timer.delta_f32();

        // Check for GameMessages. Pass the messages to the view,
        // possibly with delay added afterward.
        let received = self.game_message_receiver.try_recv();
        if let Ok(message) = received {
            match message {
                GameMessage::UpdateDeck(_) => {
                    self.view.queue_message(message);
                    self.view.queue_message(GameMessage::Delay(2.0));
                }
                GameMessage::UpdateNest(_) => {
                    self.view.queue_message(message);
                    self.view.queue_message(GameMessage::Delay(0.1));
                }
                GameMessage::UpdateHand(..) => {
                    self.view.queue_message(message);
                    self.view.queue_message(GameMessage::Delay(0.1));
                }
                GameMessage::UpdateDealer(..) => self.view.queue_message(message),
                GameMessage::GetBid(..) => {
                    self.view.queue_message(message);
                    if self.game.active_player_is_bot() {
                        self.spawn_make_bid_bot();
                        self.view.queue_message(GameMessage::Delay(1.0));
                    }
                }
                GameMessage::Delay(_) => todo!(),
            }
        }

        // Check for PlayerAction messages and call related game functions.
        let received = self.player_action_receiver.try_recv();
        if let Ok(action) = received {
            match action {
                PlayerAction::DealCards => {
                    self.view.deal_button.visible = false;
                }
                PlayerAction::MakeBid(_) => todo!(),
                PlayerAction::ChooseTrump(_) => todo!(),
                PlayerAction::PlayCard(_, _) => todo!(),
                PlayerAction::MoveCardToNest(_) => todo!(),
                PlayerAction::TakeCardFromNest(_) => todo!(),
                PlayerAction::EndNestExchange => todo!(),
            }
            self.game.perform_player_action(&action);
            self.game.do_next_action();
        }

        self.view.update(time_delta, app);
        self.update_sounds(app);
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
    fn spawn_choose_trump_bot(&self) {
        if !self.game.active_player_is_bot() {
            return;
        }
        let game_clone = self.game.clone();
        let sender = self.player_action_sender.clone();
        std::thread::spawn(move || {
            BotMgr::choose_trump(&game_clone, sender);
        });
    }

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
        draw.clear(crate::view_fn::TABLE_COLOR);

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
