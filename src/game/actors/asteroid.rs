use engine::context::Context;
use engine::physics::collision::{Collision, CollisionSide};
use engine::physics::vector::Vector2D;
use engine::sprite::Renderable;
use engine::sprite::SpriteRectangle;
use engine::view::{Actor, ActorAction};
use engine::viewport::Viewport;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

const ASTEROID_SIDE: u32 = 96;
const ASTEROID_X_MAXSPEED: f64 = 10.0;
const ASTEROID_Y_MAXSPEED: f64 = 15.0;
const ASTEROID_ACCELERATION: f64 = 0.2;

spritesheet! {
    name: Asteroid,
    state: AsteroidState,
    path: "./assets/asteroid.png",
    sprite_side: 96,
    sprites_in_row: 21,
    animations: {
        Jumping: 1..2,
        Idle: 1..143
    },
    properties: {
        curr_state: AsteroidState => AsteroidState::Jumping,
        curr_speed: Vector2D => Vector2D { x: 0.0, y: 0.0 },
        rect: SpriteRectangle => SpriteRectangle::new(64, 64, ASTEROID_SIDE, ASTEROID_SIDE)
    }
}

impl Actor for Asteroid {
    fn update(&mut self,
              context: &mut Context,
              other_actors: Vec<&mut Box<Actor>>,
              elapsed: f64)
              -> Option<ActorAction> {
        let max_y_speed = match self.curr_state {
            AsteroidState::Jumping => ASTEROID_Y_MAXSPEED,
            AsteroidState::Idle => 0.0,
        };

        let max_x_speed = if context.events.event_called("RIGHT") {
            ASTEROID_X_MAXSPEED
        } else if context.events.event_called("LEFT") {
            -ASTEROID_X_MAXSPEED
        } else {
            0.0
        };

        if context.events.event_called_once("SPACE") {
            match self.curr_state {
                AsteroidState::Jumping => {}
                AsteroidState::Idle => {
                    self.curr_speed.y = -100.0;
                    self.curr_state = AsteroidState::Jumping;
                }
            }
        }

        let target_speed = Vector2D {
            x: max_x_speed,
            y: max_y_speed,
        };

        self.curr_speed = (ASTEROID_ACCELERATION * target_speed) +
                          ((1.0 - ASTEROID_ACCELERATION) * self.curr_speed);

        self.rect.x += self.curr_speed.x as i32;
        self.rect.y += self.curr_speed.y as i32;

        match self.curr_state {
            AsteroidState::Jumping => self.rect.y += self.curr_speed.y as i32,
            AsteroidState::Idle => {}
        }

        self.rect.x += self.curr_speed.x as i32;

        let mut grounded = false;
        for actor in other_actors {
            if self.rect.collides_with(actor.bounding_box()) == Some(CollisionSide::Left) {
                while self.rect.collides_with(actor.bounding_box()) == Some(CollisionSide::Left) {
                    self.rect.x -= 1;
                }
            } else if self.rect.collides_with(actor.bounding_box()) == Some(CollisionSide::Right) {
                while self.rect.collides_with(actor.bounding_box()) == Some(CollisionSide::Right) {
                    self.rect.x += 1;
                }
            } else if self.rect.collides_with(actor.bounding_box()) == Some(CollisionSide::Bottom) {
                if self.curr_state == AsteroidState::Jumping {
                    self.curr_state = AsteroidState::Idle;
                }

                while self.rect.collides_with(actor.bounding_box()) == Some(CollisionSide::Bottom) {
                    self.rect.y -= 1;
                }
                self.rect.y += 1;

                grounded = true;
                break;
            }
        }

        if !grounded && self.curr_state == AsteroidState::Idle {
            self.curr_state = AsteroidState::Jumping;
        }

        // Update sprite animation
        if let Some(animation) = self.animations.get_mut(&self.curr_state) {
            animation.add_time(elapsed);
        }

        None
    }

    fn render(&mut self, context: &mut Context, viewport: &mut Viewport, elapsed: f64) {
        let (rx, ry) = viewport.relative_point((self.rect.x, self.rect.y));
        let rect = Rect::new_unwrap(rx, ry, self.rect.w, self.rect.h);

        // Follow the asteroid
        viewport.set_position((self.rect.x, self.rect.y));

        // Render sprite animation
        if let Some(animation) = self.animations.get_mut(&self.curr_state) {
            animation.render(&mut context.renderer, rect);
        }

        // Draw rectangle outline for checking collision detection
        context.renderer.set_draw_color(Color::RGB(0, 255, 0));
        context.renderer.draw_rect(rect);
    }

    fn set_position(&mut self, position: (i32, i32)) {
        self.rect.x = position.0;
        self.rect.y = position.1;
    }

    fn bounding_box(&self) -> Rect {
        self.rect.to_sdl().unwrap()
    }

    fn position(&self) -> (i32, i32) {
        (self.rect.x, self.rect.y)
    }
}
