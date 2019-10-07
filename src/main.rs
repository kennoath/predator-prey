extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;


use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use rand::Rng;
use rand::seq::SliceRandom;
use std::env::Args;

// It works which is cool
// Performance is bad

#[derive(Clone, Copy)]
pub enum Cell {
    Predator(f32),
    Prey,
    Empty
}

fn cell_colour(c: Cell) -> [f32; 4] {
    match c {
        Cell::Predator(f) => [f, 0.0, 0.0, 1.0],
        Cell::Prey => [0.0, 1.0, 0.0, 1.0],
        Cell::Empty => [0.0, 0.0, 0.0, 1.0],
    }
}

// Concerns only nitty gritty app stuff 
pub struct App {
    gl: GlGraphics,
    
    params: ModelParams,
    state:  ModelState,

    frame_period: f64,
    step_period: f64,
    time_since_step: f64,
}

// Concerns immutable aspects of the simulation
#[derive(Clone, Copy, Default)]
pub struct ModelParams {
    predator_reproduce_threshold: f32,
    predator_reproduce_cost: f32,
    predator_live_cost: f32,
    predator_starting_food: f32,
    predator_starting_percent: f32,
    prey_food_value: f32,
    prey_reproduce_chance: f32,
    prey_starting_percent: f32,
    gx: usize,
    gy: usize,
}

// Concerns the mutable state of the simulation
#[derive(Clone, Default)]
pub struct ModelState {
    cells: Vec<Cell>,
    gen: i32,
    numPreds: i32,
    numPrey: i32,
}


impl ModelState {
    fn step(&mut self, params: ModelParams) {
        self.gen += 1;
        let mut rng = rand::thread_rng();
        let mut acc_pred = 0;
        let mut acc_prey = 0;

        for i in 0..params.gx * params.gy {
            let mut c = self.cells[i];
            let dest_index = self.get_random_neighbouring_index(params, i);
            let other_c = self.cells[dest_index];

            // Self update
            match c {
                Cell::Predator(f) => {
                    acc_pred += 1;
                    let mut new_f = f - params.predator_live_cost;
                    if new_f > 0.0 {
                        c = Cell::Predator(new_f);
                    } else {
                        c = Cell::Empty;
                    }
                }
                Cell::Prey => {
                    acc_prey += 1;
                }
            }

            // Other update
            match (c, other_c) {

                // Predator eats prey
                (Cell::Predator(f), Cell::Prey) => {
                    let new_f = f + params.prey_food_value;
                    if new_f > params.predator_reproduce_threshold {
                        self.cells[i] = Cell::Predator(params.predator_starting_food);
                        new_f -= params.predator_reproduce_cost;
                    } else {
                        self.cells[i] = Cell::Empty;
                    }
                    if new_f > 0.0 {
                        self.cells[dest_index] = Cell::Predator(new_f);
                    }
                }

                // Predator moves
                (Cell::Predator(f), Cell::Empty) => {
                    let new_f = f;
                    if new_f > params.predator_reproduce_threshold {
                        self.cells[i] = Cell::Predator(params.predator_starting_food);
                        new_f -= params.predator_reproduce_cost;
                    } else {
                        self.cells[i] = Cell::Empty;
                    }
                    if new_f > 0.0 {
                        self.cells[dest_index] = Cell::Predator(new_f);
                    }
                }

                // Prey moves
                (Cell::Prey, Cell::Empty) => {
                    let r: f32 = rng.gen();
                    self.cells[dest_index] = Cell::Prey;
                    if r < params.prey_reproduce_chance {
                        self.cells[i] = Cell::Prey;
                    } else {
                        self.cells[i] = Cell::Empty;
                    }
                }

                // Otherwise remain stationary
                _ => self.cells[i] = c
            }
        }
        self.numPreds = acc_pred;
        self.numPrey = acc_prey;
    }

    fn disp(&self, params: ModelParams) -> String {
        return format!("Generation {}: ppred: {}, pprey: {}", self.gen, self.numPreds, self.numPrey);
    }

    // we can just look 4wise at the moment
    fn get_random_neighbouring_index(&self, params: ModelParams, index: usize) -> usize {
        let mut candidates = Vec::new();
        if index > params.gx {
            candidates.push(index - params.gx)
        }
        if index < params.gx * (params.gy-1) {
            candidates.push(index + params.gx)
        }
        if index % params.gx > 0 {
            candidates.push(index - 1)
        }
        if index % params.gx < params.gx-1 {
            candidates.push(index + 1)
        }
        *candidates.choose(&mut rand::thread_rng()).unwrap()
    }
}

// There would be a pretty easy way to make prey work on similar rules like if it had a food value and ate grass every time
// could do this with a bunch of tuples / vectors for different species that eat each other lol. maybe just a match function that tells you what square it eats


impl App {
    fn update(&mut self, args: &UpdateArgs) {
        self.time_since_step += args.dt;
        if self.time_since_step > self.step_period {
            self.time_since_step = 0.0;
            self.state.step(self.params);
            println!("{}", self.state.disp(self.params));
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let sx = 1.0 / self.params.gx as f64;
        let sy = 1.0 / self.params.gy as f64;
        
            for i in 0..self.params.gx * self.params.gy {
                let ix = (i % self.params.gx) as f64;
                let iy = (i / self.params.gx) as f64;
                let col = cell_colour(self.state.cells[i as usize]);

                self.gl.draw(args.viewport(), |c, gl| {
                    let t = c.transform.scale(args.window_size[0] as f64, args.window_size[1] as f64);
                    rectangle(col,
                            [ix * sx, iy * sy, sx, sy],
                            t, gl);
            });
        }
    }
}

fn make_app_from_args(gl: GlGraphics, args: Args) -> App {
    let mut a = App {
        gl: gl,
        
        params: ModelParams{..Default::default()},
        state: ModelState{..Default::default()},

        frame_period: 0.0,
        step_period: 0.0,
        time_since_step: 0.0,
    };

    /*
    "--predator_live_cost=0.01"
    this is getting verbose, probably implement default for the parameters struct and then make these available to change it
    params_from_args would be a cleaner more functional way to implement the model
    */

    for arg in args {
        match arg {

        }
    }

    return a;
}

fn make_app(gl: GlGraphics, gx: usize, gy: usize, params: SimParameters, fps: f64, sps: f64) -> App {
    let mut a = App {
        gl: gl,
        gx: gx,
        gy: gy,
        cells: Vec::with_capacity((gx*gy) as usize),
        gen: 0,
        params: params,
        frame_period: 1.0/fps,
        step_period: 1.0/sps,
        time_since_step: 0.0,
    };
    for _i in 0..gx*gy {
        let r: f32 = rand::thread_rng().gen();
        if r < params.predator_starting_percent {
            a.cells.push(Cell::Predator(params.predator_starting_food));
        } else if r < params.predator_starting_percent + params.prey_starting_percent {
            a.cells.push(Cell::Prey);
        } else {
            a.cells.push(Cell::Empty);
        }
    }
    return a
}



fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
            "Lotka Volterra Boi",
            [800, 800]
        )
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let params = SimParameters {
        predator_reproduce_threshold: 0.8,
        predator_reproduce_cost: 0.4,
        predator_live_cost: 0.03,
        predator_starting_food: 0.7,
        predator_starting_percent: 0.1,
        prey_food_value: 0.2,
        prey_reproduce_chance: 0.1,
        prey_starting_percent: 0.2,
    };

    let mut app = make_app(GlGraphics::new(opengl), 60, 60, params, 60.0, 10.0);
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}