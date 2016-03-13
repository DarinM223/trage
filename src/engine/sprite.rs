use cache;
use collision;
use collision::{BoundingBox, Collision, CollisionSide};
use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture};
use sdl2_image::LoadTexture;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;
use std::rc::Rc;
use viewport::Viewport;

/// The direction that a sprite is facing
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Direction {
    Left,
    Right,
}

pub trait Renderable {
    fn render(&self, renderer: &mut Renderer, dest: Rect);
}

/// A mutable rectangle for a sprite so it can be moved around
#[derive(Clone, PartialEq)]
pub struct SpriteRectangle {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl SpriteRectangle {
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> SpriteRectangle {
        SpriteRectangle {
            x: x,
            y: y,
            w: w,
            h: h,
        }
    }

    pub fn from_rect(rect: Rect) -> SpriteRectangle {
        SpriteRectangle {
            x: rect.x(),
            y: rect.y(),
            w: rect.width(),
            h: rect.height(),
        }
    }

    /// Returns a SDL Rect created from the SpriteRectangle
    /// Used for rendering SpriteRectangles in SDL
    pub fn to_sdl(&self) -> Option<Rect> {
        Rect::new(self.x, self.y, self.w, self.h).unwrap()
    }
}

/// A sprite data type that uses reference counting
/// to reuse the texture on multiple sub-sprites
#[derive(Clone)]
pub struct Sprite {
    tex: Rc<RefCell<Texture>>,
    src: Rect,
}

impl Sprite {
    pub fn new(texture: Texture) -> Sprite {
        let tex_query = texture.query();

        Sprite {
            tex: Rc::new(RefCell::new(texture)),
            src: Rect::new_unwrap(0, 0, tex_query.width, tex_query.height),
        }
    }

    /// Loads a new sprite from a path string to a sprite image file
    pub fn load(renderer: &Renderer, path: &str) -> Option<Sprite> {
        let sprite_cache = cache::sprite_cache();

        // if sprite is cached, return from cache
        if let Ok(ref cache) = sprite_cache.cache.lock() {
            if let Some(sprite) = cache.get(path).map(|sprite| sprite.clone()) {
                return Some(sprite);
            }
        }

        // otherwise load sprite from texture
        let sprite = renderer.load_texture(Path::new(path)).ok().map(Sprite::new);

        // cache result if successful
        if let Some(ref sprite) = sprite {
            if let Ok(ref mut cache) = sprite_cache.cache.lock() {
                cache.insert(path.to_owned(), sprite.clone());
            }
        }

        sprite
    }

    /// Returns a sub-sprite from a rectangle region of the original sprite
    pub fn region(&self, rect: Rect) -> Option<Sprite> {
        let new_src = Rect::new_unwrap(rect.x() + self.src.x(),
                                       rect.y() + self.src.y(),
                                       rect.width(),
                                       rect.height());

        if collision::rect_contains_rect(self.src, new_src) {
            Some(Sprite {
                tex: self.tex.clone(),
                src: new_src,
            })
        } else {
            None
        }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.src.width(), self.src.height())
    }
}

impl Renderable for Sprite {
    fn render(&self, renderer: &mut Renderer, dest: Rect) {
        renderer.copy(&mut self.tex.borrow_mut(), Some(self.src), Some(dest));
    }
}

/// Represents an animated sprite with multiple frames
#[derive(Clone)]
pub struct AnimatedSprite {
    /// frames that will be rendered
    frames: Vec<Sprite>,
    /// time between frames
    frame_delay: f64,
    /// total time sprite has been alive
    current_time: f64,
}

impl AnimatedSprite {
    fn new(frames: Vec<Sprite>, frame_delay: f64) -> AnimatedSprite {
        AnimatedSprite {
            frames: frames,
            frame_delay: frame_delay,
            current_time: 0.0,
        }
    }

    pub fn with_fps(frames: Vec<Sprite>, fps: f64) -> AnimatedSprite {
        if fps == 0.0 {
            panic!("Passed 0 to AnimatedSprite::with_fps");
        }

        AnimatedSprite::new(frames, 1.0 / fps)
    }

    fn set_frame_delay(&mut self, frame_delay: f64) {
        self.frame_delay = frame_delay;
    }

    fn set_fps(&mut self, fps: f64) {
        if fps == 0.0 {
            panic!("Passed 0 to AnimatedSprite::set_fps");
        }

        self.set_frame_delay(1.0 / fps);
    }

    /// Updates the animated sprite with the elapsed time
    pub fn add_time(&mut self, elapsed: f64) {
        self.current_time += elapsed;

        if self.current_time < 0.0 {
            self.current_time = (self.frames.len() - 1) as f64 * self.frame_delay;
        }
    }
}

