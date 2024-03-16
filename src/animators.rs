use notan::math::{vec2, Vec2};


pub struct TranslationAnimator {
    pub current: Vec2,
    end: Vec2,
    vector: Vec2,
    time_remaining: f32,
    pub completed: bool,
}

impl TranslationAnimator {
    /// Set translation change to occur at the given velocity.
    pub fn new(start: Vec2, end: Vec2, velocity: f32) -> Self {
        let radians = (end.y - start.y).atan2(end.x - start.x);
        Self {
            current: start,
            end,
            vector: vec2(radians.cos(), radians.sin()) * velocity,
            time_remaining: start.distance(end) / velocity,
            completed: false,
        }
    }

    pub fn update(&mut self, time_delta: f32) -> Vec2 {
        self.time_remaining -= time_delta;
        if self.time_remaining <= 0.0 {
            self.current = self.end;
            self.time_remaining = 0.0;
            self.completed = true;
        } else {
            self.current += self.vector * time_delta;
        }
        self.current
    }
}


pub struct AngleAnimator {
    pub current: f32,
    end: f32,
    vector: f32,
    time_remaining: f32,
    pub completed: bool,
}

impl AngleAnimator {
    /// Set angle change to occur at the given velocity.
    pub fn new(start: f32, end: f32, velocity: f32) -> Self {
        let angle_diff = AngleAnimator::shortest_angle_diff(start, end);
        let vector = velocity * angle_diff.signum();
        Self {
            current: start,
            end,
            vector,
            time_remaining: angle_diff.abs() / velocity,
            completed: false,
        }
    }

    pub fn update(&mut self, time_delta: f32) -> f32 {
        self.time_remaining -= time_delta;
        if self.time_remaining <= 0.0 {
            self.current = self.end;
            self.time_remaining = 0.0;
            self.completed = true;
        } else {
            self.current += self.vector * time_delta;
        }
        self.current
    }

    /// Returns the shortest angle difference in radians, depending on whether one goes
    /// or counterclockwise to get from start to end.
    fn shortest_angle_diff(mut start: f32, mut end: f32) -> f32 {
        let double_pi = std::f32::consts::PI * 2.0;
        start = start % double_pi;
        end = end % double_pi;

        let mut diff = end - start;
        if diff > std::f32::consts::PI {
            diff -= double_pi;
        }
        if diff < -std::f32::consts::PI {
            diff += double_pi;
        }
        diff
    }
}
