use envelope_lib;
pub use envelope_lib::Envelope as Trait;
use std::iter::FromIterator;


/// An alias to the type of point to be used for amp and freq interpolation.
pub type Point = envelope_lib::BezierPoint<f64, f64>;

/// An alias for the envelope to be used used for amp and freq interpolation.
#[derive(Clone, Debug, PartialEq)]
pub struct Envelope {
    pub points: Vec<Point>,
}

impl FromIterator<Point> for Envelope {
    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item=Point> {
        Envelope { points: iter.into_iter().collect(), }
    }
}

impl ::std::convert::From<Vec<Point>> for Envelope {
    fn from(points: Vec<Point>) -> Self {
        Envelope { points: points }
    }
}

impl<'a> Trait<'a> for Envelope {
    type X = f64;
    type Y = f64;
    type Point = Point;
    type Points = ::std::slice::Iter<'a, Point>;
    #[inline]
    fn points(&'a self) -> Self::Points { self.points.iter() }
}
