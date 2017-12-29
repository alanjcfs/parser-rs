extern crate unicode_segmentation;
use unicode_segmentation::UnicodeSegmentation;
use std::fs::File;
use std::io::{BufReader, Error, Read};
use regex::Regex;

pub fn read_file(s: &str) -> Result<Vec<String>, Error> {
    let f = File::open(s)?;
    let mut file = BufReader::new(&f);
    let mut string: String = "".to_string();
    file.read_to_string(&mut string)?;
    let unicode_scan: Vec<String> = UnicodeSegmentation::graphemes(&string[..], true).map(|x| x.to_string()).collect();
    Ok(unicode_scan)
}

#[derive(Debug, PartialEq, Clone)]
struct State {
    string: String,
    offset: usize,
}

impl State {
    fn new(string: &str, offset: usize) -> State {
        State { string: string.to_string(), offset: offset }
    }

    fn peek(&self, n: usize) -> Option<String> {
        if self.offset + n <= self.string.len() {
            Some(self.string[self.offset..self.offset + n].to_string())
        } else {
            None
        }
    }

    fn read(&self, n: usize) -> State {
        State::new(&self.string, self.offset + n)
    }

    fn is_complete(&self) -> bool {
        self.offset == self.string.len()
    }
}

#[derive(Debug, PartialEq)]
enum Match {
    Str(String),
    Chr(String),
    Seq(Vec<Match>),
    Rep(Vec<Match>),
}

#[derive(Debug, PartialEq)]
struct MatchState(Match, State);

#[derive(Debug, Clone)]
enum Func {
    Str(String),
    Chr(String),
    Rep(Box<Func>, usize),
    Seq(Vec<Func>),
    Alt(Vec<Func>),
}

trait ParserCombinator {
    fn str_(s: String) -> Box<Fn(State) -> Option<MatchState>>;
    fn chr(pattern: String) -> Box<Fn(State) -> Option<MatchState>>;
    fn seq(combinators: Vec<Func>) -> Box<Fn(State) -> Option<MatchState>>;
    fn rep(combinator: Func, n: usize) -> Box<Fn(State) -> Option<MatchState>>;
    fn alt(parsers: Vec<Func>) -> Box<Fn(State) -> Option<MatchState>>;
}

struct Addition;

