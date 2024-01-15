mod mandelbrot;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

fn new_fractal(w: u32, h: u32) -> mandelbrot::Data {
    let state = mandelbrot::State {
        width: w,
        height: h,
        max_iterations: 500,
        scale: 2.0,
        center: mandelbrot::Complex64::new(0., 0.),
        fractal_type: mandelbrot::FractalType::Mandelbrot,
    };
    mandelbrot::Data::new(state)
}

fn render_image_to_surface(fractal: &mandelbrot::Data, surface: &mut sdl2::surface::Surface) {
    surface.with_lock_mut(|buffer| {
        let mut offset = 0;
        let scale_factor = 255.0/fractal.state.max_iterations as f64;
        fractal.fractal_data.iter().for_each(|row| {
            row.iter().for_each(|entry| {
                if entry.escape >= fractal.state.max_iterations {
                    buffer[offset] = 0;
                    buffer[offset+1] = 0;
                    buffer[offset+2] = 0;
                    buffer[offset+3] = 0;
                } else {
                    let val = ((20 + (entry.escape as f64 * scale_factor) as i32) & 0xff) as u8;
                    buffer[offset] = val;
                    buffer[offset+1] = val;
                    buffer[offset+2] = val;
                    buffer[offset+2] = val;
                }
                offset += 4;
            })
        });
    });
}

fn main() {
    let context = sdl2::init().unwrap();
    let video = context.video().unwrap();

    let window = video.window("Fractals", 1024, 768)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut events = context.event_pump().unwrap();
    let mut i = 0;
    let mut done = false;
    let mut do_redraw = true;

    let mut fractal = new_fractal(canvas.window().size().0, canvas.window().size().1);
    fractal.state.center.re -= 0.5;

    let mut surface = sdl2::surface::Surface::new(fractal.state.width, fractal.state.height, sdl2::pixels::PixelFormatEnum::RGB888).unwrap();

    while !done {
        canvas.clear();

        for event in events.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    done = true;
                }
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    done = true;
                }
                Event::KeyDown { keycode: Some(Keycode::Plus), .. } => {
                    fractal.state.scale = 0.9 * fractal.state.scale;
                    do_redraw = true;
                }
                Event::KeyDown { keycode: Some(Keycode::Equals), .. } => {
                    fractal.state.scale = 0.9 * fractal.state.scale;
                    do_redraw = true;
                }
                Event::KeyDown { keycode: Some(Keycode::Minus), .. } => {
                    fractal.state.scale = 1.1 * fractal.state.scale;
                    do_redraw = true;
                }
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    fractal.state.center.re -= 0.1*fractal.state.scale;
                    do_redraw = true;
                }
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    fractal.state.center.re += 0.1*fractal.state.scale;
                    do_redraw = true;
                }
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    fractal.state.center.im += 0.1*fractal.state.scale;
                    do_redraw = true;
                }
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    fractal.state.center.im -= 0.1*fractal.state.scale;
                    do_redraw = true;
                }
                Event::KeyDown { keycode: Some(Keycode::LeftBracket), .. } => {
                    fractal.state.max_iterations += 25;
                    do_redraw = true;
                }
                Event::KeyDown { keycode: Some(Keycode::RightBracket), .. } => {
                    if fractal.state.max_iterations > 200 {
                        fractal.state.max_iterations += 25;
                        do_redraw = true;
                    }
                }
                Event::KeyDown { keycode: Some(Keycode::J), .. } => {
                    fractal.state.fractal_type = match fractal.state.fractal_type {
                        mandelbrot::FractalType::Mandelbrot => mandelbrot::FractalType::Julia(fractal.state.center),
                        mandelbrot::FractalType::Julia(_) => mandelbrot::FractalType::Mandelbrot,
                    };
                    do_redraw = true;
                }
                _ => {}
            }
        }
        if do_redraw {
            mandelbrot::compute_mandelbrot(&mut fractal);
            render_image_to_surface(&fractal, &mut surface);

            let r1 = sdl2::rect::Rect::new(0, 0, fractal.state.width, fractal.state.height);
            let r2 = sdl2::rect::Rect::new(0, 0, fractal.state.width, fractal.state.height);
            let mut dest = canvas.window().surface(&events).unwrap();
            surface.blit(
                r1, &mut dest, r2).unwrap();
            do_redraw = false;
            dest.update_window().unwrap();
        }

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }



}

