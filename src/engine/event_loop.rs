use engine::context::Context;
use engine::events::Events;
use engine::view::Actor;
use game::asteroid::Asteroid;
use sdl2;
use sdl2::TimerSubsystem;
use sdl2::pixels::Color;
use sdl2_ttf;

const FRAME_INTERVAL: u32 = 1000 / 60;

enum FrameAction {
    /// Block the event loop 
    Delay,
    /// Continue with the elapsed time
    Continue(f64),
}

struct FrameTimer<'a> {
    timer: &'a mut TimerSubsystem,
    before: u32,
    last_second: u32,
    fps: u16,
    debug: bool,
}

impl<'a> FrameTimer<'a> {
    pub fn new(timer: &'a mut TimerSubsystem, debug: bool) -> FrameTimer<'a> {
        FrameTimer {
            before: timer.ticks(),
            last_second: timer.ticks(),
            timer: timer,
            fps: 0u16,
            debug: debug,
        }
    }

    /// Call this function every frame to limit the frames to a 
    /// certain FPS
    pub fn on_frame(&mut self) -> FrameAction {
        let now = self.timer.ticks();
        let time_change = now - self.before;
        let elapsed = time_change as f64 / 1000.0;

        if time_change < FRAME_INTERVAL {
            self.timer.delay(FRAME_INTERVAL - time_change);
            return FrameAction::Delay;
        }

        self.before = now;
        self.fps += 1;

        if now - self.last_second > 1000 {
            if self.debug {
                println!("FPS: {}", self.fps);
            }

            self.last_second = now;
            self.fps = 0;
        }

        FrameAction::Continue(elapsed)
    }
}

/// Represents a SDL window to render
pub struct Window {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

/// Initializes SDL and creates the window and event loop
pub fn create_event_loop(window: Window) {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let mut timer = sdl_context.timer().unwrap();
    let ttf_context = sdl2_ttf::init();

    let window = video.window(window.title.as_str(), window.width, window.height)
                      .position_centered()
                      .opengl()
                      .build()
                      .unwrap();

    let mut game_context = Context::new(Events::new(sdl_context.event_pump().unwrap(), ""),
                                        window.renderer().accelerated().build().unwrap());

    let mut frame_timer = FrameTimer::new(&mut timer, true);

    let mut asteroid = Asteroid::new(&mut game_context.renderer, FRAME_INTERVAL as f64);

    loop {
        let mut elapsed = 0.0;
        match frame_timer.on_frame() {
            FrameAction::Delay => continue,
            FrameAction::Continue(elpsed) => elapsed = elpsed,
        }

        game_context.events.poll();

        if game_context.events.event_called("QUIT") || game_context.events.event_called("ESC") {
            break;
        }

        game_context.renderer.set_draw_color(Color::RGB(0, 0, 0));
        game_context.renderer.clear();

        asteroid.update(&mut game_context, elapsed);
        asteroid.render(&mut game_context, elapsed);
    }
}
