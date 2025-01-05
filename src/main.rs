use std::{
    io::{stdout, Write}, str::FromStr, sync::{Arc, RwLock}
};

use clap::Parser;
use contrast::round_float;

mod contrast;

#[derive(clap::Parser)]
struct Cli {
    /// List GPU renderers (for the SELECTED_GPU_RENDERER arg)
    #[arg(short, long, default_value_t = false)]
    list_gpu_renderers: bool,
    /// Select your own renderer if you want to
    #[arg(short, long)]
    selected_gpu_renderer: Option<usize>,

    /// Force VSync
    #[arg(short, long, default_value_t = false)]
    vsync: bool,
    /// Force acceleration
    #[arg(short, long, default_value_t = true)]
    acceleration: bool,
}

fn main() {
    let ctx = sdl2::init().unwrap();
    let video = ctx.video().unwrap();
    let parsed = Cli::parse();
    if parsed.list_gpu_renderers {
        for (i, item) in sdl2::render::drivers().enumerate() {
            let mut flags = vec![];
            if item.flags & 0x01 > 0 {
                flags.push("Software Rendered");
            }
            if item.flags & 0x02 > 0 {
                flags.push("Renderer Accelerated");
            }
            if item.flags & 0x04 > 0 {
                flags.push("VSynced");
            }
            if item.flags & 0x08 > 0 {
                flags.push("Support rendering into a texture");
            }
            println!(
                "Renderer #{}:\n   Name: {}\n  Flags: {}",
                i + 1,
                item.name,
                flags.join(", ")
            )
        }
        return;
    }
    let window = video
        .window("Contrasity", 800, 600)
        .position_centered()
        .resizable()
        .allow_highdpi()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas();
    if let Some(index) = parsed.selected_gpu_renderer {
        canvas = canvas.index(index as u32 - 1);
    }
    if parsed.vsync {
        canvas = canvas.present_vsync();
    }
    if parsed.acceleration {
        canvas = canvas.accelerated();
    }
    let mut canvas = canvas.build().unwrap();
    let state = Arc::new(RwLock::new(State::default()));
    receive_stdin_options(Arc::clone(&state));
    let mut event_pump = ctx.event_pump().unwrap();
    let text_rendering = sdl2::ttf::init().unwrap();
    let sample = text_rendering
        .load_font_from_rwops(
            sdl2::rwops::RWops::from_bytes(include_bytes!("./fonts/Sen-Bold.ttf")).unwrap(),
            120,
        )
        .unwrap();
    let lalalala = text_rendering
        .load_font_from_rwops(
            sdl2::rwops::RWops::from_bytes(include_bytes!("./fonts/Sen-Bold.ttf")).unwrap(),
            30,
        )
        .unwrap();
    let fps_font = text_rendering
        .load_font_from_rwops(
            sdl2::rwops::RWops::from_bytes(include_bytes!("./fonts/Sen-Bold.ttf")).unwrap(),
            15,
        )
        .unwrap();
    let texture_creator = canvas.texture_creator();

    

    // fps stuff
    let mut ft = std::time::Instant::now(); // frame time
    let mut fc = 0; // frame count
    let mut fps = 0.0; // frame per sec
    let mut mf = 0.0; // maximum fps
    let mut lf = 0.0; // minimum fps (shows on screen)
    let mut lpf = 0.0; // act as a cache
    let mut lft = std::time::Instant::now(); // minimum frame refresh time thingy

    'run: loop {
        let rect_width = 200;
        let rect_height = 200;
        let rect_x = (canvas.output_size().unwrap().0 as i32 - rect_width) / 2;
        let rect_y = (canvas.output_size().unwrap().1 as i32 - rect_height) / 2;
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'run,
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => break 'run,
                _ => {}
            }
        }
        let reader = state.read().unwrap();
        canvas.set_draw_color(sdl2::pixels::Color::RGB(
            reader.bg.0 as u8,
            reader.bg.1 as u8,
            reader.bg.2 as u8,
        ));
        canvas.clear();

        canvas.set_draw_color(sdl2::pixels::Color::RGB(
            reader.fg.0 as u8,
            reader.fg.1 as u8,
            reader.fg.2 as u8,
        ));
        for offset in 0..5 {
            canvas
                .draw_rect(sdl2::rect::Rect::new(
                    rect_x - offset,
                    rect_y - offset,
                    (rect_width + offset * 2) as u32,
                    (rect_height + offset * 2) as u32,
                ))
                .unwrap();
        }
        let score = contrast::ContrastPasses::get_contrast(reader.bg, reader.fg);
        let rating = contrast::ContrastPasses::get_rating(score);
        let text_surface = sample
            .render("Aa")
            .blended(sdl2::pixels::Color::RGB(
                reader.fg.0 as u8,
                reader.fg.1 as u8,
                reader.fg.2 as u8,
            ))
            .unwrap();
        let text_texture = texture_creator
            .create_texture_from_surface(&text_surface)
            .unwrap();

        let text_width = text_surface.width();
        let text_height = text_surface.height();
        let text_x = rect_x + (rect_width as i32 - text_width as i32) / 2;
        let text_y = rect_y + (rect_height as i32 - text_height as i32) / 2;

        let score_text = lalalala
            .render(&format!("{:.2}", score))
            .blended(if rating[2] == contrast::ContrastPasses::AALarge(true) {
                sdl2::pixels::Color::RGB(reader.fg.0 as u8, reader.fg.1 as u8, reader.fg.2 as u8)
            } else {
                if contrast::ContrastPasses::get_luminance(reader.bg) > 0.5 {
                    sdl2::pixels::Color::RGB(0, 0, 0)
                } else {
                    sdl2::pixels::Color::RGB(255, 255, 255)
                }
            })
            .unwrap();
        let score_text_texture = texture_creator
            .create_texture_from_surface(&score_text)
            .unwrap();
        let ratings_text = lalalala
            .render(&format!("{:?} {:?} {:?}", rating[0], rating[1], rating[2]))
            .blended(if rating[2] == contrast::ContrastPasses::AALarge(true) {
                sdl2::pixels::Color::RGB(reader.fg.0 as u8, reader.fg.1 as u8, reader.fg.2 as u8)
            } else {
                if contrast::ContrastPasses::get_luminance(reader.bg) > 0.5 {
                    sdl2::pixels::Color::RGB(0, 0, 0)
                } else {
                    sdl2::pixels::Color::RGB(255, 255, 255)
                }
            })
            .unwrap();
        let ratings_text_texture = texture_creator
            .create_texture_from_surface(&ratings_text)
            .unwrap();

        canvas.copy(
            &score_text_texture,
            None,
            Some(sdl2::rect::Rect::new(
                (canvas.output_size().unwrap().0 as i32 - score_text.width() as i32) / 2,
                ((canvas.output_size().unwrap().1 as i32 - score_text.height() as i32) / 2) + 150,
                score_text.width(),
                score_text.height(),
            )),
        ).unwrap();
        canvas.copy(
            &ratings_text_texture,
            None,
            Some(sdl2::rect::Rect::new(
                (canvas.output_size().unwrap().0 as i32 - ratings_text.width() as i32) / 2,
                ((canvas.output_size().unwrap().1 as i32 - ratings_text.height() as i32) / 2) + 250,
                ratings_text.width(),
                ratings_text.height(),
            )),
        ).unwrap();

        canvas
            .copy(
                &text_texture,
                None,
                Some(sdl2::rect::Rect::new(
                    text_x,
                    text_y,
                    text_width,
                    text_height,
                )),
            )
            .unwrap();
        let fps_text = fps_font
            .render(&format!("FPS: {}", round_float(fps, 2)))
            .shaded(sdl2::pixels::Color::WHITE, sdl2::pixels::Color::BLACK)
            .unwrap();
        let mf_text = fps_font
            .render(&format!("Maximum FPS: {}", round_float(mf, 2)))
            .shaded(sdl2::pixels::Color::WHITE, sdl2::pixels::Color::BLACK)
            .unwrap();
        let lfp_text = fps_font
            .render(&format!("Minimum FPS: {}", round_float(lf, 2)))
            .shaded(sdl2::pixels::Color::WHITE, sdl2::pixels::Color::BLACK)
            .unwrap();
        canvas
            .copy(
                &texture_creator.create_texture_from_surface(&fps_text).unwrap(),
                None,
                Some(sdl2::rect::Rect::new(
                    0,
                    0,
                    fps_text.width(),
                    fps_text.height(),
                )),
            )
            .unwrap();
        canvas
            .copy(
                &texture_creator.create_texture_from_surface(&mf_text).unwrap(),
                None,
                Some(sdl2::rect::Rect::new(
                    0,
                    20,
                    mf_text.width(),
                    mf_text.height(),
                )),
            )
            .unwrap();
        canvas
            .copy(
                &texture_creator.create_texture_from_surface(&lfp_text).unwrap(),
                None,
                Some(sdl2::rect::Rect::new(
                    0,
                    40,
                    lfp_text.width(),
                    lfp_text.height(),
                )),
            )
            .unwrap();
        canvas.present();
        fc += 1;
        let elapsed_time = ft.elapsed();
        if elapsed_time.as_secs() >= 1 {
            fps = fc as f64 / elapsed_time.as_secs_f64();
            fc = 0;
            ft = std::time::Instant::now();
            if fps > mf {
                mf = fps
            } else if fps < lpf {
                lpf = fps
            }
        }
        let elapsed_time = lft.elapsed();
        if elapsed_time.as_secs() >= 3 {
            lf = lpf;
            lpf = fps;
            lft = std::time::Instant::now();
        }
    }
    state.write().unwrap().exit = true;
}

