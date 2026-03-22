
mod text_parsing;

use crate::board::{Move, Board};
use crate::search::{SearchInstruction, SearchInfo, SearchResult, Search};
use crate::channeling::{channel, Sender, Receiver};
use crate::uci::text_parsing::{pop_first, parse_next_block_as_usize, collect_blocks_until_next_keyword_or_end};

use std::sync::mpsc::{channel as queueing_channel, Sender as QueueingSender, Receiver as QueueingReceiver};

use std::{thread, thread::JoinHandle};
use std::sync::{Arc, Mutex};


#[derive(Clone)]
pub enum Response<M: Move> {
    // todo: option, copyprotection?
    UciResponse,
    ReadyOk,
    Info(SearchInfo<M>),
    Bestmove(SearchResult<M>)
}

fn emit_request_for_uci_response<M: Move>(write_request_tx: &QueueingSender<Response<M>>) {
    write_request_tx.send(Response::UciResponse).expect("Sending instruction failed!")
}

fn emit_request_for_readyok<M: Move>(write_request_tx: &QueueingSender<Response<M>>) {
    write_request_tx.send(Response::ReadyOk).expect("Sending instruction failed!")
}

fn emit_quit_signal(quit_tx: &QueueingSender<()>) {
    quit_tx.send(()).expect("Quit signal could not be sent!")
}

fn emit_stop_signal(stop_tx: &Sender<()>) {
    stop_tx.send(())
}

fn emit_uci_response() {
    // TODO: Show options!
    println!("id name גֹלֶם (GOLEM)");
    println!("id author Cedric Brendel");
    println!("uciok");
    println!("");
}

fn emit_readyok() {
    println!("readyok");
    println!("");
}

