
pub mod channeling;

use crate::board::traits::{Move, Board};
use crate::search::SearchInfo;
use crate::uci::channeling::{channel, Sender, Receiver};

use std::str::SplitWhitespace;
use std::iter::Peekable;
use std::{thread, thread::JoinHandle};
use std::sync::{Arc, Mutex};


#[derive(Default, Clone)]
pub struct SearchInstruction {
    // TODO: ponder, nodes, mate
    pub searchmoves: Option<Vec<String>>,
    pub wtime_in_ms: Option<usize>,
    pub btime_in_ms: Option<usize>,
    pub winc_in_ms: Option<usize>,
    pub binc_in_ms: Option<usize>,
    pub movestogo: Option<usize>,
    pub depth: Option<usize>,
    pub movetime_in_ms: Option<usize>,
    pub infinite: bool
}

type Search<M, B> = fn(&mut B, SearchInstruction, &Receiver<bool>, &Sender<SearchInfo<M>>) -> ();

impl std::fmt::Debug for SearchInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        // for easier optional printing
        macro_rules! maybe_write {
            ($description: expr, $option: expr) => {
                match $option {
                    Option::None     => {},
                    Option::Some(t) => writeln!(f, $description, t)?
                }
            };
        }

        println!("\nSearchInstructions:");
        maybe_write!("searchmoves: {:?}", &self.searchmoves);
        maybe_write!("   infinite: {}", Option::Some(self.infinite));
        maybe_write!("      depth: {}", self.depth);
        maybe_write!("   movetime: {}", self.movetime_in_ms);
        maybe_write!("      wtime: {}", self.wtime_in_ms);
        maybe_write!("      btime: {}", self.btime_in_ms);
        maybe_write!("       winc: {}", self.winc_in_ms);
        maybe_write!("       binc: {}", self.binc_in_ms);
        maybe_write!("  movestogo: {}", self.movestogo);
        println!();

        return Ok(());
        
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
    std::process::exit(0)
}

fn handle_position<M: Move, B: Board<M>>(s: &str, board: Arc<Mutex<B>>) {

    // get keyword
    let (startpos_or_fen_keyword, tail) = pop_first(s);

    // split at "moves" (if it occurs) to obtain infromation for base position and moves
    let (maybe_fen_with_fen_keyword, maybe_moves) = match tail.find("moves") {
        Option::None      => (tail, Option::None),
        Option::Some(idx) => (&tail[..idx], Option::Some(&tail[idx+5..]))
    };

    // acquire lock of board
    let mut board = board.lock().expect("Acquiring lock failed. Mutex is probably poisoned!");

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

fn handle_go(s: &str, search_instruction_tx: &Sender<SearchInstruction>) {

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

    println!("{:?}", search_instructions);

    // send search instruction to search thread
    search_instruction_tx.send(search_instructions);

}

fn handle_stop<M: Move>(stop_tx: &Sender<bool>, search_info_rx: &Receiver<SearchInfo<M>>) {

    // send stop signal to search thread
    stop_tx.send(true);

    // emit bestmove
    let search_info = match search_info_rx.recv() {
        Option::None                => panic!("No search info was supplied by the search thread!"),
        Option::Some(search_info)   => search_info
    };
    let bestmove = search_info.r#move.expect("Search did not return a move!");
    let bestmove_str = bestmove.as_str();
    println!("bestmove {}\n", bestmove_str);

}

fn parse_and_handle_uci_command<M: Move, B: Board<M>>(
    command: &str,
    board: Arc<Mutex<B>>,
    search_instruction_tx: &Sender<SearchInstruction>,
    stop_tx: &Sender<bool>,
    search_info_rx: &Receiver<SearchInfo<M>>
) {

    // remove linebreaks from command
    let command = command.trim();

    // split at first whitespace to obtain command (uci/go/...) and remaining tail
    let (keyword, tail) = pop_first(command);

    match keyword {
        "uci"           => handle_uci(),
        "isready"       => handle_isready(),
        "position"      => handle_position(tail, board),
        "go"            => handle_go(tail, search_instruction_tx),
        "stop"          => handle_stop(stop_tx, search_info_rx),
        "ucinewgame"    => todo!("Engine does not currently support the \"ucinewgame\" command!"),
        "ponderhit"     => todo!("Engine does not currently support pondering!"),
        "setoption"     => todo!(),
        "quit"          => handle_quit(),
        _               => panic!("Unrecognized command!")
    }
}

fn spawn_search_thread<M: Move, B: Board<M>>(
    board: Arc<Mutex<B>>,
    search_instruction_rx: Receiver<SearchInstruction>,
    search_info_tx: Sender<SearchInfo<M>>,
    stop_rx: Receiver<bool>,
    search: Search<M, B>
) -> JoinHandle<()> {
    return thread::spawn(move || {

        // duration between successive queries of the search_info channel for instructions
        // TODO: This should be a static somewhere or live in some config
        let retry_duration = std::time::Duration::from_millis(10);

        loop {
            
            // listen for instructions
            match search_instruction_rx.recv() {
                Option::None                     => thread::sleep(retry_duration),
                Option::Some(search_instruction) => {
                    
                    // acquire lock of board
                    let mut locked_board = board.lock().expect("Acquiring of lock failed. Mutex is probably poisoned!");
                    
                    // clear stop signal is there is any
                    let _ = stop_rx.recv();

                    // do the search
                    search(&mut locked_board, search_instruction, &stop_rx, &search_info_tx);
                }
            }
        }
    });
}

pub fn uci_loop<M: Move, B: Board<M>>(search: Search<M, B>) -> std::io::Result<()> where B: Default {

    // main thread owns stdin and is the only thread to write to it
    let stdin = std::io::stdin();

    // make a new board and wrap it in a Arc-Mutex, so that both threads can modify it
    let board = B::default();
    let board_ref = Arc::new(Mutex::new(board));
    let board_ref_for_search_thread = Arc::clone(&board_ref);

    // channels for communicating between the main and the search thread
    let (search_instruction_tx, search_instruction_rx) = channel::<SearchInstruction>();
    let (stop_tx, stop_rx) = channel::<bool>();
    let (search_info_tx, search_info_rx) = channel::<SearchInfo<M>>();
    
    // spawn search thread
    spawn_search_thread(
        board_ref_for_search_thread,
        search_instruction_rx,
        search_info_tx,
        stop_rx,
        search
    );

    loop {

        // read command from stdin
        let command = &mut String::new();
        stdin.read_line(command)?;

        // parse & handle command
        parse_and_handle_uci_command(
            command,
            Arc::clone(&board_ref),
            &search_instruction_tx,
            &stop_tx,
            &search_info_rx
        );

    }

}