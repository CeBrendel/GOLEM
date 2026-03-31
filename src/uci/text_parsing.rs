
use std::{
    fmt,
    str::{FromStr, SplitWhitespace}
};
use std::iter::Peekable;

pub fn pop_first(s: &str) -> (&str, &str) {
    return match s.split_once(char::is_whitespace) {
        Option::None                     => (s, ""),
        Option::Some((h, t)) => (h, t)
    };
}

pub fn parse_next_block_as<T: FromStr>(blocks: &mut Peekable<SplitWhitespace>) -> T where <T as FromStr>::Err: fmt::Debug {
    match blocks.next() {
        Option::None                      => panic!("No block found!"),
        Option::Some(number_as_str) => number_as_str.parse::<T>().expect("Could not parse to requested type!")
    }
}

pub fn collect_blocks_until_next_keyword_or_end(blocks: &mut Peekable<SplitWhitespace>) -> Vec<String> {

    // make vector for collecting blocks
    let mut collected_blocks = Vec::new();

    // loop through blocks; use peeking to not unnecessarily advance "blocks" iterator
    loop {

        // peek into next block. If there is none or if it is a keyword we are done collecting, otherwise collect
        match blocks.peek() {
            Option::None               => {break;},
            Option::Some(&block) => {
                match block {
                    "searchmoves" | "ponder" | "wtime" | "btime"
                    | "winc" | "binc" | "movestogo" | "depth"
                    | "nodes" | "mate" | "movetime" | "infinite" => {break;},
                    _                                            => {

                        // advance original iterator
                        // unwrapping is ok, because the Option::None case wa shandled while peeking
                        let block = blocks.next().unwrap();

                        // remember block
                        collected_blocks.push(block.to_owned());

                    }
                }
            }
        }
    }

    return collected_blocks;
    
}
