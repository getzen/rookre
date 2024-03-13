use notan::math::{vec2, Vec2};

use crate::{
    player::PlayerId,
    view_geom::{ViewGeom, CARD_SIZE, VIEW_CENTER},
};

#[derive(Clone, PartialEq)]
pub enum CardGroup {
    Deck,
    NestExchange,
    NestAside,
    Hand,
    TrickActive,
    TrickAside,
}

/// CardLocation contains everything needed for the view to position a card properly.
/// Not all fields are used with all CardGroups.
#[derive(Clone, PartialEq)]
pub struct CardLocation {
    pub group: CardGroup,
    pub group_index: usize,
    pub group_len: usize,
    pub player: PlayerId,
    pub player_len: usize,
    pub player_is_bot: bool,
    pub mouse_over: bool,
}

impl Default for CardLocation {
    fn default() -> Self {
        CardLocation {
            group: CardGroup::Deck,
            group_index: 0,
            group_len: 0,
            player: 0,
            player_len: 0,
            player_is_bot: false,
            mouse_over: false,
        }
    }
}

impl CardLocation {
    pub fn translation(&self) -> Vec2 {
        match &self.group {
            CardGroup::Deck => VIEW_CENTER,
            CardGroup::NestExchange => {
                let x_spacing = CARD_SIZE.0 / 2.0;
                let mut pt = VIEW_CENTER;
                pt.x -= (self.group_len - 1) as f32 * x_spacing / 2.0;
                pt.x += self.group_index as f32 * x_spacing;
                pt
            }
            CardGroup::NestAside => {
                let x_spacing = 10.;
                let mut pt = vec2(730., 730.0);
                pt.x -= (self.group_len - 1) as f32 * x_spacing / 2.0;
                pt.x += self.group_index as f32 * x_spacing;
                pt
            }
            CardGroup::Hand => self.hand_card_translation(),
            CardGroup::TrickActive => {
                let distance_from_center = 100.0;
                let radians = ViewGeom::player_radians_from_center(self.player, self.player_len);
                ViewGeom::position_from(VIEW_CENTER, radians, distance_from_center)
            }
            CardGroup::TrickAside => {
                let distance_from_center = 500.0;
                let radians = ViewGeom::player_radians_from_center(self.player, self.player_len);
                ViewGeom::position_from(VIEW_CENTER, radians, distance_from_center)
            }
        }
    }

    pub fn hand_card_translation(&self) -> Vec2 {
        let max_width = match self.player_is_bot {
            true => 280.,
            false => 500.,
        };
        let max_spacing: f32 = 85.;

        let computed_width = max_width / self.group_len as f32;
        let x_spacing = max_spacing.min(computed_width);

        let mut x_offset = (self.group_len - 1) as f32 * -x_spacing / 2.0;
        x_offset += self.group_index as f32 * x_spacing;

        let distance_from_center = if self.mouse_over { 270.0 } else { 300.0 };

        let radians = ViewGeom::player_radians_from_center(self.player, self.player_len);
        let mut pos = ViewGeom::position_from(VIEW_CENTER, radians, distance_from_center);

        let angle = ViewGeom::player_rotation(self.player, self.player_len);
        pos.x += x_offset * angle.cos();
        pos.y += x_offset * angle.sin();
        pos
    }

    pub fn angle(&self) -> f32 {
        match &self.group {
            CardGroup::Deck => 0.0,
            CardGroup::NestExchange => 0.0,
            CardGroup::NestAside => 0.0,
            CardGroup::Hand => ViewGeom::player_rotation(self.player, self.player_len),
            CardGroup::TrickActive => ViewGeom::player_rotation(self.player, self.player_len),
            CardGroup::TrickAside => ViewGeom::player_rotation(self.player, self.player_len),
        }
    }

    pub fn z_order(&self) -> usize {
        match &self.group {
            CardGroup::Deck => self.group_index,
            CardGroup::NestExchange => self.group_index,
            CardGroup::NestAside => self.group_index,
            CardGroup::Hand => self.group_index + 100,
            CardGroup::TrickActive => self.group_index,
            CardGroup::TrickAside => 200,
        }
    }
}
