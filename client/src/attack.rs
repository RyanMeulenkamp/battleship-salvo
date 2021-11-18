use crate::point::Point;

pub enum Attack {
    Spent(Box<str>, Point),
    Todo,
}
