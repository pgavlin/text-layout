pub trait Num {
    fn abs(self) -> Self;
    fn pow(self, y: Self) -> Self;
}

#[cfg(feature = "std")]
impl Num for f32 {
    fn abs(self) -> f32 {
        self.abs()
    }

    fn pow(self, y: f32) -> f32 {
        self.powf(y)
    }
}

#[cfg(all(not(feature = "std"), feature = "libm"))]
impl Num for f32 {
    fn abs(self) -> f32 {
        libm::fabsf(self)
    }

    fn pow(self, y: f32) -> f32 {
        libm::powf(self, y)
    }
}

