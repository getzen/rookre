use std::{collections::VecDeque, sync::mpsc::Sender};

use notan::{
    app::{assets::Assets, App, Color, Graphics},
    draw::{CreateFont, Font},
    math::{vec2, Affine2, Vec2},
    Event,
};
use slotmap::SlotMap;

use crate::{
    bid_selector::BidSelector,
    card::{Card, CardId, CardSuit, SelectState},
    card_update::{CardGroup, CardUpdate},
    card_view::CardView,
    game::{ActionResult, Game, PlayerAction},
    image::Image,
    image_button::ImageButton,
    player::PlayerId,
    view_geom::{ViewGeom, BUTTON_POS, VIEW_CENTER},
    view_trait::ViewTrait,
};

// Colors
pub const TABLE_COLOR: Color = Color::from_rgb(0.3, 0.3, 0.3);
//pub const DEEP_GREEN: Color = Color::new(0. / 255., 175. / 255., 0. / 255., 1.);
pub const LIGHT_GRAY: Color = Color::new(225. / 255., 225. / 255., 225. / 255., 1.);
//pub const MED_GRAY: Color = Color::new(200. / 255., 200. / 255., 200. / 255., 1.);

pub struct View {
    tex_loader_completed: bool,

    deck: Vec<CardView>,
    nest: Vec<CardView>,
    hands: Vec<Vec<CardView>>,
    active_trick: Vec<Option<CardView>>,
    tricks: Vec<Vec<CardView>>,

    //card_views: Vec<CardView>,
    //card_views_z_order_dirty: bool,

    player_count: PlayerId,
    active_player_marker: Image,
    dealer_marker: Image,
    pub deal_button: ImageButton<PlayerAction>,
    pub bid_selector: BidSelector,
    discard_panel: Image,
    discard_outlines: Vec<Image>,
    trump_marker: Image,
    play_outline: Image,

    fps_update: f32,
}

impl View {
    pub fn new(
        assets: &mut Assets,
        cards: &SlotMap<CardId, Card>,
        sender: Sender<PlayerAction>,
        game: &Game,
    ) -> Self {
        View::load_texture_assets(assets);

        let deck = View::create_card_views(cards, assets, sender.clone());
        let active_player_marker = View::create_active_player_marker();
        let dealer_marker = View::create_dealer_marker();
        let deal_button = View::create_deal_button(sender.clone());
        let bid_selector = View::create_bid_selector(sender.clone());
        let discard_panel = View::create_discard_panel();
        let discard_outlines = View::create_discard_outlines(game);
        let trump_marker = View::create_trump_marker();
        let play_outline = View::create_play_outline();

        Self {
            tex_loader_completed: false,

            deck,
            nest: Vec::new(),
            hands: vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            active_trick: vec![None, None, None, None],
            tricks: vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            //card_views_z_order_dirty: false,

            player_count: game.player_count,
            active_player_marker,
            dealer_marker,
            deal_button,
            bid_selector,
            discard_panel,
            discard_outlines,
            trump_marker,
            play_outline,
            fps_update: 0.0,
        }
    }

    fn load_texture_assets(assets: &mut Assets) {
        let names = [
            "active_player",
            "bid_selector",
            "cards/back",
            "cards/outline",
            "club",
            "club_mouse_over",
            "deal_enabled",
            "deal_mouse_over",
            "dealer_marker",
            "diamond",
            "diamond_mouse_over",
            "discard",
            "done_enabled",
            "heart",
            "heart_mouse_over",
            "pass_enabled",
            "pass_mouse_over",
            "spade",
            "spade_mouse_over",
        ];
        let tex_names: Vec<String> = names.iter().map(|&n| n.to_string()).collect();
        crate::TEX_LOADER
            .lock()
            .unwrap()
            .load_assets(assets, &tex_names);
    }

    pub fn create_card_views(
        cards: &SlotMap<CardId, Card>,
        assets: &mut Assets,
        sender: Sender<PlayerAction>,
    ) -> Vec<CardView> {
        let mut card_views = Vec::new();
        let mut tex_names = Vec::new();

        for (_, card) in cards {
            let card_view = CardView::new(
                card.id,
                card.points,
                &card.file_string(),
                "cards/back",
                0.5,
                Some(sender.clone()),
            );
            card_views.push(card_view);
            tex_names.push(card.file_string());
        }

        crate::TEX_LOADER
            .lock()
            .unwrap()
            .load_assets(assets, &tex_names);

        card_views
    }

