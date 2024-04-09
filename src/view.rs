use std::{collections::VecDeque, sync::mpsc::Sender};

use notan::{
    app::{App, Color, Graphics},
    math::{vec2, Affine2, Vec2},
    Event,
};
use slotmap::SlotMap;

use crate::{
    bid_selector::BidSelector,
    card::{Card, CardId, CardSuit, SelectState},
    card_update::{CardGroup, CardUpdate},
    card_view::CardView,
    discard_panel::DiscardPanel,
    game::{Game, PlayerAction},
    image::Image,
    image_button::ImageButton,
    player::PlayerId,
    texture_loader::CARD_TEX_SCALE,
    view_geom::{ViewGeom, BUTTON_POS, CARD_SIZE, VIEW_CENTER},
    view_trait::ViewTrait,
    TEXTURES,
};

// Colors
pub const TABLE_COLOR: Color = Color::from_rgb(0.3, 0.3, 0.3);
//pub const DEEP_GREEN: Color = Color::new(0. / 255., 175. / 255., 0. / 255., 1.);
pub const LIGHT_GRAY: Color = Color::new(225. / 255., 225. / 255., 225. / 255., 1.);
//pub const MED_GRAY: Color = Color::new(200. / 255., 200. / 255., 200. / 255., 1.);

pub struct View {
    card_views: Vec<CardView<PlayerAction>>,
    card_views_z_order_dirty: bool,

    active_player_marker: Image,
    dealer_marker: Image,
    pub deal_button: ImageButton<PlayerAction>,
    pub bid_selector: BidSelector,
    discard_panel: DiscardPanel,
    card_outlines: Vec<Image>,
    trump_marker: Image,

    fps_update: f32,
}

impl View {
    pub fn new(
        gfx: &mut Graphics,
        cards: &SlotMap<CardId, Card>,
        sender: Sender<PlayerAction>,
        game: &Game,
    ) -> Self {
        let card_views = View::create_card_views(cards, gfx, sender.clone());
        let active_player_marker = View::create_active_player_marker(gfx);
        let dealer_marker = View::create_dealer_marker(gfx);
        let deal_button = View::create_deal_button(gfx, sender.clone());
        let bid_selector = View::create_bid_selector(gfx, sender.clone());
        let discard_panel = View::create_discard_panel(gfx, sender.clone());
        let card_outlines = View::create_card_outlines(gfx, game);
        let trump_marker = View::create_trump_marker(gfx);

        Self {
            card_views,
            card_views_z_order_dirty: false,

            active_player_marker,
            dealer_marker,
            deal_button,
            bid_selector,
            discard_panel,
            card_outlines,
            trump_marker,
            fps_update: 0.0,
        }
    }

