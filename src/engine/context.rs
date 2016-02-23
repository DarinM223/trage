use events::Events;
use font::FontRenderer;
use score::Score;
use sdl2::render::Renderer;
use sdl2_image;

/// Represents a SDL window to render
pub struct Window {
    pub title: &'static str,
    pub width: u32,
    pub height: u32,
}

/// Contains the main context variables for the game
/// like the renderer and the events triggered
pub struct Context<'a> {
    pub events: Events,
    pub renderer: Renderer<'a>,
    pub window: Window,
    pub font_renderer: FontRenderer,
    pub score: Score,
}

impl<'a> Context<'a> {
    /// Creates a new context given the path for the keyboard configuration
    /// and the sdl renderer
    pub fn new(window: Window, events: Events, renderer: Renderer<'a>) -> Context<'a> {
        sdl2_image::init(sdl2_image::INIT_PNG);

        Context {
            window: window,
            events: events,
            renderer: renderer,
            font_renderer: FontRenderer::new(),
            score: Score::new(),
        }
    }
}

impl<'a> Drop for Context<'a> {
    fn drop(&mut self) {
        sdl2_image::quit();
    }
}