    fn create_active_player_marker() -> Image {
        let mut image = Image::new("active_player", Vec2::ZERO, 0.5);
        image.visible = false;
        image
    }

    fn create_dealer_marker() -> Image {
        let mut image = Image::new("dealer_marker", Vec2::ZERO, 0.5);
        image.visible = false;
        image
    }

    fn create_deal_button(sender: Sender<PlayerAction>) -> ImageButton<PlayerAction> {
        let mut button = ImageButton::new(
            BUTTON_POS,
            "deal_enabled",
            "deal_mouse_over",
            "",
            0.5,
            "",
            Some(sender),
        );
        button.mouse_up_message = Some(PlayerAction::DealCards);
        button
    }

    fn create_bid_selector(sender: Sender<PlayerAction>) -> BidSelector {
        let suits = vec![
            CardSuit::Club,
            CardSuit::Diamond,
            CardSuit::Heart,
            CardSuit::Spade,
        ];
        let trans = ViewGeom::bid_view_position(0, 4);
        BidSelector::new(suits, "bid_selector", 0.50, trans, sender)
    }

    fn create_discard_panel() -> Image {
        let mut trans = VIEW_CENTER;
        trans.y += 150.0;
        let mut image = Image::new("discard", trans, 0.50);
        image.visible = false;
        image
    }

