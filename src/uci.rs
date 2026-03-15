

use std::process;
use crate::board::traits::{Move, Board};

use std::str::SplitWhitespace;
use std::iter::Peekable;


#[derive(Debug, Default)]
struct SearchInstruction {
    // TODO: ponder, nodes, mate
    searchmoves: Option<Vec<String>>,
    wtime_in_ms: Option<usize>,
    btime_in_ms: Option<usize>,
    winc_in_ms: Option<usize>,
    binc_in_ms: Option<usize>,
    movestogo: Option<usize>,
    depth: Option<usize>,
    movetime_in_ms: Option<usize>,
    infinite: bool
}

impl SearchInstruction {
    pub fn visualize(&self) {

        // for easier optional printing
        macro_rules! maybe_println {
            ($description: expr, $option: expr) => {
                match $option {
                    Option::None     => {},
                    Option::Some(t) => println!($description, t)
                }
            };
        }

        println!("\nSearchIncstructions:");
        maybe_println!("searchmoves: {:?}", &self.searchmoves);
        maybe_println!("   infinite: {}", Option::Some(self.infinite));
        maybe_println!("      depth: {}", self.depth);
        maybe_println!("   movetime: {}", self.movetime_in_ms);
        maybe_println!("      wtime: {}", self.wtime_in_ms);
        maybe_println!("      btime: {}", self.btime_in_ms);
        maybe_println!("       winc: {}", self.winc_in_ms);
        maybe_println!("       binc: {}", self.binc_in_ms);
        maybe_println!("  movestogo: {}", self.movestogo);
        println!();
        
    }
}



fn pop_first(s: &str) -> (&str, &str) {
    return match s.split_once(char::is_whitespace) {
        Option::None                     => (s, ""),
        Option::Some((h, t)) => (h, t)
    };
}

fn handle_uci() {
    // TODO: Show options!
    println!("id name גֹלֶם (GOLEM)");
    println!("id author Cedric Brendel");
    println!("uciok");
    println!("");
}

fn handle_isready() {
    println!("readyok");
    println!("");
}

fn handle_quit() {
    // TODO: This is not what we want! E. g. print bestmove.
    process::exit(0)
}

