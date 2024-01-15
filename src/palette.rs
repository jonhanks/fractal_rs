pub type PaletteData = Vec<u32>;

#[derive(Copy, Clone, Debug)]
pub enum ColorMode {
    LinearScale,
    Modulus,
}

#[derive(Copy, Clone, Debug)]
pub enum PaletteType {
    BW,
    Color1Mod,
    Color1Lin,
}

pub struct Palette {
    pub palette_type: PaletteType,
    pub color_mode: ColorMode,
    pub palette: PaletteData,
}

pub fn new_bw() -> Palette {
    let mut pd = Vec::new();
    for i in 0u32..255 {
        pd.push((i & 0xff) << 16 | (i & 0xff) << 8 | (i & 0xff));
    }
    Palette {
        palette_type: PaletteType::BW,
        color_mode: ColorMode::LinearScale,
        palette: pd,
    }
}

pub fn new_color1_mod() -> Palette {
    let mut pd = Vec::new();
    pd.append(&mut color_step(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 100));
    pd.append(&mut color_step(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 100));
    pd.append(&mut color_step(0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 100));
    pd.append(&mut color_step(1.0, 1.0, 0.0, 0.0, 0.0, 0.7, 100));
    Palette {
        palette_type: PaletteType::Color1Mod,
        color_mode: ColorMode::Modulus,
        palette: pd,
    }
}

pub fn new_color1_lin() -> Palette {
    let mut p = new_color1_mod();
    p.palette_type = PaletteType::Color1Lin;
    p.color_mode = ColorMode::LinearScale;
    p
}

fn rgb_f64_to_rgb_u32(r: f64, g: f64, b: f64) -> u32 {
    let mut sr = (r * 255.0) as u32;
    let mut sg = (g * 255.0) as u32;
    let mut sb = (b * 255.0) as u32;
    if sr > 255 {
        sr = 255;
    }
    if sg > 255 {
        sg = 255;
    }
    if sb > 255 {
        sb = 255;
    }
    ((sr << 16) | (sg << 8) | (sb)) as u32
}

fn color_step(
    mut cur_r: f64,
    mut cur_g: f64,
    mut cur_b: f64,
    dest_r: f64,
    dest_g: f64,
    dest_b: f64,
    steps: usize,
) -> PaletteData {
    let mut p = Vec::new();
    let steps_f = steps as f64;
    let delta_r = (dest_r - cur_r) / steps_f;
    let delta_g = (dest_g - cur_g) / steps_f;
    let delta_b = (dest_b - cur_b) / steps_f;

    for i in 0..steps {
        p.push(rgb_f64_to_rgb_u32(cur_r, cur_g, cur_b));
        cur_r += delta_r;
        cur_g += delta_g;
        cur_b += delta_b;
    }
    p
}
