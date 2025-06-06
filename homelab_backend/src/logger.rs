use std::fmt::Display;

pub fn log_msg(msg: impl Display) {
    println!("{}", msg);
}
