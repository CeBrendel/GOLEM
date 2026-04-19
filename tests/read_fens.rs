
use std::{
    fs::File,
    io::prelude::*
};

pub fn read_fens() -> Vec<String> {
    
    // open file
    let mut file = File::open("tests/fens/FENs").expect("Could not open file!");
    
    // read contents
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Could not read contents of file!");
    
    // split at newlines
    return contents.split('\n').map(|s| s.to_owned()).collect();
    
}

#[allow(dead_code)]
pub fn read_mates(file_name: &str) -> Vec<(String, String)> {

    // open file
    let mut file = File::open(file_name).expect("Could not open file!");

    // read contents
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Could not read contents of file!");
    
    // a place in which to write all positions to test
    let mut fens_and_moves: Vec<(String, String)> = Vec::new();

    // remove some of the lines
    let mut relevant_lines = contents
        .split("\n")
        .filter(|&block| block.len() > 10);

    // read FENs and best moves (take three lines at a time)
    loop {
        let maybe_name = relevant_lines.next();
        let maybe_fen = relevant_lines.next();
        let maybe_moves = relevant_lines.next();

        let _name = match maybe_name {
            Option::None => break,
            Option::Some(name) => name
        };
        let fen = match maybe_fen {
            Option::None => break,
            Option::Some(fen) => fen
        };
        let moves = match maybe_moves {
            Option::None => break,
            Option::Some(moves) => moves
        };

        fens_and_moves.push((String::from(fen), String::from(moves)));
    }

    return fens_and_moves;

}
