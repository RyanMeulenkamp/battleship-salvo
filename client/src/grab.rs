use std::io;
use std::io::BufRead;
use crate::orientation::Orientation;
use crate::orientation::Orientation::{Horizontal, Vertical};

pub fn grab_string() -> String {
    io::stdin().lock().lines().next().unwrap().unwrap()
}

pub fn grab_number() -> u8 {
    loop {
        if let Ok(number) = grab_string().parse() {
            return number
        }
    }
}

pub fn grab_coordinate(axis: &str) -> u8 {
    loop {
        println!("{} = ", axis);
        let distance = grab_number();
        if distance <= 9 {
            return distance;
        } else {
            println!("{} is out of bounds!", distance);
        }
    }
}

pub fn grap_coordinates() -> (u8, u8) {
    (grab_coordinate("x"), grab_coordinate("y"))
}

pub(crate) fn grab_orientation() -> Orientation {
    loop {
        return match grab_number() {
            0 => Horizontal,
            1 => Vertical,
            other => {
                eprintln!("{} not a valid orientation specifier", other);
                continue
            }
        }
    }
}
