use std::fmt;
use std::result;


#[derive(Debug)]
pub enum Error {
    X11(String),
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::X11(ref error) =>
                write!(f, "{}", error),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
