mod mandelbrot;
mod palette;


use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

use crate::mandelbrot::{compute_mandelbrot, FractalType, State};
use num_complex::{Complex64};

use eframe::{App, Frame};
use eframe::egui;
use eframe::egui::{ColorImage, Context, Sense};
use eframe::egui::Color32;
use eframe::epaint::TextureHandle;

const WIDTH:u32 = 1024;
const HEIGHT:u32 = 768;

fn cycle_palette(p: palette::Palette) -> palette::Palette {
    return match p.palette_type {
        palette::PaletteType::BW => palette::new_color1_lin(),
        palette::PaletteType::Color1Lin => palette::new_color1_mod(),
        palette::PaletteType::Color1Mod => palette::new_color2_lin(),
        palette::PaletteType::Color2Lin => palette::new_color2_mod(),
        palette::PaletteType::Color2Mod => palette::new_bw(),
    };
}

fn new_fractal(w: u32, h: u32) -> mandelbrot::Data {
    let state = State {
        width: w,
        height: h,
        max_iterations: 500,
        scale: 2.0,
        center: Complex64::new(0., 0.),
        fractal_type: FractalType::Mandelbrot,
    };
    mandelbrot::Data::new(state)
}


struct FractalImage {
    state: mandelbrot::State,
    palette: palette::PaletteType,
    texture: TextureHandle,
}

fn render_image_linear(fractal: &mandelbrot::Data, buffer: &mut [Color32], pal: &palette::PaletteData) {
    let mut offset = 0;
    let scale_factor = (pal.len() - 1) as f64 / fractal.state.max_iterations as f64;
    fractal.fractal_data.iter().for_each(|row| {
        row.iter().for_each(|entry| {
            if entry.escape >= fractal.state.max_iterations {
                buffer[offset] = Color32::BLACK;
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
    buffer: &mut [Color32],
    pal: &palette::PaletteData,
) {
    let mut offset = 0;
    let pal_len = pal.len();
    fractal.fractal_data.iter().for_each(|row| {
        row.iter().for_each(|entry| {
            if entry.escape >= fractal.state.max_iterations {
                buffer[offset] = Color32::BLACK;
            } else {
                buffer[offset] = pal[entry.escape as usize % pal_len];
            }
            offset += 1;
        });
    });
}

fn render_image_to_surface(
    fractal: &mandelbrot::Data,
    image: &mut ColorImage,
    pal: &palette::Palette,
) {
    match pal.color_mode {
        palette::ColorMode::LinearScale => render_image_linear(fractal, image.pixels.as_mut_slice(), &pal.palette),
        palette::ColorMode::Modulus => render_image_modulus(fractal, image.pixels.as_mut_slice(), &pal.palette),
    };
}

struct StateAndPalette {
    state: mandelbrot::State,
    pal: palette::PaletteType,
}

impl StateAndPalette {
    pub fn new(state: mandelbrot::State, pal: palette::PaletteType) -> Self {
        Self{
            state,
            pal,
        }
    }
}

struct FractalViewer {
    current_state: mandelbrot::State,
    current_texture: Option<TextureHandle>,
    current_palette: palette::PaletteType,
    ui_recv: Receiver<FractalImage>,
    ui_send: Sender<Option<StateAndPalette>>
}

impl FractalViewer {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut state = mandelbrot::State::new(WIDTH, HEIGHT);
        state.center.re -= 0.5;

        let (background_send, ui_recv) = channel::<FractalImage>();
        let (ui_send, background_recv) = channel::<Option<StateAndPalette>>();
        let background_cc = cc.egui_ctx.clone();
        let thread_handle = thread::spawn(move || {
            background_thread(background_cc, background_recv, background_send);
        });

        ui_send.send(Some(StateAndPalette::new(state.clone(), palette::PaletteType::Color1Lin))).unwrap();
        FractalViewer{
            current_state: state,
            current_texture: None,
            current_palette: palette::PaletteType::Color1Lin,
            ui_recv,
            ui_send,
        }
    }
}

impl App for FractalViewer {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let mut new_palette = self.current_palette.clone();
        if let Ok(new_image) = self.ui_recv.try_recv() {
            self.current_state = new_image.state;
            self.current_palette = new_image.palette.clone();
            new_palette = self.current_palette.clone();
            self.current_texture = Some(new_image.texture);
        }

        let mut panel = egui::CentralPanel::default();
        let mut new_state = self.current_state.clone();

        let mut send_new_state = false;
        panel.show(ctx, |ui| {
            if let Some(texture) = self.current_texture.as_ref() {
                let img = egui::Image::new((texture.id(), texture.size_vec2())).sense(Sense::click());
                let mut img_resp = ui.add(img);
                if img_resp.clicked() {

                    let pos = img_resp.hover_pos().unwrap();
                    println!("clicked at {:?} rect is {:?}", &pos, &img_resp.rect);

                    let x = (pos.x - img_resp.rect.left()) as i32;
                    let y = (pos.y - img_resp.rect.top()) as i32;
                    new_state.center = new_state.pixel_to_mandelbrot_coord(x, y);
                    send_new_state = true;
                }

            }
        });
        egui::Window::new("Controls")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("+").clicked() {
                            new_state.scale *= 0.9;
                            send_new_state = true;
                        }
                        if ui.button("-").clicked() {
                            new_state.scale *= 1.1;
                            send_new_state = true;
                        }
                        ui.label("Zoom")
                    });
                    ui.horizontal((|ui| {
                        egui::ComboBox::from_label("Fractal Type")
                            .selected_text(format!("{}", self.current_state.fractal_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut new_state.fractal_type, FractalType::Mandelbrot, format!("{}", FractalType::Mandelbrot));
                                let julia = match new_state.fractal_type {
                                    FractalType::Mandelbrot => FractalType::Julia(self.current_state.center),
                                    FractalType::Julia(c) => FractalType::Julia(c),
                                };
                                ui.selectable_value(&mut new_state.fractal_type, julia.clone(), format!("{}", julia));
                            })
                    }));
                    ui.horizontal((|ui| {
                        egui::ComboBox::from_label("Palette")
                            .selected_text(format!("{:?}", new_palette))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut new_palette, palette::PaletteType::BW, format!("{:?}", palette::PaletteType::BW));
                                ui.selectable_value(&mut new_palette, palette::PaletteType::Color1Lin, format!("{:?}", palette::PaletteType::Color1Lin));
                                ui.selectable_value(&mut new_palette, palette::PaletteType::Color2Lin, format!("{:?}", palette::PaletteType::Color2Lin));
                                ui.selectable_value(&mut new_palette, palette::PaletteType::Color1Mod, format!("{:?}", palette::PaletteType::Color2Mod));
                                ui.selectable_value(&mut new_palette, palette::PaletteType::Color2Mod, format!("{:?}", palette::PaletteType::Color2Mod));
                            })
                    }));
                    ui.horizontal((|ui| {
                        if ui.button("+").clicked() {
                            new_state.max_iterations += 50;
                            send_new_state = true;
                        }
                        if ui.button("-").clicked() {
                            new_state.max_iterations -= 50;
                            send_new_state = true;
                        }
                        ui.label("Detail");
                    }));
                })
            });
        if new_palette != self.current_palette {
            send_new_state = true;
        }
        if new_state.fractal_type != self.current_state.fractal_type {
            send_new_state = true;
            println!("cur fractal: {0}, new fractal: {1}", self.current_state.fractal_type, new_state.fractal_type);
        }
        if send_new_state {
            self.ui_send.send(Some(StateAndPalette::new(new_state, new_palette))).unwrap();
        }
    }
}


fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([WIDTH as f32, HEIGHT as f32]).with_resizable(false),
        ..Default::default()
    };

    eframe::run_native("Fractal Viewer", options, Box::new(|cc| Box::new(FractalViewer::new(cc))))
}

fn background_thread(ctx: egui::Context, from_ui: Receiver<Option<StateAndPalette>>, out: Sender<FractalImage>) {
    println!("background thread started");
    loop {
        let val = from_ui.recv();
        let (state, pal_type) = match val {
            Ok(val) => match val {
                Some(new_state) => (new_state.state, new_state.pal),
                None => return,
            },
            Err(_) => return,
        };
        println!("Got state from ui");
        let mut image = ColorImage::new([state.width as usize, state.height as usize], Color32::BLACK);
        let pal = pal_type.to_palette();
        let mut fractal = mandelbrot::Data::new(state);

        let start = std::time::Instant::now();
        compute_mandelbrot(&mut fractal);
        let calc_dur = start.elapsed();
        let start = std::time::Instant::now();
        render_image_to_surface(&mut fractal, &mut image, &pal);
        let render_dur = start.elapsed();
        let start = std::time::Instant::now();
        let txt = ctx.load_texture("current", image, Default::default());
        let load_dur = start.elapsed();
        println!("calc: {:?}, render: {:?}, load: {:?}", calc_dur, render_dur, load_dur);
        out.send(FractalImage {
            state: fractal.state,
            palette: pal_type,
            texture: txt,
        }).unwrap();
        ctx.request_repaint();
    }
}