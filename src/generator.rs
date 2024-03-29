use image::codecs::bmp::BmpEncoder;
use image::ColorType;
use rand::prelude::*;
use std::fmt;

use crate::color::Color;

pub struct Progress {
    pub current: u32,
    pub total: u32,
}

impl Progress {
    pub fn percentage(&self) -> u32 {
        self.current * 100 / self.total
    }
}

impl fmt::Display for Progress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}%", self.percentage())
    }
}

pub struct Generator<R> {
    size: [u32; 2],
    center: [u32; 2],
    rng: R,

    current_ring: i32,
    ring_count: i32,
    data: Vec<Option<Color>>,
}

impl<R: Rng> Generator<R> {
    pub fn new(size: [u32; 2], center: [u32; 2], rng: R) -> Self {
        let cx = center[0] as i32;
        let cy = center[1] as i32;
        let sx = size[0] as i32;
        let sy = size[1] as i32;

        // center
        let ring_count = *[cx, cy, sx - cx, sy - cy].iter().max().unwrap();

        let mut gen = Generator {
            size,
            center,
            rng,
            current_ring: 1,
            ring_count,
            data: vec![None; (size[0] * size[1]) as usize],
        };

        // place first pixel
        // TODO: make configurable
        {
            let r = gen.rng.gen::<f32>();
            let g = gen.rng.gen::<f32>();
            let b = gen.rng.gen::<f32>();
            let make_light = 1.0 / (0.299 * r * r + 0.587 * g * g + 0.114 * b * b).sqrt();
            let p = gen.at_mut(cx, cy).expect("Center point is out of range!");
            *p = Some(Color {
                r: r * make_light,
                g: g * make_light,
                b: b * make_light,
            });
        }

        gen
    }

    pub fn next(&mut self) -> Option<Progress> {
        if self.current_ring == self.ring_count {
            return None;
        }

        let r = self.current_ring;
        let cx = self.center[0] as i32;
        let cy = self.center[1] as i32;
        let vs = self.around(cx, cy, r);
        for (x, y) in vs {
            let mut c: Option<Color> = None;
            for (x, y) in self.around(x, y, 1) {
                if let Some(Some(px)) = self.at(x, y) {
                    c = Some(px);
                    break;
                }
            }

            let c = match c {
                Some(x) => x,
                None => continue,
            };

            let c = c.mutate(&mut self.rng);

            let px = match self.at_mut(x, y) {
                Some(x) => x,
                None => continue,
            };

            *px = Some(c);
        }
        let p = Progress {
            current: self.current_ring as u32,
            total: self.ring_count as u32,
        };
        self.current_ring += 1;
        Some(p)
    }

    pub fn finish(&mut self) {
        while let Some(_) = self.next() {}
    }

    pub fn save(&self, writer: &mut impl std::io::Write) {
        let mut data = Vec::with_capacity(self.data.len() * 3);
        for c in self.data.iter() {
            fn to_u8(x: f32) -> u8 {
                if x < 0.0 {
                    return 0;
                }
                if x > 1.0 {
                    return 255;
                }
                (x * 255.0) as u8
            }

            let c_default = Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            };
            let c = match c {
                Some(x) => &x,
                None => &c_default,
            };
            data.push(to_u8(c.r));
            data.push(to_u8(c.g));
            data.push(to_u8(c.b));
        }

        let mut enc = BmpEncoder::new(writer);
        enc.encode(&data, self.size[0], self.size[1], ColorType::Rgb8)
            .unwrap();
    }

    fn check(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.size[0] as i32 && y < self.size[1] as i32
    }

    fn around(&mut self, i: i32, j: i32, r: i32) -> Vec<(i32, i32)> {
        let mut xs = Vec::new();

        let put = |xs: &mut Vec<(i32, i32)>, x, y| {
            if self.check(x, y) {
                xs.push((x, y));
            }
        };

        // TODO: what about cirlces instad of squares?
        for o in -r..r {
            put(&mut xs, i + o, j + r);
            put(&mut xs, i - o, j - r);
            put(&mut xs, i - r, j + o);
            put(&mut xs, i + r, j - o);
        }

        xs.shuffle(&mut self.rng);
        xs
    }

    fn at_mut(&mut self, x: i32, y: i32) -> Option<&mut Option<Color>> {
        if !self.check(x, y) {
            None
        } else {
            let i = (y as u32 * self.size[0] + x as u32) as usize;
            unsafe { Some(self.data.get_unchecked_mut(i)) }
        }
    }

    fn at(&self, x: i32, y: i32) -> Option<Option<Color>> {
        if !self.check(x, y) {
            None
        } else {
            let i = (y as u32 * self.size[0] + x as u32) as usize;
            unsafe { Some(self.data.get_unchecked(i).clone()) }
        }
    }
}