impl Renderable for AnimatedSprite {
    fn render(&self, renderer: &mut Renderer, dest: Rect) {
        if self.frames.len() == 0 {
            panic!("There has to be at least one frame!");
        }

        let current_frame = (self.current_time / self.frame_delay) as usize % self.frames.len();

        let frame = &self.frames[current_frame];
        frame.render(renderer, dest);
    }
}

pub struct AnimationData {
    /// The width of each animation frame
    pub width: u32,
    /// The height of each animation frame
    pub height: u32,
    /// The number of frames in each row of the spritesheet
    pub sprites_in_row: i32,
    /// The path to the spritesheet file
    pub path: &'static str,
}

pub struct Animation {
    data: AnimationData,
    spritesheet: Sprite,
}

impl Animation {
    pub fn new(data: AnimationData, renderer: &mut Renderer) -> Animation {
        let spritesheet = match Sprite::load(renderer, data.path) {
            Some(spritesheet) => spritesheet,
            None => panic!("{} is not a valid path", data.path),
        };

        Animation {
            data: data,
            spritesheet: spritesheet,
        }
    }

    pub fn range(&self, start: i32, end: i32) -> Vec<Sprite> {
        (start..end)
            .map(|elem| {
                let x = elem % self.data.sprites_in_row;
                let y = elem / self.data.sprites_in_row;

                let region = Rect::new_unwrap((self.data.width as i32) * x,
                                              (self.data.height as i32) * y,
                                              self.data.width,
                                              self.data.height);
                self.spritesheet.region(region)
            })
            .flat_map(|sprite| sprite)
            .collect()
    }
}

pub struct AnimationManager<State> {
    fps: f64,
    animations: HashMap<State, (AnimatedSprite, BoundingBox)>,
    /// Saves the current state for better performance
    curr_state: Option<State>,
    /// Saves the current bounding box for better performance
    curr_bbox: Option<BoundingBox>,
    /// Saves the current animation for better performance
    curr_anim: Option<AnimatedSprite>,
}

impl<State> AnimationManager<State> where State: Clone + Eq + Hash
{
    pub fn new(fps: f64) -> AnimationManager<State> {
        AnimationManager {
            fps: fps,
            animations: HashMap::new(),
            curr_state: None,
            curr_bbox: None,
            curr_anim: None,
        }
    }

    pub fn add(&mut self, s: State, anims: Vec<Sprite>, bound: BoundingBox) {
        self.animations.insert(s, (AnimatedSprite::with_fps(anims, self.fps), bound));
    }

    fn set_state(&mut self, s: &State) {
        // Insert the saved bounding box and animation back into the hashmap
        if let (Some(state), Some(bbox), Some(anim)) = (self.curr_state.take(),
                                                        self.curr_bbox.take(),
                                                        self.curr_anim.take()) {
            self.animations.insert(state, (anim, bbox));
        }

        // Save the new state
        self.curr_state = Some(s.clone());
        if let Some((anim, bbox)) = self.animations.remove(s) {
            self.curr_bbox = Some(bbox);
            self.curr_anim = Some(anim);
        }
    }

    /// Returns an immutable reference to the animation for the given state
    pub fn anim(&mut self, s: &State) -> Option<&AnimatedSprite> {
        if let Some(ref state) = self.curr_state {
            if state == s {
                return self.curr_anim.as_ref();
            }
        }

        self.set_state(s);
        self.curr_anim.as_ref()
    }

    /// Returns a mutable reference to the animation for the given state
    pub fn anim_mut(&mut self, s: &State) -> Option<&mut AnimatedSprite> {
        if let Some(ref state) = self.curr_state {
            if state == s {
                return self.curr_anim.as_mut();
            }
        }

        self.set_state(s);
        self.curr_anim.as_mut()
    }

    /// Returns an immutable reference to the bounding box for the given state
    pub fn bbox(&mut self, s: &State) -> Option<&BoundingBox> {
        if let Some(ref state) = self.curr_state {
            if state == s {
                return self.curr_bbox.as_ref();
            }
        }

        self.set_state(s);
        self.curr_bbox.as_ref()
    }

    /// Returns a mutable reference to the bounding box for the given state
    pub fn bbox_mut(&mut self, s: &State) -> Option<&mut BoundingBox> {
        if let Some(ref state) = self.curr_state {
            if state == s {
                return self.curr_bbox.as_mut();
            }
        }

        self.set_state(s);
        self.curr_bbox.as_mut()
    }

    /// Checks if the animation at the state collides with another bounding box
    /// and returns the side of the collision if it happens
    pub fn collides_with(&mut self,
                         s: &State,
                         other_bbox: &Option<BoundingBox>)
                         -> Option<CollisionSide> {
        if let Some(bounding_box) = self.bbox(s) {
            if let Some(ref bbox) = *other_bbox {
                return bounding_box.collides_with(bbox);
            }
        }

        None
    }

