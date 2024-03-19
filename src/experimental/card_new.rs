

use std::sync::mpsc::Sender;

use notan::{app::{Color, Graphics, Texture}, draw::{Draw, DrawImages, DrawTransform}, math::{Affine2, Vec2}, Event};

use crate::{animators::{AngleAnimator, TranslationAnimator}, card::Card, game::PlayerAction, texture_loader::{ViewFn, CARD_TEX_SCALE}, transform::Transform, view_geom::{CARD_SIZE, CARD_SIZE_HOVER}, view_trait::ViewTrait};

//pub type CardId = slotmap::DefaultKey;
/// The rank showing on the card face, possibly plus 0.5 points for half-rank.
pub type CardRank = f32;
// The rank according to the game rules. Use a maximum of one decimal place, eg 10.5.
//pub type GameRank = f32;
pub type CardPoints = u8;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CardKind {
    Suited,
    Joker,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CardSuit {
    Club,
    Diamond,
    Heart,
    Spade,
}

//#[derive(Copy, Clone, Debug, PartialEq)]
//pub enum CardExposure {
//    FaceDown, FaceUp, FaceUpHumanOnly
//}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SelectState {
    Selectable,   // Expands a bit in size when mouse over.
    Unselectable, // Normal size and appearance, just unselectable.
    Dimmed,       // Unselectable and the view should shade in gray to show it.
}


pub struct CardNew {
    pub id: CardId,
    // Model
    pub kind: CardKind,
    pub suit: CardSuit,
    pub rank: CardRank, // might vary from what is shown on face
    //pub game_rank: GameRank,
    pub points: CardPoints,
    pub is_trump: bool,
    
    // In between
    pub face_up: bool,
    pub select_state: SelectState,

    // View
    pub view: Option<CardViewNew>,
    
}

impl CardNew {
    pub fn new(kind: CardKind, suit: CardSuit, rank: CardRank, points: CardPoints) -> Self {
        let id = slotmap::DefaultKey::default();
        Self {
            id,
            kind,
            suit,
            rank,
            points,
            is_trump: false,
            face_up: false,
            select_state: SelectState::Unselectable,
            view: None,
        }
    }
}

pub struct CardViewNew {
    pub transform: Transform,
    // Not needed if using various vecs for storage?
    //pub z_order: u8,

    pub face_tex: Texture,
    pub back_tex: Texture,

    pub contains_pt: bool, // needed by draw()

    // Animation
    pub translation_animator: Option<TranslationAnimator>,
    pub angle_animator: Option<AngleAnimator>,

    pub sender: Option<Sender<PlayerAction>>,
    pub mouse_up_message: Option<PlayerAction>,
}

//impl ViewTrait for CardViewNew {
impl CardViewNew {
    pub fn new(card: &Card, gfx: &mut Graphics, sender: Option<Sender<PlayerAction>>) -> Self {
        let face_tex = ViewFn::load_card_texture(gfx, card);

        let transform =
            Transform::from_pos_tex_scale_centered(Vec2::ZERO, &face_tex, CARD_TEX_SCALE, true);

        let back_tex = gfx
            .create_texture()
            .from_image(include_bytes!("assets/cards/back.png"))
            .build()
            .unwrap();

        Self {
            //id: card.id,
            //z_order: 0,
            transform: transform.clone(),

            face_tex,
            back_tex,

            contains_pt: false,

            translation_animator: None,
            angle_animator: None,

            sender,
            mouse_up_message: None,
        }
    }

    pub fn animate_to(&mut self, pt: Vec2, trans_vel: f32, angle: f32, angle_vel: f32) {
        // Create translation animator if needed.
        //let end_pt = location.translation();
        if !self.transform.translation().abs_diff_eq(pt, 0.1) {
            let animator =
                TranslationAnimator::new(self.transform.translation(), pt, trans_vel);
            self.translation_animator = Some(animator);
        }

        // Create angle animator if needed.
        //let end_angle = location.angle();
        if (self.transform.angle() - angle).abs() > 0.01 {
            let animator = AngleAnimator::new(self.transform.angle(), angle, angle_vel);
            self.angle_animator = Some(animator);
        }

        //self.z_order = location.z_order();
        //self.face_down = location.face_down();
        //self.location = location;
    }

    fn update(&mut self, time_delta: f32) {
        if let Some(animator) = &mut self.translation_animator {
            self.transform.set_translation(animator.update(time_delta));
            if animator.completed {
                self.translation_animator = None;
            }
        }

        if let Some(animator) = &mut self.angle_animator {
            self.transform.set_angle(animator.update(time_delta));
            if animator.completed {
                self.angle_animator = None;
            }
        }
    }

    fn contains_pt(&mut self, screen_pt: Vec2, parent_affine: &Affine2) -> bool {
        self.contains_pt = self.transform
            .contains_screen_point(screen_pt, parent_affine);
        self.contains_pt

    }   

    fn send_message_for_event(&mut self, event: &Event) {
        match event {
            Event::MouseUp { .. } => {
                if let Some(sender) = &self.sender {
                    if let Some(message) = &self.mouse_up_message {
                        sender.send(*message).expect("Message send error.");
                    }
                }
                //println!("Card {:?}: mouse up", self.id);
            }
            _ => {}
        }
    }

    fn draw(&mut self, draw: &mut Draw, parent_affine: &Affine2, card: &CardNew) {
        let tex = match card.face_up {
            true => &self.face_tex,
            false => &self.back_tex,
        };

        let mut color = Color::WHITE;

        match card.select_state {
            SelectState::Selectable => match self.contains_pt {
                true => self.transform.set_size(CARD_SIZE_HOVER),
                false => self.transform.set_size(CARD_SIZE),
            },
            SelectState::Unselectable => {
                self.transform.set_size(CARD_SIZE);
            }
            SelectState::Dimmed => {
                self.transform.set_size(CARD_SIZE);
                color = crate::view::LIGHT_GRAY;
            }
        }

        let (size_x, size_y) = self.transform.size().into();

        draw.image(tex)
            .transform(self.transform.mat3_with_parent(parent_affine))
            .size(size_x, size_y)
            .color(color);
    }
}