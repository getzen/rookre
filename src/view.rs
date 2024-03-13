use std::{collections::VecDeque, sync::mpsc::Sender};

use notan::{
    app::{App, Graphics},
    math::{vec2, Affine2, Vec2},
    Event,
};
use slotmap::SlotMap;

use crate::{
    animators::{AngleAnimator, TranslationAnimator},
    bid_selector::BidSelector,
    card::{Card, CardId, CardSuit},
    card_location::{CardGroup, CardLocation},
    card_view::CardView,
    discard_panel::{self, DiscardPanel},
    game::{Game, GameAction, GameMessage, PlayerAction},
    image::Image,
    image_button::ImageButton,
    player::PlayerId,
    view_geom::{ViewGeom, BUTTON_POS},
    view_trait::ViewTrait,
};

pub struct View {
    game_message_queue: VecDeque<GameMessage>,
    queue_empty: bool,
    card_views: Vec<CardView>,
    card_views_z_order_dirty: bool,

    pub active_player_marker: Image,
    pub dealer_marker: Image,
    pub deal_button: ImageButton<PlayerAction>,
    pub bid_selector: BidSelector,
    pub discard_panel: DiscardPanel,

    fps_update: f32,
}

impl View {
    pub fn new(
        gfx: &mut Graphics,
        cards: &SlotMap<CardId, Card>,
        sender: Sender<PlayerAction>,
    ) -> Self {
        let card_views = View::create_card_views(cards, gfx);
        let active_player_marker = View::create_active_player_marker(gfx);
        let dealer_marker = View::create_dealer_marker(gfx);
        let deal_button = View::create_deal_button(gfx, sender.clone());
        let bid_selector = View::create_bid_selector(gfx, sender.clone());
        let discard_panel = View::create_discard_panel(gfx, sender.clone());

        Self {
            game_message_queue: VecDeque::new(),
            queue_empty: true,
            card_views,
            card_views_z_order_dirty: false,

            active_player_marker,
            dealer_marker,
            deal_button,
            bid_selector,
            discard_panel,

            fps_update: 0.0,
        }
    }

    pub fn create_card_views(cards: &SlotMap<CardId, Card>, gfx: &mut Graphics) -> Vec<CardView> {
        let mut card_views = Vec::new();

        for (_, card) in cards {
            let card_view = CardView::new(card, gfx);
            card_views.push(card_view);
        }
        card_views
    }

