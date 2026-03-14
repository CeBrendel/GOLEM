

use std::process;
use crate::board::traits::{Move, Board};

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

fn handle_go<M: Move, B: Board<M>>(s: &str, board: &mut B) {

    let (head, _tail) = pop_first(s);

    match head {
        "searchmoves" => todo!(),
        "ponder"      => todo!(),
        "wtime"       => todo!(),
        "btime"       => todo!(),
        "winc"        => todo!(),
        "binc"        => todo!(),
        "movestogo"   => todo!(),
        "depth"       => todo!(),
        "nodes"       => todo!(),
        "mate"        => todo!(),
        "movetime"    => todo!(),
        "infinite"    => todo!(),
        _             => panic!("Received invalid \"go\" command!")
    }

}

pub fn parse_uci_command<M: Move, B: Board<M>>(command: &str, board: &mut B) {

    // remove linebreaks from command
    let command = command.trim();

    // split at first whitespace to obtain command (uci/go/...) and remaining tail
    let (keyword, tail) = pop_first(command);

    match keyword {
        "position"      => handle_position(tail, board),
        "go"            => handle_go(tail, board),
        "isready"       => handle_isready(),
        "ucinewgame"    => todo!(),
        "stop"          => todo!(),
        "ponderhit"     => todo!(),
        "uci"           => handle_uci(),
        "setoption"     => todo!(),
        "quit"          => handle_quit(),
        _               => panic!("Unrecognized command!")
    }
}