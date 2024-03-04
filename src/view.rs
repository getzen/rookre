use std::{collections::VecDeque, sync::mpsc::Sender};

use notan::{
    app::{App, Graphics},
    graphics::Texture,
    math::{vec2, Affine2, Vec2},
    Event,
};
use slotmap::SlotMap;

use crate::{
    animators::{AngleAnimator, TranslationAnimator},
    bid::Bid,
    bid_selector::BidSelector,
    card::{Card, CardId, CardSuit},
    game::{Game, GameAction, GameMessage, PlayerAction},
    image::Image,
    image_button::ImageButton,
    player::PlayerId,
    sprite::Sprite,
    view_fn::ViewFn,
    view_trait::ViewTrait,
};

pub struct View {
    game_message_queue: VecDeque<GameMessage>,
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
            game_message_queue: VecDeque::new(),
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
        let mut marker = Image::new(tex, Vec2::ZERO);
        marker.visible = false;
        marker
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
            String::new(),
            Some(sender),
        );
        button.mouse_up_message = Some(PlayerAction::DealCards);
        button
    }

    fn create_bid_selector_1(gfx: &mut Graphics, sender: Sender<PlayerAction>) -> BidSelector {
        let suits = vec![
            CardSuit::Club,
            CardSuit::Diamond,
            CardSuit::Heart,
            CardSuit::Spade,
        ];
        BidSelector::new(suits, gfx, sender)
    }

    /// Add the action to the queue.
    pub fn queue_message(&mut self, message: GameMessage) {
        self.game_message_queue.push_back(message);
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
            self.update_card(id, pos, angle, 100 + idx, !is_bot); // adding 100 so hand cards are higher than deck cards
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
    /// Set send_msg to false once a hit is found to ensure only one object sends a message.
    fn handle_mouse_event(
        &mut self,
        event: &Event,
        screen_pt: Vec2,
        parent_affine: &Affine2,
        mut send_msg: bool,
    ) -> bool {
        let screen_pt = vec2(screen_pt.x, screen_pt.y);

        if self
            .deal_button
            .handle_mouse_event(event, screen_pt, parent_affine, send_msg)
        {
            send_msg = false;
        }

        if self
            .bid_selector
            .handle_mouse_event(event, screen_pt, parent_affine, send_msg)
        {
            send_msg = false;
        }

        // Iterate in reverse to check on-top sprites first.
        for sprite in self.sprites.iter_mut().rev() {
            if sprite.handle_mouse_event(event, screen_pt, parent_affine, send_msg) {
                send_msg = false;
            }
        }
        !send_msg
    }

    fn update(&mut self, time_delta: f32, app: &mut App) {
        // Move the ready-to-go actions from the queue and add to a temporary Vec.
        let mut messages_ready = Vec::new();

        loop {
            if let Some(msg) = self.game_message_queue.pop_front() {
                match msg {
                    GameMessage::Delay(mut time) => {
                        time -= time_delta;
                        if time > 0.0 {
                            self.game_message_queue.push_front(GameMessage::Delay(time));
                            //println!("{time}");
                        }
                        break;
                    }
                    _ => {
                        messages_ready.push(msg);
                    }
                }
            } else {
                break;
            }
        }

        // Fire the messages.
        for msg in &messages_ready {
            match &msg {
                GameMessage::UpdateDeck(game) => {
                    self.update_deck(&game);
                }
                GameMessage::UpdateNest(game) => {
                    self.update_nest(&game, true);
                }
                GameMessage::UpdateHand(game, p) => {
                    self.update_hand(&game, *p);
                }
                GameMessage::UpdateDealer(game) => {
                    self.update_dealer(&game);
                }
                GameMessage::GetBid(game) => {
                    self.get_bid(&game);
                }
                GameMessage::Delay(_) => {}
            };
        }

        if !self.queue_empty && self.game_message_queue.is_empty() {
            println!("-- view: queue empty --");
            self.queue_empty = true;
        }

        // Update the sprites
        for sprite in &mut self.sprites {
            sprite.update(time_delta, app);
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
