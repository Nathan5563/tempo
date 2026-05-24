use std::{
    io::{self, BufRead, Write},
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread,
    time::Duration,
};

use super::position::Move;
use super::search::SearchLimits;
use super::{Engine, STARTPOS_FEN};

const ENGINE_NAME: &str = "Tempo";
const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
const ENGINE_AUTHOR: &str = "Nathan";
const POLL_INTERVAL: Duration = Duration::from_millis(10);

pub fn run() -> io::Result<()>
{
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move ||
    {
        let stdin = io::stdin();
        for line in stdin.lock().lines()
        {
            let Ok(line) = line else
            {
                break;
            };

            if sender.send(line).is_err()
            {
                break;
            }
        }
    });

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    run_event_loop(receiver, &mut out)
}

fn run_event_loop<W: Write>(
    receiver: Receiver<String>,
    out: &mut W,
) -> io::Result<()>
{
    let mut session = Session { engine: Engine::new() };

    loop
    {
        match receiver.recv_timeout(POLL_INTERVAL)
        {
            Ok(line) =>
            {
                if session.handle_line(&line, out)?
                {
                    break;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) =>
            {
                session.emit_stopped_search(out)?;
                break;
            }
        }

        session.emit_finished_search(out)?;
        out.flush()?;
    }

    out.flush()
}

struct Session
{
    engine: Engine,
}

impl Session
{
    fn handle_line<W: Write>(
        &mut self,
        line: &str,
        out: &mut W,
    ) -> io::Result<bool>
    {
        let mut tokens = line.split_ascii_whitespace();
        let Some(command) = tokens.next() else
        {
            return Ok(false);
        };

        match command
        {
            "uci" =>
            {
                writeln!(out, "id name {} {}", ENGINE_NAME, ENGINE_VERSION)?;
                writeln!(out, "id author {}", ENGINE_AUTHOR)?;
                writeln!(out, "uciok")?;
            }
            "isready" => writeln!(out, "readyok")?,
            "ucinewgame" => self.engine.new_game(),
            "position" =>
            {
                if let Err(err) = self.handle_position(tokens)
                {
                    writeln!(out, "info string {}", err)?;
                }
            }
            "go" =>
            {
                let limits = parse_go(tokens);
                self.engine.start_search(limits);
            }
            "stop" => self.emit_stopped_search(out)?,
            "quit" =>
            {
                let _ = self.engine.stop_search();
                return Ok(true);
            }
            _ => {}
        }

        Ok(false)
    }

    fn handle_position<'a, I>(&mut self, tokens: I) -> Result<(), String>
    where
        I: Iterator<Item = &'a str>,
    {
        let tokens = tokens.collect::<Vec<_>>();
        let Some(kind) = tokens.first().copied() else
        {
            return Err("position command is missing a position".to_owned());
        };

        let (fen, move_index) = match kind
        {
            "startpos" => (STARTPOS_FEN.to_owned(), 1),
            "fen" =>
            {
                if tokens.len() < 7
                {
                    return Err(
                        "position fen requires six FEN fields".to_owned()
                    );
                }

                (tokens[1..7].join(" "), 7)
            }
            _ =>
            {
                return Err(
                    "position command must use startpos or fen".to_owned()
                );
            }
        };

        let moves = if move_index == tokens.len()
        {
            &[][..]
        }
        else
        {
            if tokens[move_index] != "moves"
            {
                return Err("position moves must follow moves token".to_owned());
            }

            &tokens[move_index + 1..]
        };

        self.engine
            .set_position(&fen, moves)
            .map_err(|err| err.to_string())
    }

    fn emit_finished_search<W: Write>(&mut self, out: &mut W) -> io::Result<()>
    {
        if let Some(result) = self.engine.poll_search()
        {
            write_bestmove(out, result)?;
        }

        Ok(())
    }

    fn emit_stopped_search<W: Write>(&mut self, out: &mut W) -> io::Result<()>
    {
        if let Some(result) = self.engine.stop_search()
        {
            write_bestmove(out, result)?;
        }

        Ok(())
    }
}

fn parse_go<'a, I>(tokens: I) -> SearchLimits
where
    I: Iterator<Item = &'a str>,
{
    let mut limits = SearchLimits::default();
    let mut tokens = tokens;

    while let Some(token) = tokens.next()
    {
        match token
        {
            "depth" =>
            {
                if let Some(depth) = tokens.next()
                {
                    limits.depth = depth.parse().ok();
                }
            }
            "movetime" =>
            {
                if let Some(movetime) = tokens.next()
                {
                    limits.movetime_ms = movetime.parse().ok();
                }
            }
            "infinite" => limits.infinite = true,
            _ => {}
        }
    }

    limits
}

fn write_bestmove<W: Write>(
    out: &mut W,
    best_move: Option<Move>,
) -> io::Result<()>
{
    match best_move
    {
        Some(mov) => writeln!(out, "bestmove {}", mov),
        None => writeln!(out, "bestmove 0000"),
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn run_script(lines: &[&str]) -> String
    {
        let mut session = Session { engine: Engine::new() };
        let mut out = Vec::new();

        for line in lines
        {
            if session.handle_line(line, &mut out).unwrap()
            {
                break;
            }
            drain_search(&mut session, &mut out);
        }

        session.emit_stopped_search(&mut out).unwrap();

        String::from_utf8(out).unwrap()
    }

    fn drain_search(session: &mut Session, out: &mut Vec<u8>)
    {
        for _ in 0..100
        {
            session.emit_finished_search(out).unwrap();
            if session.engine.search.is_none()
            {
                return;
            }

            std::thread::sleep(POLL_INTERVAL);
        }
    }

    fn assert_single_bestmove(output: &str, expected: &str)
    {
        let bestmoves = output
            .lines()
            .filter(|line| line.starts_with("bestmove "))
            .collect::<Vec<_>>();

        assert_eq!(bestmoves, vec![format!("bestmove {}", expected)]);
    }

    #[test]
    fn uci_and_isready_report_handshake_lines()
    {
        let output = run_script(&["uci", "isready", "quit"]);

        assert_eq!(
            output,
            format!(
                "id name Tempo {}\nid author Nathan\nuciok\nreadyok\n",
                env!("CARGO_PKG_VERSION"),
            ),
        );
    }

    #[test]
    fn position_startpos_moves_affects_next_search()
    {
        let output = run_script(&["position startpos moves e2e4", "go"]);

        assert_single_bestmove(&output, "a7a6");
    }

    #[test]
    fn position_fen_affects_next_search()
    {
        let output = run_script(&[
            "position fen 4k3/8/8/8/8/8/4P3/4K3 w - - 0 1",
            "go",
        ]);

        assert_single_bestmove(&output, "e2e3");
    }

    #[test]
    fn ucinewgame_resets_position()
    {
        let output = run_script(&[
            "position startpos moves e2e4",
            "ucinewgame",
            "go",
        ]);

        assert_single_bestmove(&output, "a2a3");
    }

    #[test]
    fn no_legal_moves_reports_null_bestmove()
    {
        let output = run_script(&[
            "position fen 7k/5Q2/6K1/8/8/8/8/8 b - - 0 1",
            "go",
        ]);

        assert_single_bestmove(&output, "0000");
    }

    #[test]
    fn stop_without_search_emits_no_bestmove()
    {
        let output = run_script(&["stop"]);

        assert_eq!(output, "");
    }

    #[test]
    fn go_parses_supported_limits()
    {
        let limits = parse_go(
            ["depth", "4", "movetime", "125", "infinite"].into_iter()
        );

        assert_eq!(limits.depth, Some(4));
        assert_eq!(limits.movetime_ms, Some(125));
        assert!(limits.infinite);
    }

    #[test]
    fn malformed_position_reports_info_string_and_keeps_position()
    {
        let output = run_script(&[
            "position startpos moves e2e4",
            "position fen 8/8/8",
            "go",
        ]);

        assert!(
            output.contains("info string position fen requires six FEN fields")
        );
        assert_single_bestmove(&output, "a7a6");
    }

    #[test]
    fn malformed_position_moves_keep_position()
    {
        let output = run_script(&[
            "position startpos moves e2e4",
            "position startpos e2e4",
            "position startpos moves e2e5",
            "go",
        ]);

        assert!(
            output.contains("info string position moves must follow moves token")
        );
        assert!(output.contains("info string illegal move: e2e5"));
        assert_single_bestmove(&output, "a7a6");
    }

    #[test]
    fn quit_exits_without_output()
    {
        let output = run_script(&["quit", "uci"]);

        assert_eq!(output, "");
    }
}
