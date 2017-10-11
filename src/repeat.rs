pub fn repeat(max: i32) -> Repeat {
    Repeat { max: max, iteration: 0 }
}

pub struct Repeat {
    max: i32,
    iteration: i32,
}

impl Iterator for Repeat {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        if self.iteration >= self.max {
            Option::None
        } else {
            let val = Option::Some(self.iteration);
            self.iteration += 1;
            val
        }
    }
}
