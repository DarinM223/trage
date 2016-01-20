use engine::context::Context;
use engine::view::Actor;
use engine::sprite::Renderable;
use engine::sprite::Rectangle;
use sdl2::rect::Rect;

const ASTEROID_SIDE: u32 = 96;

spritesheet! {
    name: Asteroid,
    state: AsteroidState,
    path: "./assets/asteroid.png",
    sprite_side: 96,
    sprites_in_row: 21,
    animations: {
        Spinning: 1..143
    },
    properties: {
        curr_state: AsteroidState => AsteroidState::Spinning,
        rect: Rectangle => Rectangle::new(64, 64, ASTEROID_SIDE, ASTEROID_SIDE),
        vel: f64 => 0.0
    }
}

impl Actor for Asteroid {
    fn update(&mut self, context: &mut Context, elapsed: f64) {
        // self.rect.x -= (elapsed * self.vel) as i32;
        self.animations.get_mut(&self.curr_state).unwrap().add_time(elapsed);
    }

    fn render(&mut self, context: &mut Context, elapsed: f64) {
        self.animations.get_mut(&self.curr_state).unwrap().render(&mut context.renderer, self.rect.to_sdl().unwrap());
    }
}
