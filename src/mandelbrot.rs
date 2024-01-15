use rayon::prelude::*;
pub use num_complex::{Complex64, ComplexFloat};

pub enum FractalType {
    Mandelbrot,
    Julia(Complex64),
}
pub struct State {
    pub width: u32,
    pub height: u32,
    pub max_iterations: u32,
    pub scale: f64,
    pub center: Complex64,
    pub fractal_type: FractalType,
}

impl State {
    pub fn aspect(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    pub fn increments(&self) -> (f64, f64) {
        ( self.scale / self.width as f64,
          (self.scale / self.aspect()) / self.height as f64 )
    }
}

pub type DataRow = Vec<FractalSample>;

pub struct Data {
    pub state: State,
    pub fractal_data: Vec<Vec<FractalSample>>,
}

impl Data {
    pub fn new(state: State) -> Self {
        if state.width == 0 || state.height == 0 {
            panic!("Bad dimensions in fractal state");
        }
        let mut data:Vec<Vec<FractalSample>> = Vec::new();
        data.reserve(state.height as usize);
        for _y in 0..state.height {
            let mut row: Vec<FractalSample> = Vec::new();
            row.resize(state.width as usize, FractalSample{z: Complex64::new(0.,0.), escape: 0});
            data.push(row);
        }
        Self{
            state,
            fractal_data: data,
        }
    }

    pub fn resize(&mut self) {
        if self.state.width == 0 || self.state.height == 0 {
            panic!("Bad dimensions in fractal state");
        }
        let mut data:Vec<Vec<FractalSample>> = Vec::new();
        data.reserve(self.state.height as usize);
        for _y in 0..self.state.height {
            let mut row: Vec<FractalSample> = Vec::new();
            row.resize(self.state.width as usize, FractalSample{z: Complex64::new(0.,0.), escape: 0});
            data.push(row);
        }
        self.fractal_data = data;
    }
}

#[derive(Copy,Clone,Debug,Default)]
pub struct FractalSample {
    pub z: Complex64,
    pub escape: u32,
}

fn mandelbrot_f(c: Complex64, z0: Complex64, cur_iterations: u32,  max_iterations: u32) -> FractalSample {
    let mut z = z0;

    let mut i = cur_iterations;
    while i < max_iterations && z.abs() < 2.0 {
        z = z*z + c;
        i += 1;
    }
    FractalSample{
        z,
        escape: i,
    }
}

fn mandelbrot_row(mut x_cur: f64, y_cur: f64, x_incr: f64, state: &State, data_row: &mut Vec<FractalSample>) {
    for x in 0..state.width {
        let z = Complex64::new(x_cur, y_cur);
        let c = z;

        data_row[x as usize] = mandelbrot_f(c, z, 0, state.max_iterations);
        x_cur += x_incr;
    }
}

fn julia_row(mut x_cur: f64, y_cur: f64, x_incr: f64, c: Complex64, state: &State, data_row: &mut Vec<FractalSample>) {
    for x in 0..state.width {
        let z = Complex64::new(x_cur, y_cur);

        data_row[x as usize] = mandelbrot_f(c, z, 0, state.max_iterations);
        x_cur += x_incr;
    }
}

pub fn compute_mandelbrot(fd: &mut Data) {
    if fd.fractal_data.len() != fd.state.height as usize{
        panic!("sample array is the wrong size");
    }
    let aspect = fd.state.aspect();
    let (x_incr, y_incr) = fd.state.increments();
    let mut y_cur = fd.state.center.im + (fd.state.scale/aspect)/2.0;
    let x_cur = fd.state.center.re - fd.state.scale/2.0;


    match fd.state.fractal_type {
        FractalType::Mandelbrot => {
            fd.fractal_data.par_iter_mut().enumerate().for_each(|entry| {
                mandelbrot_row(x_cur, y_cur - ((entry.0 as f64) * y_incr), x_incr, &fd.state, entry.1);
            });
        },
        FractalType::Julia(c) => {
            fd.fractal_data.par_iter_mut().enumerate().for_each(|entry| {
                julia_row(x_cur, y_cur - ((entry.0 as f64) * y_incr), x_incr, c, &fd.state, entry.1);
            });
        }
    }
}