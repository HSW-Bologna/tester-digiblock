#[derive(Clone, Default)]
pub struct Model {
    pub count: i32,
}

impl Model {
    pub fn increase(self: Self) -> Self {
        Self {
            count: self.count + 1,
        }
    }

    pub fn decrease(self: Self) -> Self {
        Self {
            count: self.count - 1,
        }
    }
}
