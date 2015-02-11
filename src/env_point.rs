//!
//!  env_point.rs
//!
//!  Created by Mitchell Nordine at 06:27PM on December 19, 2014.
//!
//!

use envelope::Point as EnvPoint;

/// A point for interpolating the Amplitude and Frequency envelopes.
#[derive(Debug, Copy, Clone, RustcDecodable, RustcEncodable)]
pub struct Point {
    time: f64,
    value: f64,
    curve: f64,
}

impl EnvPoint for Point {
    type F = f64;
    fn x(&self) -> f64 { self.time }
    fn y(&self) -> f64 { self.value }
    fn curve(&self) -> f64 { self.curve }
    fn new(x: f64, y: f64, curve: f64) -> Point {
        Point { time: x, value: y, curve: curve }
    }
}

