
use std::{
    fs::File,
    io::prelude::*
};

pub fn read_fens() -> Vec<String> {

    // open file
    let mut file = File::open("tests/FENs").expect("Could not open file!");

    // read contents
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Could not read contents of file!");

    // split at newlines
    return contents.split('\n').map(|s| s.to_owned()).collect();

}
