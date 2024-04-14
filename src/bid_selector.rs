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

pub struct BidSelector {
    pub visible: bool,
    pub transform: Transform,
    pub z_order: usize,
    pub texture: Texture,

    pub pass_button: ImageButton<PlayerAction>,
    suit_buttons: Vec<ImageButton<PlayerAction>>,
}

impl BidSelector {
    pub fn new(suits: Vec<CardSuit>, gfx: &mut Graphics, sender: Sender<PlayerAction>) -> Self {
        let texture = gfx
            .create_texture()
            .from_image(include_bytes!("assets/bid_selector.png"))
            .build()
            .unwrap();

        let trans = Transform::from_pos_tex_scale_centered(
            ViewGeom::bid_view_position(0, 4),
            &texture,
            crate::texture_loader::TEX_SCALE,
            true,
        );

        let pass_button = BidSelector::create_pass_button(sender.clone());
        let suit_buttons = BidSelector::create_suit_buttons(suits, sender.clone());

        Self {
            visible: false,
            transform: trans,
            z_order: 0,
            texture,
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
        if !self.visible {
            return;
        }

        let (size_x, size_y) = self.transform.size().into();
        draw.image(&self.texture)
            .transform(self.transform.mat3_with_parent(parent_affine))
            .size(size_x, size_y);

        let affine = *parent_affine * self.transform.affine2();
        for button in &mut self.suit_buttons {
            button.draw(draw, &affine);
        }
        self.pass_button.draw(draw, &affine);
    }
}