fn emit_search_info<M: Move>(search_info: SearchInfo<M>) {
    
    let mut s = String::from("info");

    // turn score into a string
    let score = match &search_info.score {
        Option::None                => Option::None,
        Option::Some(score) => Option::Some(score.as_str())
    };

    let pv_line: Option<String> = match &search_info.principal_variation_line {
        Option::None             => Option::None,
        Option::Some(v) => Option::Some(v.iter().map(|r#move| r#move.as_str()).collect::<Vec<_>>().join(" "))
    };

    macro_rules! maybe_append {
        ($description: expr, $option: expr) => {
            match $option {
                Option::None    => {},
                Option::Some(t) => {s += &format!($description, t)}
            }
        };
    }

    maybe_append!(" depth {}", search_info.depth);
    maybe_append!(" time {}", search_info.time);
    maybe_append!(" nodes {}", search_info.nodes);
    maybe_append!(" score {}", score);
    maybe_append!(" pv {}", pv_line);

    println!("{}", s);

}

fn emit_search_result<M: Move>(search_result: SearchResult<M>) {
    println!("bestmove {}\n", search_result.bestmove.as_str());
}

fn handle_position<M: Move, B: Board<M>>(s: &str, board: Arc<Mutex<B>>) {

    // get keyword
    let (startpos_or_fen_keyword, tail) = pop_first(s);

    // split at "moves" (if it occurs) to obtain infromation for base position and moves
    let (maybe_fen_with_fen_keyword, maybe_moves) = match tail.find("moves") {
        Option::None             => (tail, Option::None),
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

fn handle_go(s: &str, search_instruction_tx: &QueueingSender<SearchInstruction>) {

    // make empty SearchInstruction
    let mut search_instructions = SearchInstruction::default();

    // split command into blocks and handle them separately
    // make peekable so that we can determine scope of searchmoves command without advancing iterator
    let mut blocks = s.split_whitespace().peekable();
    loop {

        // for more succinct parsing of numbers
        macro_rules! write_field {
            ($field: ident) => {
                {
                    let value = parse_next_block_as_usize(&mut blocks);
                    search_instructions.$field = Option::Some(value);
                }
            };
        }

        // get next block and parse it
        match blocks.next() {
            Option::None          => {break;}
            Option::Some(keyword) => {

                match keyword {
                    "wtime"       => write_field!(wtime_in_ms),
                    "btime"       => write_field!(btime_in_ms),
                    "winc"        => write_field!(winc_in_ms),
                    "binc"        => write_field!(binc_in_ms),
                    "movestogo"   => write_field!(movestogo),
                    "depth"       => write_field!(depth),
                    "movetime"    => write_field!(movetime_in_ms),
                    "infinite"    => {search_instructions.infinite = true;},
                    "searchmoves" => {
                        let collected_blocks = collect_blocks_until_next_keyword_or_end(&mut blocks);
                        search_instructions.searchmoves = Option::Some(collected_blocks);
                    },

                    "nodes"       => todo!("Engine currently does not support only searching a fixed number of nodes."),
                    "mate"        => todo!("Engine currently does not support mate search."),
                    "ponder"      => todo!("Engine currently does not support pondering!"),

                    _             => panic!("Received invalid \"go\" command! Keyword: {}, Remaining blocks: {:?}", keyword, blocks)
                }
            }
        }
    };

    // send search instruction to search thread
    search_instruction_tx.send(search_instructions).expect("Sending failed!");

}

fn parse_and_handle_uci_command<M: Move, B: Board<M>>(
    command: &str,
    board: Arc<Mutex<B>>,
    search_instruction_tx: &QueueingSender<SearchInstruction>,
    stop_tx: &Sender<()>,
    write_request_tx: &QueueingSender<Response<M>>,
    quit_tx: &QueueingSender<()>
) {

    // remove linebreaks from command
    let command = command.trim();

    // split at first whitespace to obtain command (uci/go/...) and remaining tail
    let (keyword, tail) = pop_first(command);

    match keyword {
        "uci"        => emit_request_for_uci_response(write_request_tx),
        "isready"    => emit_request_for_readyok(write_request_tx),
        "position"   => handle_position(tail, board),
        "go"         => handle_go(tail, search_instruction_tx),
        "stop"       => emit_stop_signal(stop_tx),
        "ucinewgame" => todo!("Engine does not currently support the \"ucinewgame\" command!"),
        "ponderhit"  => todo!("Engine does not currently support pondering!"),
        "setoption"  => todo!(),
        "quit"       => emit_quit_signal(quit_tx),
        _            => ()  // simply ignore unrecognized keywords
    }
}

fn spawn_parsing_thread<M: Move, B: Board<M>>(
    board_ref: Arc<Mutex<B>>,
    search_instruction_tx: QueueingSender<SearchInstruction>,
    write_request_tx: QueueingSender<Response<M>>,
    stop_tx: Sender<()>,
    quit_tx: QueueingSender<()>
) -> JoinHandle<()> {
    return thread::spawn(move || {

        // this thread is the only thread that listens and writes to stdin/stdout
        let stdin = std::io::stdin();

        loop {

            // read command from stdin
            let command = &mut String::new();
            match stdin.read_line(command) {
                Err(_) => continue,  // if reading failed, simply ignore and continue
                Ok(_)  => ()
            };

            // parse & handle command
            parse_and_handle_uci_command(
                command,
                Arc::clone(&board_ref),
                &search_instruction_tx,
                &stop_tx,
                &write_request_tx,
                &quit_tx
            );

        }

    });
}

fn spawn_stdout_writer<M: Move>(
    write_request_rx: QueueingReceiver<Response<M>>
) -> JoinHandle<()> {
    return thread::spawn(move || {
        
        // repeatedly listen for messages in the channel
        loop {

            // wait for message to come through
            let message = match write_request_rx.recv() {
                Err(_)                   => panic!("Write request channel is hung up!"),
                Ok(message) => message
            };

            // handle message
            match message {
                Response::<M>::UciResponse                  => emit_uci_response(),
                Response::<M>::ReadyOk                      => emit_readyok(),
                Response::<M>::Info(info)    => emit_search_info(info),
                Response::Bestmove(result) => emit_search_result(result),
            }

        }

    });
}

fn spawn_search_thread<M: Move, B: Board<M>>(
    board: Arc<Mutex<B>>,
    search_instruction_rx: QueueingReceiver<SearchInstruction>,
    stop_rx: Receiver<()>,
    write_request_tx: QueueingSender<Response<M>>,
    search: Search<M, B>
) -> JoinHandle<()> {
    return thread::spawn(move || {

        // repeatedly listen for search instructions
        loop {
            
            // listen for instructions
            let search_instructions = match search_instruction_rx.recv() {
                Err(_)                              => panic!("Search instruction channel is hung up!"),
                Ok(instructions) => instructions
            };
                    
            // acquire lock of board
            let mut locked_board = board.lock().expect("Acquiring of lock failed. Mutex is probably poisoned!");
            
            // clear stop signal is there is any
            let _ = stop_rx.recv();

            // do the search
            let result = search(&mut locked_board, search_instructions, &stop_rx, &write_request_tx);
        
            // send the result
            write_request_tx.send(Response::Bestmove(result)).expect("Sending of search result failed!");

        }

    });
}

pub fn uci_loop<M: Move, B: Board<M>>(search: Search<M, B>) where B: Default {

    // make a new board and wrap it in a Arc-Mutex, so that both threads can modify it
    let board = B::default();
    let board_ref_for_parsing_thread = Arc::new(Mutex::new(board));
    let board_ref_for_search_thread = Arc::clone(&board_ref_for_parsing_thread);

    // channels for communicating between the main and the search thread
    let (search_instruction_tx, search_instruction_rx) = queueing_channel::<SearchInstruction>();
    let (stop_tx, stop_rx) = channel::<()>();
    let (quit_tx, quit_rx) = queueing_channel::<()>();
    let (write_request_tx, write_request_rx) = queueing_channel::<Response<M>>();

    // make multiple Senders for the write request channel
    let write_request_tx_for_parsing_thread = write_request_tx.clone();
    let write_request_tx_for_search_thread = write_request_tx;

    // spawn thread for reading stdin and parsing commands
    spawn_parsing_thread(
        board_ref_for_parsing_thread, 
        search_instruction_tx,
        write_request_tx_for_parsing_thread,
        stop_tx,
        quit_tx
    );

    // spawn thread for reading stdout
    spawn_stdout_writer(write_request_rx);

    // spawn search thread
    spawn_search_thread(
        board_ref_for_search_thread,
        search_instruction_rx,
        stop_rx,
        write_request_tx_for_search_thread,
        search
    );

    // wait for signal to quit
    match quit_rx.recv() {
        Err(_) => panic!("Quit channel is hung up!"),
        Ok(_)  => std::process::exit(0)
    }

}