    /// Adds time to the current animation
    pub fn add_time(&mut self, s: &State, elapsed: f64) {
        if let Some(animation) = self.anim_mut(s) {
            animation.add_time(elapsed);
        }
    }

    /// Changes the bounding box of the current animation
    pub fn change_pos(&mut self, s: &State, rect: &SpriteRectangle) {
        if let Some(bounding_box) = self.bbox_mut(s) {
            bounding_box.change_pos(rect);
        }
    }

    /// Renders an animation in the manager
    pub fn render(&mut self,
                  s: &State,
                  rect: &SpriteRectangle,
                  viewport: &mut Viewport,
                  renderer: &mut Renderer,
                  debug: bool) {
        if debug {
            if let Some(bounding_box) = self.bbox(s) {
                match *bounding_box {
                    BoundingBox::Rectangle(ref rect) => {
                        renderer.set_draw_color(::sdl2::pixels::Color::RGB(230, 230, 230));
                        let (rx, ry) = viewport.relative_point((rect.x, rect.y));
                        let rect = Rect::new_unwrap(rx, ry, rect.w, rect.h);
                        renderer.fill_rect(rect);
                    }
                }
            }
        }

        let (rx, ry) = viewport.relative_point((rect.x, rect.y));
        let rect = Rect::new_unwrap(rx, ry, rect.w, rect.h);

        if let Some(animation) = self.anim_mut(s) {
            animation.render(renderer, rect);
        } else {
            panic!("Could not find animation");
        }
    }
}

/// Macro for easily creating block classes
/// Example:
/// block! {
///     name: GrassBlock, // the name of the block
///     path: "assets/spritesheet.png", // the path of the spritesheet
///     index: 20, // the sprite index inside the spritesheet
///     width: 5, // width of block
///     height: 5, // height of block
///     sprites_in_row: 10, // number of blocks in the spritesheet in a row
///     size: 20, // size of the rendered block
/// }
#[macro_export]
macro_rules! block {
    (
        name: $name:ident,
        path: $path:expr,
        index: $index:expr,
        width: $width:expr,
        height: $height:expr,
        sprites_in_row: $sprites_in_row:expr,
        size: $size:expr,
    ) => {
        pub struct $name {
            id: i32,
            pub rect: ::engine::sprite::SpriteRectangle,
            pub sprite: ::engine::sprite::Sprite,
        }

        impl $name {
            pub fn new(id: i32,
                       position: (i32, i32),
                       renderer: &mut ::sdl2::render::Renderer,
                       _fps: f64)
                       -> $name {
                let anim_data = ::engine::sprite::AnimationData {
                    width: $width,
                    height: $height,
                    sprites_in_row: $sprites_in_row,
                    path: $path,
                };

                let anim = ::engine::sprite::Animation::new(anim_data, renderer);
                let mut sprite_anims = anim.range($index, $index + 1);
                let sprite = sprite_anims.pop().unwrap();

                $name {
                    id: id,
                    rect: ::engine::sprite::SpriteRectangle::new(position.0,
                                                                 position.1,
                                                                 $size,
                                                                 $size),
                    sprite: sprite,
                }
            }
        }

        impl ::engine::view::Actor for $name {
            fn on_collision(&mut self,
                            _c: &mut ::engine::context::Context,
                            _a: ::engine::view::ActorData,
                            _s: ::engine::collision::CollisionSide)
                            -> ::engine::view::ActorAction {
                ::engine::view::ActorAction::None
            }

            #[allow(unused_imports)]
            fn collides_with(&mut self,
                             other_actor: &::engine::view::ActorData)
                             -> Option<::engine::collision::CollisionSide> {
                use ::engine::collision::Collision;
                use ::engine::sprite::SpriteRectangle;

                self.rect.collides_with(other_actor.rect)
            }

            fn update(&mut self,
                      _context: &mut ::engine::context::Context,
                      _elapsed: f64)
                      -> ::engine::view::ActorAction {
                ::engine::view::ActorAction::None
            }

            #[allow(unused_imports)]
            fn render(&mut self,
                      context: &mut ::engine::context::Context,
                      viewport: &mut ::engine::viewport::Viewport,
                      _elapsed: f64) {
                use ::engine::sprite::Renderable;

                let (rx, ry) = viewport.relative_point((self.rect.x, self.rect.y));
                let rect = ::sdl2::rect::Rect::new_unwrap(rx, ry, self.rect.w, self.rect.h);

                self.sprite.render(&mut context.renderer, rect);
            }

            fn data(&mut self) -> ::engine::view::ActorData {
                ::engine::view::ActorData {
                    id: self.id,
                    state: 0 as u32,
                    damage: 0,
                    checks_collision: false,
                    rect: self.rect.to_sdl().unwrap(),
                    bounding_box: Some(::engine::collision::BoundingBox::Rectangle(
                            self.rect.clone())),
                    actor_type: ::engine::view::ActorType::Block,
                }
            }
        }
    }
}
