mod color;
mod generator;

use std::fs::File;
use std::io::stdout;
use std::io::BufWriter;

pub use crate::generator::*;

use rand::prelude::*;
use std::path::Path;
use std::path::PathBuf;

use structopt::StructOpt;

fn parse_tuple(s: &str) -> Result<(u32, u32), String> {
    let split: Vec<_> = s.split(|c| c == ',' || c == 'x').collect();

    if split.len() != 2 {
        return Err(String::from(
            "Two arguments are needed seperated by a comma in the form '12,34' or 12x34",
        ));
    }

    let s0 = match split[0].parse() {
        Ok(i) => i,
        Err(_) => return Err(format!("Failed to parse item in tuple: {:?}", split[0])),
    };

    let s1 = match split[1].parse() {
        Ok(i) => i,
        Err(_) => return Err(format!("Failed to parse item in tuple: {:?}", split[1])),
    };

    Ok((s0, s1))
}

#[derive(StructOpt, Debug)]
pub struct Config {
    #[structopt(long)]
    pub seed: Option<u64>,

    #[structopt(short, long, parse(try_from_str = parse_tuple))]
    pub center: Option<(u32, u32)>,

    #[structopt(short, long, parse(try_from_str = parse_tuple))]
    pub size: Option<(u32, u32)>,

    #[structopt(short, long)]
    pub output: Option<PathBuf>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            seed: None,
            center: None,
            size: None,
            output: None,
        }
    }
}

// TODO: make platform independent
fn x11_resolution() -> (u32, u32) {
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();
    (
        screen.width_in_pixels() as u32,
        screen.height_in_pixels() as u32,
    )
}

pub fn run(cfg: &Config) -> Result<(), String> {
    let mut rnd = match cfg.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_rng(thread_rng()).unwrap(),
    };

    let (w, h) = match cfg.size {
        Some(r) => r,
        None => x11_resolution(),
    };

    let (cx, cy) = match cfg.center {
        Some(r) => r,
        None => {
            let cx = rnd.gen_range(0, w);
            let cy = rnd.gen_range(0, h);
            (cx, cy)
        }
    };

    eprintln!("resolution: {}x{}", w, h);
    eprintln!("Creating image");
    let mut gen = Generator::new([w, h], [cx, cy], rnd);
    eprintln!("generating...");
    gen.generate()?;

    match &cfg.output {
        Some(path) => {
            if path.as_path() == Path::new("-") {
                let mut w = stdout();
                gen.save(&mut w);
            } else {
                eprintln!("Saving to {:?}...", path.display());
                let file = File::create(path).unwrap();
                let mut w = BufWriter::new(file);
                gen.save(&mut w);
            }
        }
        None => {
            eprintln!("No output path set");
        }
    };

    Ok(())
}