impl ParserCombinator for Addition {
    fn str_(s: String) -> Box<Fn(State) -> Option<MatchState>> {
        Box::new(move |state| {
            let chunk = state.peek(s.len());

            if let Some(chunk) = chunk {
                if chunk == s {
                    Some(MatchState(Match::Str(chunk), state.read(s.len())))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn chr(pattern: String) -> Box<Fn(State) -> Option<MatchState>> {
        Box::new(move |state| {
            let chunk = state.peek(1);

            let pattern_regex = Regex::new(&format!("[{}]", pattern)).unwrap();
            if let Some(chunk) = chunk {
                if pattern_regex.is_match(&chunk) {
                    Some(MatchState(Match::Chr(chunk), state.read(1)))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }


    fn seq(combinators: Vec<Func>) -> Box<Fn(State) -> Option<MatchState>> {
        Box::new(move |state| {
            let mut matches = Vec::new();
            // Feed input state through a chain of other combinators, output state of one combinator
            // becomes the input for the next. If all combinators match, return the sequence
            // with state set to the last state otherwise, return None.
            let mut current_state = Some(state.clone());

            for combinator in &combinators {
                let result = match combinator {
                    &Func::Str(ref s) => {
                        Self::str_(s.to_owned())(current_state.clone().unwrap())
                    }
                    &Func::Chr(ref s) => {
                        Self::chr(s.to_owned())(current_state.clone().unwrap())
                    }
                    &Func::Rep(ref s, size) => {
                        Self::rep(*s.to_owned(), size)(current_state.clone().unwrap())
                    }
                    &Func::Seq(ref s) => {
                        Self::seq(s.clone().to_vec())(current_state.clone().unwrap())
                    }
                    &Func::Alt(ref s) => {
                        Self::alt(s.clone().to_vec())(current_state.clone().unwrap())
                    }
                };
                if let Some(MatchState(node, new_state)) = result {
                    matches.push(node);
                    current_state = Some(new_state.clone());
                }
            }

            if current_state.is_some(){
                Some(MatchState(Match::Seq(matches), current_state.unwrap()))
            }
            else {
                None
            }
        })
    }

    fn rep(combinator: Func, n: usize) -> Box<Fn(State) -> Option<MatchState>> {
        Box::new(move |state| {
            let mut matches = Vec::new();
            let mut last_state = None;
            let mut current_state = Some(state.clone());

            while current_state.is_some() {
                let s = current_state.clone().unwrap();
                last_state = Some(s.clone());
                let result = match combinator {
                    Func::Str(ref f) => {
                        Self::str_(f.to_owned())(s.clone())
                    }
                    Func::Chr(ref f) => {
                        Self::chr(f.to_owned())(s.clone())
                    }
                    Func::Rep(ref f, size) => {
                        Self::rep(*f.to_owned(), size)(s.clone())
                    }
                    Func::Seq(ref f) => {
                        Self::seq(f.clone().to_vec())(s.clone())
                    }
                    Func::Alt(ref f) => {
                        Self::alt(f.clone().to_vec())(s.clone())
                    }
                };
                if let Some(MatchState(node, new_state)) = result {
                    current_state = Some(new_state);
                    matches.push(node);
                } else {
                    current_state = None;
                }
            }

            if matches.len() >= n {
                Some(MatchState(Match::Rep(matches), last_state.unwrap().clone()))
            } else {
                None
            }

        })
    }

    fn alt(parsers: Vec<Func>) -> Box<Fn(State) -> Option<MatchState>> {
        Box::new(move |state| {
            for parser in &parsers {
                let r = match parser {
                    &Func::Str(ref f) => {
                        Self::str_(f.to_owned())(state.clone())
                    }
                    &Func::Chr(ref f) => {
                        Self::chr(f.to_owned())(state.clone())
                    }
                    &Func::Rep(ref f, size) => {
                        Self::rep(*f.to_owned(), size)(state.clone())
                    }
                    &Func::Seq(ref f) => {
                        Self::seq(f.clone().to_vec())(state.clone())
                    }
                    &Func::Alt(ref f) => {
                        Self::alt(f.clone().to_vec())(state.clone())
                    }
                };
                if let Some(result) = r {
                    return Some(result)
                }
            }

            None
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let state = State::new(&"I'm just a string", 0);
        assert_eq!(state, State { string: "I'm just a string".to_string(), offset: 0 });
    }

    #[test]
    fn test_peek() {
        let state = State::new(&"I'm just a string", 0);
        assert_eq!(state.peek(8).unwrap(), "I'm just");
    }

    #[test]
    fn test_read() {
        let state = State::new("I'm just a string", 11);
        assert_eq!(state.read(6), State { string: "I'm just a string".to_string(), offset: 17 });
    }

    #[test]
    fn test_is_complete() {
        let state = State::new("I'm just a string", 0);
        assert_eq!(state.is_complete(), false);
        let state = State::new("I'm just a string", 17);
        assert_eq!(state.is_complete(), true);
    }

    #[test]
    fn test_str() {
        let input = State::new("hello world", 0);
        let hello = Addition::str_(r"hello".to_string());
        let world = Addition::str_(r"world".to_string());

        if let Some(MatchState(m, s)) = hello(input) {
            assert_eq!(m, Match::Str("hello".to_string()));
            assert_eq!(s, State::new("hello world", 5));
        } else {
            panic!("no output")

        }

        let input = State::new("hello world", 0);
        if let Some(MatchState(m, s)) = world(input.read(6)) {
            assert_eq!(m, Match::Str("world".to_string()));
            assert_eq!(s, State::new("hello world", 11));
        } else {
            panic!("no output")
        }
    }

    #[test]
    fn test_chr() {
        let input = State::new("12 + 34", 0);
        let digit = Addition::chr(r"0-9".to_string());

        if let Some(MatchState(m, s)) = digit(input.read(1)) {
            assert_eq!(m, Match::Chr("2".to_string()));
            assert_eq!(s, State::new("12 + 34", 2))
        } else {
            panic!("no output")
        }

        if let Some(MatchState(m, s)) = digit(input.read(5)) {
            assert_eq!(m, Match::Chr("3".to_string()));
            assert_eq!(s, State::new("12 + 34", 6))
        } else {
            panic!("no output")
        }

        assert_eq!(digit(input.read(2)), None);
    }

    #[test]
    fn test_seq() {
        let input = State::new("7+8", 0);
        // Sanity check
        let digit = Addition::chr(r"0-9".to_string());
        let reg = Addition::str_("+".to_string());
        assert_eq!(digit(input.read(0)), Some(MatchState(Match::Chr("7".to_string()), State::new("7+8", 1))));
        assert_eq!(reg(input.read(1)), Some(MatchState(Match::Str("+".to_string()), State::new("7+8", 2))));
        assert_eq!(digit(input.read(2)), Some(MatchState(Match::Chr("8".to_string()), State::new("7+8", 3))));


        let addition = Addition::seq(vec![Func::Chr("0-9".to_string()), Func::Str(r"+".to_string()), Func::Chr("0-9".to_string())]);

        let results = addition(input);
        if let Some(MatchState(m, s)) = results {
            assert_eq!(
                m,
                Match::Seq(
                    vec![
                        Match::Chr("7".to_string()),
                        Match::Str("+".to_string()),
                        Match::Chr("8".to_string()),
                        ]));
            assert_eq!(s, State::new("7+8", 3));
        } else {
            panic!("addition(input) did not generate a result")
        }
    }

    #[test]
    fn test_rep() {
        let input = State::new("2017", 0);
        let number = Addition::rep(Func::Chr("0-9".to_string()), 1);
        let results = number(input);
        if let Some(MatchState(m, s)) = results {
            assert_eq!(
                m,
                Match::Rep(
                    vec![
                    Match::Chr("2".to_string()),
                    Match::Chr("0".to_string()),
                    Match::Chr("1".to_string()),
                    Match::Chr("7".to_string()),
                    ]));
        } else {
            panic!("number(input) did not generate a result")
        }
    }

    // In the reproof of tests lies the true proof of a parser combinator
    #[test]
    fn test_alt() {
        let w = Func::Rep(Box::new(Func::Str(" ".to_string())), 0);
        let number = Func::Alt(
            vec![
            Func::Str("0".to_string()),
            Func::Seq(
                vec![
                Func::Chr("1-9".to_string()),
                Func::Rep(Box::new(Func::Chr("0-9".to_string())), 0)
                ],
            )]);
        let addition = Func::Seq(
            vec![
            number.clone(),
            w.clone(),
            Func::Str("+".to_string()), w.clone(), number.clone()
            ]);
        let expression = Addition::alt(vec![addition, number.clone()]);
        let result = expression(State::new("12", 0));
        if let Some(MatchState(m, s)) = result {
            assert_eq!(s,
                       State::new("12", 2));
            assert_eq!(m,
                       Match::Seq(
                           vec![
                           Match::Seq(
                               vec![
                               Match::Chr("1".to_string()),
                               Match::Rep(vec![Match::Chr("2".to_string())])
                               ]
                           ),
                           Match::Rep(vec![]),
                           Match::Rep(vec![]),
                           Match::Seq(vec![Match::Rep(vec![])])
                           ]));
        } else {
            panic!("no results from alt");
        }

        let result = expression(State::new("34 + 567", 0));
        if let Some(MatchState(m, s)) = result {
            assert_eq!(s,
                       State::new("34 + 567", 8));
            assert_eq!(m,
                       Match::Seq(vec![
                           Match::Seq(
                               vec![
                               Match::Chr("3".to_string()),
                               Match::Rep(vec![Match::Chr("4".to_string())])]),
                               Match::Rep(vec![ Match::Str(" ".to_string())]),
                               Match::Str("+".to_string()),
                               Match::Rep(vec![Match::Str(" ".to_string())]),
                               Match::Seq(vec![Match::Chr("5".to_string()),
                                    Match::Rep(vec![Match::Chr("6".to_string()), Match::Chr("7".to_string())])
                               ]
                           )
                       ]));
        } else {
            panic!("no results from alt");
        }

    }
}