    pub fn create_card_views(
        cards: &SlotMap<CardId, Card>,
        gfx: &mut Graphics,
        sender: Sender<PlayerAction>,
    ) -> Vec<CardView<PlayerAction>> {
        let mut card_views = Vec::new();

        for (_, card) in cards {
            let card_view = CardView::new(card, gfx, Some(sender.clone()));
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

    fn create_card_outlines(gfx: &mut Graphics, game: &Game) -> Vec<Image> {
        let mut outlines = Vec::new();
        for idx in 0..2 {
            let tex = gfx
                .create_texture()
                .from_image(include_bytes!("assets/cards/outline.png"))
                .build()
                .unwrap();
            let mut image = Image::new(tex, Vec2::ZERO);
            image.transform.set_size(CARD_SIZE);
            let update = CardUpdate {
                group: CardGroup::NestExchange,
                group_len: game.options.nest_size as usize,
                group_index: idx,
                ..Default::default()
            };
            image.transform.set_translation(update.translation());
            image.visible = false;
            outlines.push(image);
        }
        outlines
    }

    fn create_trump_marker(gfx: &mut Graphics) -> Image {
        let tex = gfx
            .create_texture()
            .from_image(include_bytes!("assets/club.png"))
            .build()
            .unwrap();
        let mut marker = Image::new(tex, VIEW_CENTER);
        marker.transform.set_size(vec2(80.0, 80.0));
        marker.visible = false;
        marker
    }

    pub fn update_cards(&mut self, updates: &mut VecDeque<CardUpdate>, time_delta: f32) {
        // Loop until a card needs updating or there are no updates left then break.
        // This bypasses needless card updates.
        loop {
            if let Some(mut update) = updates.pop_front() {
                // if update.delay > 0.0 {
                //     update.delay -= time_delta;
                //     if update.delay > 0.0 {
                //         updates.push_front(update);
                //     }
                //     break;
                // }
                if self.update_card(update) {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Returns false if the card did not need updating.
    fn update_card(&mut self, update: CardUpdate) -> bool {
        let card_view = self
            .card_views
            .iter_mut()
            .find(|s| s.id == update.id)
            .unwrap();

        if card_view.update == update {
            return false;
        }

        // This include location, angle, and z_order.
        card_view.animate_to(update, 500.0, 6.0);
        self.card_views_z_order_dirty = true;

        card_view.face_up = update.face_up;
        card_view.select_state = update.select_state;
        card_view.update = update;
        true
    }

    pub fn update_active_player(&mut self, p: PlayerId, count: PlayerId) {
        let pos = ViewGeom::active_player_marker_position(p, count);
        let angle = ViewGeom::player_rotation(p, count);
        self.active_player_marker.transform.set_translation(pos);
        self.active_player_marker.transform.set_angle(angle);
        self.active_player_marker.visible = true;
    }

    pub fn update_dealer(&mut self, p: PlayerId, count: PlayerId) {
        let pos = ViewGeom::dealer_marker_position(p, count);
        let angle = ViewGeom::player_rotation(p, count);
        self.dealer_marker.transform.set_translation(pos);
        self.dealer_marker.transform.set_angle(angle);
        self.dealer_marker.visible = true;
    }

    pub fn get_bid(&mut self, game: &Game) {
        if game.active_player_is_bot() {
            println!("bot bidding: {}", game.active_player);
        } else {
            self.bid_selector.visible = true;
            println!("bid_selector visible");
        }
    }

    pub fn set_trump(&mut self, suit: Option<CardSuit>) {
        if let Some(suit) = suit {
            let tex_string = match suit {
                CardSuit::Club => "club".to_string(),
                CardSuit::Diamond => "diamond".to_string(),
                CardSuit::Heart => "heart".to_string(),
                CardSuit::Spade => "spade".to_string(),
                CardSuit::Joker => panic!(),
            };
            if let Some(tex) = TEXTURES.lock().unwrap().get(&tex_string) {
                self.trump_marker.texture = tex.clone();
            }
            self.trump_marker.visible = true;
        }
    }

    pub fn get_discard(&mut self, game: &Game) {
        if game.active_player_is_bot() {
            println!("bot choosing discard: {}", game.active_player);
        } else {
            self.discard_panel.visible = true;

            // Set message for eligible cards.
            for id in game.active_hand() {
                if let Some(card) = game.cards.get(*id) {
                    let card_view = self.card_views.iter_mut().find(|s| s.id == *id).unwrap();
                    card_view.mouse_up_message = match card.select_state {
                        SelectState::Selectable => Some(PlayerAction::MoveCardToNest(*id)),
                        _ => None,
                    }
                }
            }
        }

        for outline in &mut self.card_outlines {
            outline.visible = true;
        }
    }

    pub fn end_discard(&mut self) {
        self.discard_panel.visible = false;
        for outline in &mut self.card_outlines {
            outline.visible = false;
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
            }
        }

        !send_msg
    }

    fn update(&mut self, time_delta: f32, app: &mut App) {
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

        // Images
        self.active_player_marker.draw(draw, parent_affine);
        self.dealer_marker.draw(draw, parent_affine);
        self.trump_marker.draw(draw, parent_affine);
        for outline in &mut self.card_outlines {
            outline.draw(draw, parent_affine);
        }

        for card_view in &mut self.card_views {
            card_view.draw(draw, parent_affine);
        }

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
