use std::{collections::VecDeque, sync::mpsc::Sender};

use notan::{
    app::{App, Graphics},
    graphics::Texture,
    math::{vec2, Affine2, Vec2},
    Event,
};
use slotmap::SlotMap;

use crate::{
    animators::{AngleAnimator, TranslationAnimator}, bid::Bid, bid_selector::BidSelector, card::{Card, CardId}, game::{Game, GameAction, GameMessage, PlayerAction}, image::Image, image_button::ImageButton, player::PlayerId, sprite::Sprite, view_fn::ViewFn, view_trait::ViewTrait
};

#[derive(Clone)]
pub struct Action {
    message: GameMessage,
    delay: f32,
    game: Option<Game>,
}

impl Action {
    pub fn new(message: GameMessage, delay: f32, game: Option<Game>) -> Self {
        Self {
            message,
            delay,
            game,
        }
    }
}

pub struct View {
    action_queue: VecDeque<Action>,
    last_action_time: f32,
    queue_empty: bool,
    sprites: Vec<Sprite>,
    sprites_z_order_dirty: bool,

    pub dealer_marker: Image,
    pub deal_button: ImageButton<PlayerAction>,
    pub bid_selector: BidSelector,

    fps_update: f32,
}

impl View {
    pub fn new(
        gfx: &mut Graphics,
        cards: &SlotMap<CardId, Card>,
        sender: Sender<PlayerAction>,
    ) -> Self {
        let sprites = View::create_card_sprites(cards, gfx);
        let dealer_marker = View::create_dealer_marker(gfx);
        let deal_button = View::create_deal_button(gfx, sender.clone());
        let bid_selector = View::create_bid_selector_1(gfx, sender.clone());

        Self {
            action_queue: VecDeque::new(),
            last_action_time: 0.0,
            queue_empty: true,
            sprites,
            sprites_z_order_dirty: false,

            dealer_marker,
            deal_button,
            bid_selector,

            fps_update: 0.0,
        }
    }

    pub fn create_card_sprites(cards: &SlotMap<CardId, Card>, gfx: &mut Graphics) -> Vec<Sprite> {
        let mut sprites = Vec::new();

        let face_down_tex = gfx
            .create_texture()
            .from_image(include_bytes!("assets/cards/back.png"))
            .build()
            .unwrap();

        for (_, card) in cards {
            let sprite = View::create_card_sprite(card, &face_down_tex, gfx);
            sprites.push(sprite);
        }
        sprites
    }

    fn create_card_sprite(card: &Card, face_down_tex: &Texture, gfx: &mut Graphics) -> Sprite {
        let face_up_tex = ViewFn::load_card_texture(gfx, card);
        let sprite = Sprite::new(
            card.id,
            face_up_tex,
            Vec2::ZERO,
            Some(face_down_tex.clone()),
        );
        sprite
    }

    fn create_dealer_marker(gfx: &mut Graphics) -> Image {
        let tex = gfx
            .create_texture()
            .from_image(include_bytes!("assets/dealer_marker.png"))
            .build()
            .unwrap();
        Image::new(tex, Vec2::ZERO)
    }

    fn create_deal_button(
        gfx: &mut Graphics,
        sender: Sender<PlayerAction>,
    ) -> ImageButton<PlayerAction> {
        let enabled = gfx
            .create_texture()
            .from_image(include_bytes!("assets/deal_enabled.png"))
            .build()
            .unwrap();

        let mouse_over = gfx
            .create_texture()
            .from_image(include_bytes!("assets/deal_mouse_over.png"))
            .build()
            .unwrap();

        let mut button = ImageButton::new(
            ViewFn::deck_position(),
            enabled,
            Some(mouse_over),
            None,
            0.5,
            String::new(),
            Some(sender),
        );
        button.mouse_up_message = Some(PlayerAction::DealCards);
        button
    }

    fn create_bid_selector_1(gfx: &mut Graphics, sender: Sender<PlayerAction>) -> BidSelector {
        let tex = gfx
            .create_texture()
            .from_image(include_bytes!("assets/bid_selector_1.png"))
            .build()
            .unwrap();

        let pos = ViewFn::bid_view_position(0, 4);
        BidSelector::new(pos, tex, gfx, sender)
    }

    /// Add the action to the queue.
    pub fn queue_message(&mut self, message: GameMessage) {
        // The Delay action is used to push back subsequent actions.
        let delay = match message {
            GameMessage::Delay(d) => d,
            _ => 0.0,
        };
        //self.last_action_time += delay;
        let action = Action::new(message, delay, game_clone);
        self.action_queue.push_back(action);
        self.queue_empty = false;
    }

    fn update_card(&mut self, id: &CardId, pos: Vec2, angle: f32, z_order: usize, face_up: bool) {
        let sprite = self.sprites.iter_mut().find(|s| s.id == *id).unwrap();

        // Create translation animator if needed.
        if !sprite.transform.translation().abs_diff_eq(pos, 0.1) {
            let animator = TranslationAnimator::new(
                sprite.transform.translation(),
                pos,
                500.0, // velocity
            );
            sprite.translation_animator = Some(animator);
        }

        // Create angle animator if needed.
        if (sprite.transform.angle() - angle).abs() > 0.01 {
            let animator = AngleAnimator::new(sprite.transform.angle(), angle, 6.0);
            sprite.angle_animator = Some(animator);
        }

        sprite.z_order = z_order;
        self.sprites_z_order_dirty = true;

        sprite.use_alt_texture = !face_up;
    }

    fn update_deck(&mut self, game: &Game) {
        for (idx, id) in game.deck.iter().enumerate() {
            let pos = ViewFn::deck_position();
            self.update_card(id, pos, 0.0, idx, false);
        }
    }