fn handle_position<M: Move, B: Board<M>>(s: &str, board: &mut B) {

    // get keyword
    let (startpos_or_fen_keyword, tail) = pop_first(s);

    // split at "moves" (if it occurs) to obtain infromation for base position and moves
    let (maybe_fen_with_fen_keyword, maybe_moves) = match tail.find("moves") {
        Option::None      => (tail, Option::None),
        Option::Some(idx) => (&tail[..idx], Option::Some(&tail[idx+5..]))
    };

    // parse startpos and fen keyword
    match startpos_or_fen_keyword {
        "startpos" => board.put_into_startpos(),
        "fen"      => {
            let (_, fen) = pop_first(maybe_fen_with_fen_keyword);
            board.put_into_fen(fen);
        },
        _          => panic!("Received invalid \"position\" command!")
    }

    // push moves onto board (if there are any)
    match maybe_moves {
        Option::None        => {},  // do nothing
        Option::Some(moves) => {

            // split at whitespaces to obtain moves, and make them
            for algebraic_move in moves.split_whitespace() {
                let r#move = M::from_algebraic(algebraic_move);
                board.make(r#move);
            }

        }
    }

    // show board after handling of command
    board.visualize();

}

fn parse_next_block_as_usize(blocks: &mut Peekable<SplitWhitespace>) -> usize {
    match blocks.next() {
        Option::None                => panic!("No block found!"),
        Option::Some(number_as_str) => number_as_str.parse::<usize>().expect("Could not parse to usize!")
    }
}

fn collect_blocks_until_next_keyword_or_end(blocks: &mut Peekable<SplitWhitespace>) -> Vec<String> {

    // make vector for collecting blocks
    let mut collected_blocks = Vec::new();

    // loop through blocks; use peeking to not unnecessarily advance "blocks" iterator
    loop {

        // peek into next block. If there is none or if it is a keyword we are done collecting, otherwise collect
        match blocks.peek() {
            Option::None        => {break;},
            Option::Some(&block) => {
                match block {
                    "searchmoves" | "ponder" | "wtime" | "btime"
                    | "winc" | "binc" | "movestogo" | "depth"
                    | "nodes" | "mate" | "movetime" | "infinite"    => {break;},
                    _                                               => {

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

fn handle_go<M: Move, B: Board<M>>(s: &str, board: &mut B) -> SearchInstruction {

    // make empty SearchInstruction
    let mut search_instructions = SearchInstruction::default();

    // split command into blocks and handle them separately
    // make peekable so that we can determine scope of searchmoves command without advancing iterator
    let mut blocks = s.split_whitespace().peekable();
    loop {

        // get next block and parse it
        match blocks.next() {
            Option::None          => {break;}
            Option::Some(keyword) => {

                match keyword {
                    "searchmoves" => {
                        let collected_blocks = collect_blocks_until_next_keyword_or_end(&mut blocks);
                        search_instructions.searchmoves = Option::Some(collected_blocks);
                    },
                    "ponder"      => todo!("Engine currently does not support pondering!"),
                    "wtime"       => {
                        let wtime_in_ms = parse_next_block_as_usize(&mut blocks);
                        search_instructions.wtime_in_ms = Option::Some(wtime_in_ms);
                    },
                    "btime"       => {
                        let btime_in_ms = parse_next_block_as_usize(&mut blocks);
                        search_instructions.btime_in_ms = Option::Some(btime_in_ms);
                    },
                    "winc"        => {
                        let winc_in_ms = parse_next_block_as_usize(&mut blocks);
                        search_instructions.winc_in_ms = Option::Some(winc_in_ms);
                    },
                    "binc"        => {
                        let binc_in_ms = parse_next_block_as_usize(&mut blocks);
                        search_instructions.binc_in_ms = Option::Some(binc_in_ms);
                    },
                    "movestogo"   => {
                        let movestogo = parse_next_block_as_usize(&mut blocks);
                        search_instructions.movestogo = Option::Some(movestogo);
                    },
                    "depth"       => {
                        let depth = parse_next_block_as_usize(&mut blocks);
                        search_instructions.depth = Option::Some(depth);
                    },
                    "nodes"       => todo!("Engine currently does not support only searching a fixed number of nodes."),
                    "mate"        => todo!("Engine currently does not support mate search."),
                    "movetime"    => {
                        let movetime_in_ms = parse_next_block_as_usize(&mut blocks);
                        search_instructions.movetime_in_ms = Option::Some(movetime_in_ms);
                    },
                    "infinite"    => {search_instructions.infinite = true;},
                    _             => {
                        println!("Keyword: {}, Remaining blocks: {:?}", keyword, blocks);
                        panic!("Received invalid \"go\" command!");
                    }
                }
            }
        }
    };

    search_instructions.visualize();

    return search_instructions;

}

pub fn parse_uci_command<M: Move, B: Board<M>>(command: &str, board: &mut B) {

    // remove linebreaks from command
    let command = command.trim();

    // split at first whitespace to obtain command (uci/go/...) and remaining tail
    let (keyword, tail) = pop_first(command);

    match keyword {
        "uci"           => handle_uci(),
        "isready"       => handle_isready(),
        "position"      => handle_position(tail, board),
        "go"            => {handle_go(tail, board);},
        "stop"          => todo!(),
        "ucinewgame"    => todo!("Engine does not currently support the \"ucinewgame\" command!"),
        "ponderhit"     => todo!("Engine does not currently support pondering!"),
        "setoption"     => todo!(),
        "quit"          => handle_quit(),
        _               => panic!("Unrecognized command!")
    }
}