    fn create_discard_outlines(game: &Game) -> Vec<Image> {
        let mut outlines = Vec::new();
        for idx in 0..2 {
            let mut image = Image::new("cards/outline", Vec2::ZERO, 0.35);
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

    fn create_play_outline() -> Image {
        let mut image = Image::new("cards/outline", Vec2::ZERO, 0.35);
        image.visible = false;
        image
    }

    fn create_trump_marker() -> Image {
        let mut image = Image::new("", VIEW_CENTER, 0.25);
        image.visible = false;
        image
    }

    // pub fn update_cards(&mut self, updates: &mut VecDeque<CardUpdate>, time_delta: f32) {
    //     // Loop until a card needs updating or there are no updates left then break.
    //     // This bypasses needless card updates.
    //     loop {
    //         if let Some(mut update) = updates.pop_front() {
    //             if self.update_card(update) {
    //                 break;
    //             }
    //         } else {
    //             break;
    //         }
    //     }
    // }

    // /// Returns false if the card did not need updating.
    // fn update_card(&mut self, update: CardUpdate) -> bool {
    //     let card_view = self
    //         .card_views
    //         .iter_mut()
    //         .find(|s| s.id == update.id)
    //         .unwrap();

    //     if card_view.update == update {
    //         return false;
    //     }

    //     // This include location, angle, and z_order.
    //     card_view.animate_to(update, 500.0, 6.0);
    //     self.card_views_z_order_dirty = true;

    //     //card_view.face_up = update.face_up;
    //     //card_view.select_state = update.select_state;
    //     card_view.update = update;
    //     true
    // }

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

    fn update_hand(&mut self, p: PlayerId, is_bot: bool) {
        if let Some(hand) = self.hands.get_mut(p) {
            let group_len = hand.len();
            for (idx, card) in hand.iter_mut().enumerate() {
                let update = CardUpdate {
                    group: CardGroup::Hand,
                    group_index: idx,
                    group_len: group_len,
                    player: p,
                    player_len: self.player_count,
                    player_is_bot: is_bot,
                    ..Default::default()
                };
                // This include location, angle, and z_order.
                card.animate_to(update, 500.0, 6.0);
            }
        }
    }

    fn deal_card(&mut self, p: PlayerId, is_bot: bool, id: CardId) {
        if let Some(idx) = self.deck.iter().position(|c| c.id == id) {
            let card = self.deck.remove(idx);
            self.hands[p].push(card);
            self.update_hand(p, is_bot);
        }
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
        match suit {
            Some(suit) => {
                self.trump_marker.set_texture_id(suit.to_string());
                self.trump_marker.visible = true;
            }
            None => {
                // set some kind of no-trump texture
            }
        }
    }

    pub fn get_discard(&mut self, game: &Game) {
        if game.active_player_is_bot() {
            println!("bot choosing discard: {}", game.active_player);
        } else {
            self.discard_panel.visible = true;

            // Set message for eligible cards.
            // for id in game.active_hand() {
            //     if let Some(card) = game.cards.get(*id) {
            //         let card_view = self.card_views.iter_mut().find(|s| s.id == *id).unwrap();
            //         card_view.mouse_up_message = match card.select_state {
            //             SelectState::Selectable => Some(PlayerAction::MoveCardToNest(*id)),
            //             _ => None,
            //         }
            //     }
            // }
        }

        for outline in &mut self.discard_outlines {
            outline.visible = true;
        }
    }

    pub fn end_discard(&mut self) {
        self.discard_panel.visible = false;
        for outline in &mut self.discard_outlines {
            outline.visible = false;
        }
    }

    pub fn get_card_play(&mut self, p: PlayerId, game: &Game) {
        let update = CardUpdate {
            group: CardGroup::TrickActive,
            player_len: game.player_count,
            player: p,
            ..Default::default()
        };
        self.play_outline
            .transform
            .set_translation(update.translation());
        self.play_outline.visible = true;

        // Set message for eligible cards.
        // let p = game.active_player;
        // for id in game.active_hand() {
        //     if let Some(card) = game.cards.get(*id) {
        //         let card_view = self.card_views.iter_mut().find(|s| s.id == *id).unwrap();
        //         card_view.mouse_up_message = match card.select_state {
        //             SelectState::Selectable => Some(PlayerAction::PlayCard(p, *id)),
        //             _ => None,
        //         }
        //     }
        // }
    }

    pub fn end_card_play(&mut self) {
        self.play_outline.visible = false;
    }


    pub fn handle_action_result(&mut self, result: &ActionResult) {
        match result {
            ActionResult::CardsAddedToDeck(_) => todo!(),
            ActionResult::PlayerAdvanced(_) => todo!(),
            ActionResult::DealerAdvanced(_) => todo!(),
            ActionResult::PlayerIsBotUpdated(_) => todo!(),
            ActionResult::CardDealt(p, is_bot, id) => {
                self.deal_card(*p, *is_bot, *id);
            },
            ActionResult::CardTurnedFaceUp(_, _) => todo!(),
            ActionResult::CardDealtToNest(_) => todo!(),
            ActionResult::HandSorted(_, _) => todo!(),
            ActionResult::BidMade(_, _) => todo!(),
            ActionResult::TrumpSet(_) => todo!(),
            ActionResult::NestCardAddedToHand(_, _) => todo!(),
            ActionResult::Discarded(_, _, _) => todo!(),
            ActionResult::CardPlayed(_, _, _) => todo!(),
            ActionResult::TrickAwarded(_, _) => todo!(),
            ActionResult::NextTrickPrepared => todo!(),
           
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
        // for card_view in self.card_views.iter_mut().rev() {
        //     if card_view.handle_mouse_event(event, screen_pt, parent_affine, send_msg) {
        //         send_msg = false;
        //     }
        // }

        !send_msg
    }

    fn update(&mut self, time_delta: f32, app: &mut App) {
        if !self.tex_loader_completed {
            println!("TEX_LOADER updating");
            self.tex_loader_completed = crate::TEX_LOADER.lock().unwrap().update();
        }

        // Update the cards.
        // for card_view in &mut self.card_views {
        //     card_view.update(time_delta, app);
        // }

        // // Sort the cards by z-order if needed.
        // if self.card_views_z_order_dirty {
        //     self.card_views.sort_by(|a, b| a.z_order.cmp(&b.z_order));
        //     self.card_views_z_order_dirty = false;
        // }

        self.fps_update -= time_delta;
    }

    fn draw(&mut self, draw: &mut notan::draw::Draw, parent_affine: &Affine2) {
        //let now = std::time::Instant::now();

        // Images
        self.active_player_marker.draw(draw, parent_affine);
        self.dealer_marker.draw(draw, parent_affine);
        self.trump_marker.draw(draw, parent_affine);
        for outline in &mut self.discard_outlines {
            outline.draw(draw, parent_affine);
        }
        self.play_outline.draw(draw, parent_affine);

        for card in &mut self.deck {
            card.draw(draw, parent_affine);
        }

        for card in &mut self.nest {
            card.draw(draw, parent_affine);
        }

        for p in 0..self.player_count {
            for card in &mut self.hands[p] {
                card.draw(draw, parent_affine);
            }
        }

        for card in &mut self.active_trick {
            if let Some(card) = card {
                card.draw(draw, parent_affine);
            }
        }

        for p in 0..self.player_count {
            for card in &mut self.tricks[p] {
                card.draw(draw, parent_affine);
            }
        }


        // for card_view in &mut self.card_views {
        //     card_view.draw(draw, parent_affine);
        // }

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