    fn create_active_player_marker(gfx: &mut Graphics) -> Image {
        let tex = gfx
            .create_texture()
            .from_image(include_bytes!("assets/active_player.png"))
            .build()
            .unwrap();
        let mut marker = Image::new(tex, Vec2::ZERO);
        marker.visible = false;
        marker
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
            BUTTON_POS,
            enabled,
            Some(mouse_over),
            None,
            String::new(),
            Some(sender),
        );
        button.mouse_up_message = Some(PlayerAction::DealCards);
        button
    }

    fn create_bid_selector(gfx: &mut Graphics, sender: Sender<PlayerAction>) -> BidSelector {
        let suits = vec![
            CardSuit::Club,
            CardSuit::Diamond,
            CardSuit::Heart,
            CardSuit::Spade,
        ];
        BidSelector::new(suits, gfx, sender)
    }

    fn create_discard_panel(gfx: &mut Graphics, sender: Sender<PlayerAction>) -> DiscardPanel {
        DiscardPanel::new(gfx, sender)
    }

    /// Add the action to the queue.
    pub fn queue_message(&mut self, message: GameMessage) {
        self.game_message_queue.push_back(message);
        self.queue_empty = false;
    }

    fn update_card(&mut self, id: CardId, location: &CardLocation, game: &Game) {
        let card_view = self.card_views.iter_mut().find(|s| s.id == id).unwrap();
        card_view.location = location.clone();

        // Create translation animator if needed.
        let new_trans = location.translation();
        if !card_view
            .transform
            .translation()
            .abs_diff_eq(new_trans, 0.1)
        {
            let animator = TranslationAnimator::new(
                card_view.transform.translation(),
                new_trans,
                500.0, // velocity
            );
            card_view.translation_animator = Some(animator);
        }

        // Create angle animator if needed.
        let new_angle = location.angle();
        if (card_view.transform.angle() - new_angle).abs() > 0.01 {
            let animator = AngleAnimator::new(card_view.transform.angle(), new_angle, 6.0);
            card_view.angle_animator = Some(animator);
        }

        card_view.z_order = location.z_order();
        self.card_views_z_order_dirty = true;

        if let Some(card) = game.cards.get(id) {
            card_view.face_down = !card.face_up;
        }
    }

    fn update_deck(&mut self, game: &Game) {
        let mut location = CardLocation {
            group: CardGroup::Deck,
            ..Default::default()
        };
        for (idx, id) in game.deck.iter().enumerate() {
            location.group_index = idx;
            self.update_card(*id, &location, game);
        }
    }

    fn update_hand(&mut self, game: &Game, player_id: PlayerId) {
        let hand = &game.players[player_id].hand;

        let mut location = CardLocation {
            group: CardGroup::Hand,
            group_len: hand.len(),
            player: player_id,
            player_len: game.player_count,
            player_is_bot: game.player_is_bot(player_id),
            ..Default::default()
        };

        for (idx, id) in hand.iter().enumerate() {
            let card_view = self.card_views.iter_mut().find(|s| s.id == *id).unwrap();
            location.group_index = idx;
            location.mouse_over = card_view.mouse_over;
            self.update_card(*id, &location, game);
        }
    }

    fn update_nest(&mut self, game: &Game) {
        let mut location = CardLocation {
            group: CardGroup::NestExchange,
            group_len: game.nest.len(),
            ..Default::default()
        };
        for (idx, id) in game.nest.iter().enumerate() {
            location.group_index = idx;
            self.update_card(*id, &location, game);
        }
    }

    fn update_active_player(&mut self, game: &Game) {
        let p = game.active_player;
        let count = game.player_count;
        let pos = ViewGeom::active_player_marker_position(p, count);
        let angle = ViewGeom::player_rotation(p, count);
        self.active_player_marker.transform.set_translation(pos);
        self.active_player_marker.transform.set_angle(angle);
        self.active_player_marker.visible = true;
    }

    fn update_dealer(&mut self, game: &Game) {
        let p = game.dealer;
        let count = game.player_count;
        let pos = ViewGeom::dealer_marker_position(p, count);
        let angle = ViewGeom::player_rotation(p, count);
        self.dealer_marker.transform.set_translation(pos);
        self.dealer_marker.transform.set_angle(angle);
        self.dealer_marker.visible = true;
    }

    fn get_bid(&mut self, game: &Game) {
        if game.active_player_is_bot() {
            println!("bot bidding: {}", game.active_player);
        } else {
            let suits = game.available_trump_suits();
            self.bid_selector.set_enabled_suits(suits);
            self.bid_selector.visible = true;
            println!("bid_selector visible");
        }
    }

    fn get_discard(&mut self, game: &Game) {
        if game.active_player_is_bot() {
            println!("bot choosing discard: {}", game.active_player);
        } else {
            self.discard_panel.visible = true;

            // Set eligible cards.
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

        if self
            .discard_panel
            .handle_mouse_event(event, screen_pt, parent_affine, send_msg)
        {
            send_msg = false;
        }

        // Iterate in reverse to check on-top sprites first.
        for card_view in self.card_views.iter_mut().rev() {
            if card_view.handle_mouse_event(event, screen_pt, parent_affine, send_msg) {
                send_msg = false;
                card_view.mouse_over = true;
            } else {
                card_view.mouse_over = false;
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
                    self.update_nest(&game);
                }
                GameMessage::UpdateHand(game, p) => {
                    self.update_hand(&game, *p);
                }
                GameMessage::UpdateActivePlayer(game) => {
                    self.update_active_player(game);
                }
                GameMessage::UpdateDealer(game) => {
                    self.update_dealer(&game);
                }
                GameMessage::GetBid(game) => {
                    self.get_bid(&game);
                }
                GameMessage::GetDiscard(game) => {
                    self.get_discard(&game);
                }
                GameMessage::Delay(_) => {}
            };
        }

        if !self.queue_empty && self.game_message_queue.is_empty() {
            println!("-- view: queue empty --");
            self.queue_empty = true;
        }

        // Update the cards.
        for card_view in &mut self.card_views {
            card_view.update(time_delta, app);
        }

        // Sort the cards by z-order if needed.
        if self.card_views_z_order_dirty {
            self.card_views.sort_by(|a, b| a.z_order.cmp(&b.z_order));
            self.card_views_z_order_dirty = false;
        }

        self.fps_update -= time_delta;
    }

    fn draw(&mut self, draw: &mut notan::draw::Draw, parent_affine: &Affine2) {
        //let now = std::time::Instant::now();

        for card_view in &mut self.card_views {
            card_view.draw(draw, parent_affine);
        }

        // Images
        self.active_player_marker.draw(draw, parent_affine);
        self.dealer_marker.draw(draw, parent_affine);

        // Buttons and panels
        self.deal_button.draw(draw, parent_affine);

        self.bid_selector.draw(draw, parent_affine);

        self.discard_panel.draw(draw, parent_affine);

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
