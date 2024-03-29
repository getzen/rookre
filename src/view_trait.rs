use notan::{
    draw::Draw,
    math::{Affine2, Vec2},
    prelude::Graphics,
    Event,
};

// Notan method calls:
// fn event(controller: &mut Controller, event: Event);
// fn update(app: &mut App, controller: &mut Controller);
// fn draw(gfx: &mut Graphics, controller: &mut Controller) {
//     // draw created here:
//     let mut draw = gfx.create_draw();
//     self.view_xyz.draw(&mut draw);
//     // time to render
//     gfx.render(&draw);
// }

pub trait ViewTrait {
    // Create draw at controller level using gfx.create_draw() and conclude drawing
    // there with fx.render(&draw). gfx is passed to views since it has other
    // useful functions.
    fn draw(&mut self, draw: &mut Draw, parent_affine: &Affine2);
    // Sample implementation:
    // if !self.visible {
    //     return;
    // }
    // let (size_x, size_y) = self.transform.draw_size().into();
    // draw.image(&self.texture)
    //     .transform(self.transform.mat3_with_parent(parent_affine))
    //     .size(size_x, size_y);

    // Get time_delta at controller level using app.timer.delta_f32(). No point in
    // calling that function over and over for each view.
    fn update(&mut self, time_delta: f32, _app: &mut notan::app::App) {}

    // fn contains_pt(&self, _mouse_pt: Vec2, _parent_affine: &Affine2) -> bool {
    //     false
    //     // Sample - easiest, for rect shapes
    //     //self.transform.contains_screen_point(screen_pt, parent_affine)

    //     // Sample - for circular shapes
    //     // let pt = self.transform.transform_screen_point(screen_pt, parent_affine);
    //     // pt.distance(self.transform.translation()) <= self.radius
    // }

    fn handle_mouse_event(
        &mut self,
        event: &Event,
        screen_pt: Vec2,
        parent_affine: &Affine2,
        mut _send_msg: bool,
    ) -> bool {
        let mut contains = false;

        // Sample:
        // if self
        //     .transform
        //     .contains_screen_point(screen_pt, parent_affine)
        // {
        //     if send_msg {
        //         self.send_message_for_event(event);
        //     }
        //     contains = true;
        // }

        contains
    }

    fn send_message_for_event(&mut self, event: &Event) -> bool {
        // match event {
        //     Event::MouseUp { .. } => {
        //         if let Some(sender) = &self.sender {
        //             if let Some(message) = self.mouse_up_message {
        //                 sender.send(message).expect("Message send error.");
        //                 return true;
        //             }
        //         }
        //     }
        //     _ => {}
        // }
        false
    }

    // fn keyboard_event_handled...
}
