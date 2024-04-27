use std::sync::mpsc::Sender;

use notan::draw::*;
use notan::math::vec2;
use notan::math::Affine2;
use notan::math::Vec2;
use notan::prelude::*;

use crate::card::CardSuit;
use crate::game::PlayerAction;
use crate::image_button::ImageButton;
use crate::transform::Transform;
use crate::view_geom::ViewGeom;
use crate::view_trait::ViewTrait;
use crate::TEX_LOADER;

pub struct BidSelector {
    pub visible: bool,
    pub transform: Transform,
    pub z_order: usize,
    
    tex_id: String,
    tex: Option<Texture>,
    tex_size_mult: f32,

    pub pass_button: ImageButton<PlayerAction>,
    suit_buttons: Vec<ImageButton<PlayerAction>>,
}

impl BidSelector {
    pub fn new(suits: Vec<CardSuit>, tex_id: &str, tex_size_mult: f32, translation: Vec2, sender: Sender<PlayerAction>) -> Self {
        let transform = Transform::from_translation(translation);
       

        let pass_button = BidSelector::create_pass_button(sender.clone());
        let suit_buttons = BidSelector::create_suit_buttons(suits, sender.clone());

        Self {
            visible: false,
            transform,
            z_order: 0,
            tex_id: tex_id.to_string(),
            tex: None,
            tex_size_mult,
            pass_button,
            suit_buttons,
        }
    }

    fn create_pass_button(sender: Sender<PlayerAction>) -> ImageButton<PlayerAction> {
        let trans = vec2(266., 55.);
        let mut button = ImageButton::new(
            trans,
            "pass_enabled",
            "pass_mouse_over",
            "",
            0.5,
            "",
            Some(sender),
        );
        button.mouse_up_message = Some(PlayerAction::MakeBid(None));
        button
    }

    fn create_suit_buttons(
        suits: Vec<CardSuit>,
        sender: Sender<PlayerAction>,
    ) -> Vec<ImageButton<PlayerAction>> {
        let mut buttons = Vec::new();

        for (idx, suit) in suits.iter().enumerate() {
            let (enabled, mouse_over) = match suit {
                CardSuit::Club => ("club", "club_mouse_over"),
                CardSuit::Diamond => ("diamond", "diamond_mouse_over"),
                CardSuit::Heart => ("heart", "heart_mouse_over"),
                CardSuit::Spade => ("spade", "spade_mouse_over"),
                CardSuit::Joker => panic!(),
            };

            let trans = vec2(32. + 50. * idx as f32, 53.);
            let mut button = ImageButton::new(
                trans,
                enabled,
                mouse_over,
                "",
                0.18,
                "",
                Some(sender.clone()),
            );
            button.mouse_up_message = Some(PlayerAction::MakeBid(Some(*suit)));
            buttons.push(button);
        }
        buttons
    }
}

impl ViewTrait for BidSelector {
    fn handle_mouse_event(
        &mut self,
        event: &Event,
        screen_pt: Vec2,
        parent_affine: &Affine2,
        mut send_msg: bool,
    ) -> bool {
        if !self.visible {
            return false;
        }

        let mut contains = false;

        let affine = *parent_affine * self.transform.affine2();

        for button in &mut self.suit_buttons {
            if button.handle_mouse_event(event, screen_pt, &affine, send_msg) {
                send_msg = false;
                contains = true;
            }
        }

        if self
            .pass_button
            .handle_mouse_event(event, screen_pt, &affine, send_msg)
        {
            contains = true;
        }

        contains
    }

    fn draw(&mut self, draw: &mut Draw, parent_affine: &Affine2) {
        if self.tex.is_none() && !self.tex_id.is_empty() {
            if let Some(texture) = TEX_LOADER.lock().unwrap().get_tex(&self.tex_id) {
                self.tex = Some(texture.clone());
                let size: Vec2 = texture.size().into();
                self.transform.set_size(size * self.tex_size_mult);
            } else {
                return;
            }
        }
        
        if !self.visible {
            return;
        }

        if let Some(tex) = &self.tex {
            let (size_x, size_y) = self.transform.size().into();
                draw.image(tex)
                .transform(self.transform.mat3_with_parent(parent_affine))
                .size(size_x, size_y);
        }

        let affine = *parent_affine * self.transform.affine2();
        for button in &mut self.suit_buttons {
            button.draw(draw, &affine);
        }
        self.pass_button.draw(draw, &affine);
    }
}