#[derive(Debug, Clone, Copy)]
struct State {
    fg: (f64, f64, f64),
    bg: (f64, f64, f64),
    exit: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            fg: (0.0, 255.0, 255.0),
            bg: (0.0, 0.0, 0.0),
            exit: false,
        }
    }
}

fn receive_stdin_options(state: Arc<RwLock<State>>) {
    let stdin = std::io::stdin();
    std::thread::spawn(move || while !state.read().unwrap().exit {
        print!("Background: ");
        stdout().flush().unwrap();
        let mut bg = String::new();
        stdin.read_line(&mut bg).unwrap();
        bg = bg.trim().to_string();
        let bged;
        if bg.starts_with("#") {
            let parsed = hex_color::HexColor::from_str(&bg);
            match parsed {
                Ok(color) => {
                    bged = (color.r as f64, color.g as f64, color.b as f64);
                }
                Err(e) => {
                    println!("Invalid background color: {:?}", e);
                    continue;
                }
            }
        } else {
            let gayed = bg.trim().split(' ').collect::<Vec<&str>>();
            if gayed.len() != 3 {
                println!("Invalid background color: invalid length");
                continue;
            }
            bged = (
                gayed[0].parse::<f64>().unwrap(),
                gayed[1].parse::<f64>().unwrap(),
                gayed[2].parse::<f64>().unwrap(),
            );
        }
        print!("Foreground: ");
        stdout().flush().unwrap();
        let mut fg = String::new();
        stdin.read_line(&mut fg).unwrap();
        fg = fg.trim().to_string();
        let fged;
        if fg.starts_with("#") {
            let parsed = hex_color::HexColor::from_str(&fg);
            match parsed {
                Ok(color) => {
                    fged = (color.r as f64, color.g as f64, color.b as f64);
                }
                Err(e) => {
                    println!("Invalid background color: {:?}", e);
                    continue;
                }
            }
        } else {
            let gayed = fg.trim().split(' ').collect::<Vec<&str>>();
            if gayed.len() != 3 {
                println!("Invalid background color: invalid length");
                continue;
            }
            fged = (
                gayed[0].parse::<f64>().unwrap(),
                gayed[1].parse::<f64>().unwrap(),
                gayed[2].parse::<f64>().unwrap(),
            );
        }
        let mut b = state.write().unwrap();
        b.fg = fged;
        b.bg = bged;
    });
}
