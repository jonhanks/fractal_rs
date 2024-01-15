mod mandelbrot;
mod palette;

use crate::mandelbrot::FractalType;
use num_complex::Complex64;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

fn cycle_palette(p: palette::Palette) -> palette::Palette {
    return match p.palette_type {
        palette::PaletteType::BW => palette::new_color1_lin(),
        palette::PaletteType::Color1Lin => palette::new_color1_mod(),
        palette::PaletteType::Color1Mod => palette::new_bw(),
    };
}

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

fn u8slice_to_u32(input: &mut [u8]) -> &mut [u32] {
    if input.len() == 0 || input.len() % 4 != 0 {
        panic!("invalid length for u8 slice, not a multiple of 4");
    }
    let count = input.len() / 4;
    unsafe {
        let p = input.as_mut_ptr() as *mut u32;
        std::slice::from_raw_parts_mut(p, count)
    }
}

fn render_image_linear(fractal: &mandelbrot::Data, buffer: &mut [u32], pal: &palette::PaletteData) {
    let mut offset = 0;
    let scale_factor = (pal.len() - 1) as f64 / fractal.state.max_iterations as f64;
    fractal.fractal_data.iter().for_each(|row| {
        row.iter().for_each(|entry| {
            if entry.escape >= fractal.state.max_iterations {
                buffer[offset] = 0;
            } else {
                let val = (entry.escape as f64 * scale_factor) as usize;
                buffer[offset] = pal[val];
            }
            offset += 1;
        })
    });
}

fn render_image_modulus(
    fractal: &mandelbrot::Data,
    buffer: &mut [u32],
    pal: &palette::PaletteData,
) {
    let mut offset = 0;
    let pal_len = pal.len();
    fractal.fractal_data.iter().for_each(|row| {
        row.iter().for_each(|entry| {
            if entry.escape >= fractal.state.max_iterations {
                buffer[offset] = 0;
            } else {
                buffer[offset] = pal[entry.escape as usize % pal_len];
            }
            offset += 1;
        });
    });
}

fn render_image_to_surface(
    fractal: &mandelbrot::Data,
    surface: &mut sdl2::surface::Surface,
    pal: &palette::Palette,
) {
    surface.with_lock_mut(|buffer| {
        let mut buffer = u8slice_to_u32(buffer);
        match pal.color_mode {
            palette::ColorMode::LinearScale => render_image_linear(fractal, buffer, &pal.palette),
            palette::ColorMode::Modulus => render_image_modulus(fractal, buffer, &pal.palette),
        };
    });
}

fn main() {
    let context = sdl2::init().unwrap();
    let video = context.video().unwrap();

    let window = video
        .window("Fractals", 1024, 768)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut events = context.event_pump().unwrap();

    let mut pal = palette::new_color1_mod();

    let mut done = false;
    let mut do_redraw = true;

    let mut fractal = new_fractal(canvas.window().size().0, canvas.window().size().1);
    fractal.state.center.re -= 0.5;

    let mut surface = sdl2::surface::Surface::new(
        fractal.state.width,
        fractal.state.height,
        sdl2::pixels::PixelFormatEnum::RGB888,
    )
    .unwrap();

    while !done {
        canvas.clear();

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    done = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => {
                    done = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => {
                    pal = cycle_palette(pal);
                    println!(
                        "New palette type is {:?} color mode is {:?}",
                        pal.palette_type, pal.color_mode
                    );
                    do_redraw = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Plus),
                    ..
                } => {
                    fractal.state.scale = 0.9 * fractal.state.scale;
                    do_redraw = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Equals),
                    ..
                } => {
                    fractal.state.scale = 0.9 * fractal.state.scale;
                    do_redraw = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Minus),
                    ..
                } => {
                    fractal.state.scale = 1.1 * fractal.state.scale;
                    do_redraw = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    fractal.state.center.re -= 0.1 * fractal.state.scale;
                    do_redraw = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    fractal.state.center.re += 0.1 * fractal.state.scale;
                    do_redraw = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    fractal.state.center.im += 0.1 * fractal.state.scale;
                    do_redraw = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    fractal.state.center.im -= 0.1 * fractal.state.scale;
                    do_redraw = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    if let FractalType::Julia(c) = fractal.state.fractal_type {
                        let c = Complex64::new(c.re - 0.1 * fractal.state.scale, c.im);
                        fractal.state.fractal_type = FractalType::Julia(c);
                        do_redraw = true;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    if let FractalType::Julia(c) = fractal.state.fractal_type {
                        let c = Complex64::new(c.re + 0.1 * fractal.state.scale, c.im);
                        fractal.state.fractal_type = FractalType::Julia(c);
                        do_redraw = true;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    if let FractalType::Julia(c) = fractal.state.fractal_type {
                        let c = Complex64::new(c.re, c.im + 0.1 * fractal.state.scale);
                        fractal.state.fractal_type = FractalType::Julia(c);
                        do_redraw = true;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    if let FractalType::Julia(c) = fractal.state.fractal_type {
                        let c = Complex64::new(c.re, c.im - 0.1 * fractal.state.scale);
                        fractal.state.fractal_type = FractalType::Julia(c);
                        do_redraw = true;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::LeftBracket),
                    ..
                } => {
                    fractal.state.max_iterations += 25;
                    do_redraw = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::RightBracket),
                    ..
                } => {
                    if fractal.state.max_iterations > 200 {
                        fractal.state.max_iterations += 25;
                        do_redraw = true;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::J),
                    ..
                } => {
                    fractal.state.fractal_type = match fractal.state.fractal_type {
                        mandelbrot::FractalType::Mandelbrot => {
                            mandelbrot::FractalType::Julia(fractal.state.center)
                        }
                        mandelbrot::FractalType::Julia(_) => mandelbrot::FractalType::Mandelbrot,
                    };
                    do_redraw = true;
                }
                _ => {}
            }
        }
        if do_redraw {
            mandelbrot::compute_mandelbrot(&mut fractal);
            render_image_to_surface(&fractal, &mut surface, &pal);

            let r1 = sdl2::rect::Rect::new(0, 0, fractal.state.width, fractal.state.height);
            let r2 = sdl2::rect::Rect::new(0, 0, fractal.state.width, fractal.state.height);
            let mut dest = canvas.window().surface(&events).unwrap();
            surface.blit(r1, &mut dest, r2).unwrap();
            do_redraw = false;
            dest.update_window().unwrap();
        }

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
