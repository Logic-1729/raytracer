use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval { pub min: f64, pub max: f64,}

impl Interval {
    pub fn new(min: f64, max: f64) -> Self { Interval { min, max }}

    pub fn default() -> Self { Interval::empty()}

    /// Default interval is empty: (min=+infinity, max=-infinity)
    pub fn empty() -> Self { Interval {min: f64::INFINITY,max: f64::NEG_INFINITY,}}

    /// Universe interval: (min=-infinity, max=+infinity)
    pub fn universe() -> Self {Interval {min: f64::NEG_INFINITY,max: f64::INFINITY,}}

    pub fn from_two(a: &Interval, b: &Interval) -> Self {Interval {min: a.min.min(b.min),max: a.max.max(b.max),}}

    pub fn size(&self) -> f64 { self.max - self.min}

    pub fn contains(&self, x: f64) -> bool { self.min <= x && x <= self.max }

    pub fn surrounds(&self, x: f64) -> bool { self.min < x && x < self.max}

    pub fn clamp(&self, x: f64) -> f64 {
        if x < self.min {  self.min  } 
        else if x > self.max { self.max} 
        else { x }
    }

    pub fn expand(&self, delta: f64) -> Self {
        let padding = delta / 2.0;
        Interval { min: self.min - padding, max: self.max + padding,}
    }
}

impl Add<f64> for Interval {
    type Output = Interval;

    fn add(self, displacement: f64) -> Interval {
        Interval {
            min: self.min + displacement,
            max: self.max + displacement,
        }
    }
}

// 支持 f64 + Interval，直接调用 Interval + f64
impl Add<Interval> for f64 {
    type Output = Interval;

    fn add(self, ival: Interval) -> Interval {
        ival + self
    }
}