    fn update_hand(&mut self, game: &Game, player_id: PlayerId) {
        let p_count = game.players.len();
        let hand = &game.players[player_id].hand;
        let is_bot = game.player_is_bot(player_id);
        for (idx, id) in hand.iter().enumerate() {
            let pos = ViewFn::hand_card_position(player_id, p_count, is_bot, idx, hand.len());
            let angle = ViewFn::player_rotation(player_id, p_count);
            self.update_card(id, pos, angle, idx, !is_bot);
        }
    }

    fn move_card_to_hand(&mut self, card_id: CardId, player_id: PlayerId) {

    }

    fn update_nest(&mut self, game: &Game, display: bool) {
        for (idx, id) in game.nest.iter().enumerate() {
            let card = game.cards.get(*id).expect("No card with id: {id}");
            let pos = if display {
                ViewFn::nest_display_position(idx, game.nest.len())
            } else {
                ViewFn::nest_side_position(idx, game.nest.len())
            };
            self.update_card(&id, pos, 0.0, 100 + idx, card.face_up);
        }
    }

    fn update_dealer(&mut self, game: &Game) {
        let p = game.dealer;
        let count = game.players.len();
        let pos = ViewFn::dealer_marker_position(p, count);
        let angle = ViewFn::player_rotation(p, count);
        self.dealer_marker.transform.set_translation(pos);
        self.dealer_marker.transform.set_angle(angle);
        self.dealer_marker.visible = true;
    }

    fn get_bid(&mut self, game: &Game) {
        if game.active_player_is_bot() {
            println!("bot bidding: {}", game.active_player);
        } else {
            println!("bid_selector visible");
            self.bid_selector.visible = true;
        }
    }
}

impl ViewTrait for View {
    fn mouse_event_handled(
        &mut self,
        event: &Event,
        screen_pt: Vec2,
        parent_affine: &Affine2,
    ) -> bool {
        let screen_pt = vec2(screen_pt.x, screen_pt.y);

        // Check bid chooser first since it is on top (if visible).
        // if self
        //     .trump_chooser_view
        //     .event_handled(&event, &mut self.draw, pt)
        // {
        //     return true;
        // }

        if self
            .deal_button
            .mouse_event_handled(event, screen_pt, parent_affine)
        {
            return true;
        }

        if self
            .bid_selector
            .mouse_event_handled(event, screen_pt, parent_affine)
        {
            return true;
        }

        // Iterate in reverse to check on-top sprites first.
        for sprite in self.sprites.iter_mut().rev() {
            if sprite.mouse_event_handled(event, screen_pt, parent_affine) {
                return true;
            }
        }

        // if self.okay_button.event_handled(&event, &mut self.draw, pt) {
        //     return true;
        // }

        // if let Some(bid_select) = &mut self.bid_select_view {
        //     if bid_select.event_handled(&event, &mut self.draw, pt) {
        //         return true;
        //     }
        // }
        false
    }

    fn update(&mut self, app: &mut App, time_delta: f32) {
        // Update when the last action will occur. Used in add_action() to properly
        // queue up actions.
        //self.last_action_time -= time_delta;
        //self.last_action_time = self.last_action_time.max(0.0);

        // Adjust the delay for the next action in the queue.
        if let Some(action) = self.action_queue.front_mut() {
            action.delay -= time_delta;
        }
        
        // Move the ready-to-fire actions from the queue and add to a temporary Vec.
        let mut actions_to_fire = Vec::new();

        if let Some(action) = self.action_queue.front() {
            if action.delay <= 0.0 {
                actions_to_fire.push(self.action_queue.pop_front().unwrap());
            }
        }
        // loop {
        //     if let Some(action) = self.action_queue.front() {
        //         if action.delay <= 0.0 {
        //             actions_to_fire.push(self.action_queue.pop_front().unwrap());
        //             continue;
        //         }
        //     }
        //     break;
        // }

        // Fire the actions.
        for action in &actions_to_fire {
            match &action.message {
                GameMessage::UpdateDeck(game) => {
                    self.update_deck(game);
                }
                GameMessage::UpdateNest(game) => {
                    self.update_nest(game, true);
                }
                GameMessage::UpdateHand(game, p) => {
                    self.update_hand(game, *p);
                }
                GameMessage::UpdateDealer(game) => {
                    self.update_dealer(game);
                }
                GameMessage::GetBid(game) => {
                    self.get_bid(game);
                }
                GameMessage::Delay(_) => {}
                
            };
        }

        if !self.queue_empty && self.action_queue.is_empty() {
            println!("-- view: queue empty --");
            self.queue_empty = true;
        }

        // Update the sprites
        for sprite in &mut self.sprites {
            sprite.update(app, time_delta);
        }

        // Sort the sprites by z-order if needed.
        if self.sprites_z_order_dirty {
            self.sprites.sort_by(|a, b| a.z_order.cmp(&b.z_order));
            self.sprites_z_order_dirty = false;
        }

        self.fps_update -= time_delta;
    }

    fn draw(&mut self, draw: &mut notan::draw::Draw, parent_affine: &Affine2) {
        let now = std::time::Instant::now();

        for sprite in &mut self.sprites {
            sprite.draw(draw, parent_affine);
        }

        // Images
        self.dealer_marker.draw(draw, parent_affine);

        // Buttons
        self.deal_button.draw(draw, parent_affine);

        self.bid_selector.draw(draw, parent_affine);

        // FPS
        if self.fps_update < 0.0 {
            // let draw_fps = (60.0 / (now.elapsed().as_secs_f32() / 0.0167)) as usize;
            // println!(
            //     "draw millis: {}, fps: {}",
            //     now.elapsed().as_millis(),
            //     draw_fps
            // );
            self.fps_update = 2.0;
        }
    }
}
