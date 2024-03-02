use std::{collections::VecDeque, sync::mpsc::Sender};

use notan::{
    app::{App, Graphics},
    graphics::Texture,
    math::{vec2, Affine2, Vec2},
    Event,
};
use slotmap::SlotMap;

use crate::{
    animators::{AngleAnimator, TranslationAnimator}, bid::Bid, bid_selector_1::{self, BidSelector1}, card::{Card, CardId}, game::{Game, GameAction, GameMessage, PlayerAction}, image_button::ImageButton, player::PlayerId, sprite::Sprite, text_button::TextButton, view_fn::{ViewFn, CARD_SIZE}, view_trait::ViewTrait
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

    //text_buttons: Vec<TextButton<PlayerAction>>,

    //image_buttons: Vec<ImageButton<PlayerAction>>,
    pub deal_button: ImageButton<PlayerAction>,

    pub bid_selector_1: BidSelector1,
}

impl View {
    pub fn new(
        gfx: &mut Graphics,
        cards: &SlotMap<CardId, Card>,
        sender: Sender<PlayerAction>,
    ) -> Self {
        let sprites = View::create_card_sprites(cards, gfx);
        let deal_button = View::create_deal_button(gfx, sender.clone());
        let bid_selector_1 = View::create_bid_selector_1(gfx, sender.clone());

        Self {
            action_queue: VecDeque::new(),
            last_action_time: 0.0,
            queue_empty: true,
            sprites,
            sprites_z_order_dirty: false,

            deal_button,
            bid_selector_1,
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
            let mut sprite = View::create_card_sprite(gfx, card, &face_down_tex);
            sprite.transform.set_scale_from_size(CARD_SIZE.into());
            sprite.transform.set_translation(vec2(0.0, 0.0));
            sprites.push(sprite);
        }
        sprites
    }

    fn create_card_sprite(gfx: &mut Graphics, card: &Card, face_down_tex: &Texture) -> Sprite {
        let face_up_tex = ViewFn::load_card_texture(gfx, card);
        let sprite = Sprite::new(
            card.id,
            face_up_tex,
            Vec2::ZERO,
            Some(face_down_tex.clone()),
        );
        sprite
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

    fn create_bid_selector_1(gfx: &mut Graphics,
        sender: Sender<PlayerAction>) -> BidSelector1 {
            let tex = gfx
            .create_texture()
            .from_image(include_bytes!("assets/bid_selector_1.png"))
            .build()
            .unwrap();

        let pos = ViewFn::bid_view_position(0, 4);
        //let pos = vec2(0., 0.);
        BidSelector1::new(pos, tex, 1.0, gfx, sender)
    }

    /// Add the action to the queue. It will occur after the last action already
    /// in the queue, if any.
    pub fn queue_message(&mut self, message: GameMessage, game_clone: Option<Game>) {
        // The Delay action is used to push back subsequent actions.
        let delay = match message {
            GameMessage::Delay(d) => d,
            _ => 0.0,
        };
        self.last_action_time += delay;
        let action = Action::new(message, self.last_action_time, game_clone);
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
            let animator = AngleAnimator::new(sprite.transform.angle(), angle, 3.0);
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

    fn get_bid(&mut self, game: &Game) {
        if game.active_player_is_bot() {
            println!("bot bidding: {}", game.active_player);
        } else {
            println!("presenting bid select");
        }
    }
}

impl ViewTrait for View {
    fn mouse_event_handled(
        &mut self,
        event: &Event,
        screen_pt: Vec2,
        parent_affine: Option<&Affine2>,
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
            .bid_selector_1
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
        self.last_action_time -= time_delta;
        self.last_action_time = self.last_action_time.max(0.0);

        for action in &mut self.action_queue {
            action.delay -= time_delta;
        }

        // Move the ready-to-fire actions from the queue and add to a temporary Vec.
        let mut actions_to_fire = Vec::new();
        loop {
            if let Some(action) = self.action_queue.front() {
                if action.delay <= 0.0 {
                    actions_to_fire.push(self.action_queue.pop_front().unwrap());
                    continue;
                }
            }
            break;
        }

        // Fire the actions.
        for action in &actions_to_fire {
            match &action.message {
                GameMessage::UpdateDeck => {
                    self.update_deck(action.game.as_ref().unwrap());
                }
                GameMessage::UpdateNest => {
                    self.update_nest(action.game.as_ref().unwrap(), true);
                }
                GameMessage::UpdateHand(p) => {
                    self.update_hand(action.game.as_ref().unwrap(), *p);
                }
                GameMessage::GetBid(_) => {
                    self.get_bid(action.game.as_ref().unwrap());
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
    }

    fn draw(
        &mut self,
        draw: &mut notan::draw::Draw,
        parent_affine: &notan::math::Affine2,
        gfx: &mut notan::prelude::Graphics,
    ) {
        for sprite in &mut self.sprites {
            sprite.draw(draw, parent_affine, gfx);
        }

        // Buttons
        self.deal_button.draw(draw, parent_affine, gfx);

        self.bid_selector_1.visible = true;
        self.bid_selector_1.draw(draw, parent_affine, gfx);
    }
}
