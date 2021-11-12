#![deny(clippy::all)]

use std::{fs, io};

extern crate json_tools;

use json_tools::{Buffer, BufferType, Lexer, Token, TokenType};

type TT = TokenType;


//const inputs: &[&str; 1] = &[
//    r#"  {"face": "bruce", ["one", "two", "three", "😂", 1.8e23]}"#,
//];

/*
const TEXT: &str = r#"  {"face": "bruce", "array": {"faked": ["one", "t\"wo", "three", "😂", 1.8e23, true, null]}, "tail": "end"}"#;
const T2: &str = r#"{"face": "bruce", "one": {"two": {"three": {"last": 5}}}}"#;
const T3: &str = r#"{"one":{"a":[{"x": 1},{"y": 2}, {"z": 3}]}, "two": 3}"#;
const T4: &str = r#"{"one":{}}"#;
// */

#[test]
fn visualize() {
    let input = fs::read_to_string(&"./tests/index-3-issue-38.json").unwrap();
    println!("{}", input.len());
    for token in Lexer::new(input.bytes(), BufferType::Span) {
        match token.buf {
            Buffer::Span(span) => {
                let token_text = &input[span.first as usize..span.end as usize];
                println!("{:?} from {} to {}: {}", token.kind, span.first, span.end, token_text);
            },
            _ => println!("{:?}", token.buf)
        }
    }
}

// annotations, endscreen, playerResponse,
// serviceEndpoints, signInEndpoint
#[test]
fn test_parser() -> Result<(), Box<dyn std::error::Error>> {
    //let bytes: String = fs::read_to_string(&"./tests/issue-38.json")?;
    //let bytes: String = fs::read_to_string(&"./tests/xsrf-1/annotations/subscribeButtonRenderer/items/serviceEndpoints.json")?;
    let bytes: String = fs::read_to_string(&"./tests/issue-38.json")?;
    string_parser(&bytes);

    Ok(())
}

#[test]
fn full_json_test_suite_x() -> Result<(), Box<dyn std::error::Error>> {

    let mut entries = fs::read_dir("./JSONTestSuite")?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_none() || path.extension().unwrap() != "json" {
            continue;
        }
        let beginning = path.file_name().unwrap().to_str().unwrap();
        if !beginning.starts_with("y_") && !beginning.starts_with("i_") {
            continue;
        }
        let test = std::path::Path::new("./JSONTestSuite");
        let test = test.join(path.file_name().unwrap());
        println!("testing {:?}", test.to_str().unwrap());


        if beginning.starts_with("y_") {
            let bytes: String = fs::read_to_string(test.to_str().unwrap())?;
            string_parser(&bytes)?;
        } else if beginning.starts_with("i_") {
            if let Ok(bytes) = fs::read_to_string(test.to_str().unwrap()) {
                if let Ok(_e) = string_parser(&bytes) {
                    println!("passed");
                } else {
                    println!("failed parsing");
                }
            } else {
                println!("failed reading file");
            }
        }
    }

    Ok(())
}


fn string_parser(text: &str) -> Result<(), ParseError> {
    //let mut path = &Vec::<Token>::new();
    //let mut state_stack = &Vec::<State>::new();
    let mut parser = Parser {};

    parser.parse(text)
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum State<'a> {
    ExpectAtom,
    Object(ObjectState),
    Array(ArrayState),
    ExpectNothing,
    _Done,
    BadJson(&'a str),
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ObjectState {
    FirstKey,
    NeedColon,
    MaybeMore,
    NeedKey,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ArrayState {
    MaybeAtom,
    MaybeMore,
}

struct _StateStack<'a> {
    stack: &'a mut Vec<State<'a>>,
}

impl<'a> _StateStack<'a> {
    fn _push(&mut self, state: State<'a>) {
        #[cfg(test)]
        println!("  pushing {:?}", state);
        self.stack.push(state);
    }
    fn _pop(&mut self) -> State {
        let state = self.stack.pop().unwrap();
        #[cfg(test)]
        println!("  popping {:?}", state);
        state
    }
}

struct Parser {
    //path: &'a Vec<Token>,
    //state_stack: &'a Vec<State<'a>>,
}

impl Parser {
    fn parse(&mut self, input: &str) -> Result<(), ParseError> {
        let mut path = Vec::<Token>::new();
        let mut state: State = State::ExpectAtom;
        let mut state_stack: Vec::<State> = vec![State::ExpectNothing];
        let mut prev_state: State = state;

        let mut pop_state: bool = false;

        let mut lex_iterator = Lexer::new(input.bytes(), BufferType::Span).peekable();

        'new_token: loop {
            let tok: Token;
            if let Some(token) = lex_iterator.next() {
                tok = token;
            } else {
                // can't set state to State::Done because tok is not defined
                // in this loop. break results in the last state being left on
                // the stack; it's checked at the end.
                //state = State::Done;
                break;
            }

            //println!("(token {})", tok_string(input, &tok));

            // continue 'same_token effectively turns the previous lex_iterator.next()
            // into a peek.
            'same_token: loop {
                if pop_state {
                    pop_state = false;
                    let pstate = state;
                    state = state_stack.pop().unwrap();
                    //println!("  popping {:?} => {:?}", pstate, state);
                }
                //print_path("cur", input, &path);
                //println!("{:?} => {:?} with {:?}", prev_state, state, tok_string(input, &tok));
                prev_state = state;

                match state {
                    State::ExpectAtom => {
                        match tok.kind {
                            TT::BooleanTrue | TT::BooleanFalse | TT::Null | TT::Number => {
                                // have standalone value that we don't care about
                                pop_state = true;
                            },
                            TT::String => {
                                // TODO scan string for badness and then nothing more should be
                                // present.
                                pop_state = true;
                            },
                            TT::CurlyOpen => {
                                // now parsing an object, possibly empty
                                state = State::Object(ObjectState::FirstKey);
                            },
                            TT::BracketOpen => {
                                // parse an array, possibly empty
                                state = State::Array(ArrayState::MaybeAtom);
                            },
                            _ => {
                                state = State::BadJson("unexpected @ ExpectAtom");
                            }
                        }
                    },
                    State::Object(object_state) => {
                        match object_state {
                            ObjectState::FirstKey => {
                                match tok.kind {
                                    TT::String => {
                                        // TODO scan string key for badness
                                        path.push(tok.clone());
                                        //println!("pushed path on String in ObjectState::FirstKey {}", path.len());
                                        state = State::Object(ObjectState::NeedColon);
                                    },
                                    TT::CurlyClose => {
                                        // pop state. i don't think it's possible for
                                        // the stack to be empty because we're in a state
                                        // that implies something is on the stack.
                                        pop_state = true;
                                        //path.pop().unwrap();
                                        if path.is_empty() {
                                            //println!("path was emptied on {:?} {}", tok, tok_string(input, &tok));
                                        }
                                        //println!("popped path on CurlyClose in ObjectState::FirstKey {}", path.len());
                                    },
                                    _ => {
                                        state = State::BadJson("key or close brace required");
                                    }
                                }
                            },
                            ObjectState::NeedColon => {
                                if tok.kind == TT::Colon {
                                    //state = State::Object(ObjectState::NeedAtom);
                                    state_stack.push(State::Object(ObjectState::MaybeMore));
                                    state = State::ExpectAtom;
                                } else {
                                    state = State::BadJson("missing colon");
                                }
                            },
                            ObjectState::MaybeMore => {
                                match tok.kind {
                                    TT::Comma => {
                                        path.pop().unwrap();
                                        if path.is_empty() {
                                            //println!("path was emptied on {:?} {}", tok, tok_string(input, &tok));
                                        }
                                        //println!("popped path on Comma in ObjectState::MaybeMore {}", path.len());
                                        state = State::Object(ObjectState::NeedKey);
                                    },
                                    TT::CurlyClose => {
                                        pop_state = true;
                                        path.pop().unwrap();
                                        if path.is_empty() {
                                            //println!("path was emptied on {:?} {}", tok, tok_string(input, &tok));
                                        }
                                        //println!("popped state and path on CurlyClose in ObjectState::MaybeMore {}", path.len());
                                    },
                                    _ => {
                                        state = State::BadJson("expected comma or right brace");
                                    }
                                }
                            },
                            ObjectState::NeedKey => {
                                match tok.kind {
                                    TT::String => {
                                        // TODO scan string for badness
                                        path.push(tok.clone());
                                        //println!("pushed path on String in ObjectState::NeedKey {}", path.len());
                                        state = State::Object(ObjectState::NeedColon);
                                    },
                                    _ => {
                                        state = State::BadJson("missing key");
                                    }
                                }
                            }
                        }
                    },
                    State::Array(array_state) => {
                        match array_state {
                            ArrayState::MaybeAtom => {
                                match tok.kind {
                                    TT::BracketClose => {
                                        pop_state = true;
                                    },
                                    _ => {
                                        state_stack.push(State::Array(ArrayState::MaybeMore));
                                        state = State::ExpectAtom;
                                        //println!("  re-using {:?}", tok.kind);
                                        continue 'same_token;
                                    },
                                }
                            },
                            ArrayState::MaybeMore => {
                                match tok.kind {
                                    TT::Comma => {
                                        state_stack.push(State::Array(ArrayState::MaybeMore));
                                        state = State::ExpectAtom;
                                    },
                                    TT::BracketClose => {
                                        pop_state = true;
                                    },
                                    _ => {
                                        state = State::BadJson("expected comma or right bracket");
                                    }
                                }
                            },
                        }
                    },
                    State::ExpectNothing => {
                        panic!("popped an unpushed state, back to State::ExpectNothing");
                    },
                    State::_Done => {
                        break 'new_token;

                    },
                    State::BadJson(text) => {
                        return Err(ParseError::new(text));
                    },
                }
                continue 'new_token;

            }
        }

        let final_state = state_stack.pop();
        if final_state.is_none() {
            return Err(ParseError::new("no final state found"));
        }
        let final_state = final_state.unwrap();
        if final_state != State::ExpectNothing {
            return Err(ParseError::new("final state must be State::ExpectNothing"));
        }

        println!("path len {:?}", path.len());
        if path.len() < 10 {
            print_path("final", input, &path);
        }

        Ok(())
    }
}


fn tok_string(input: &str, token: &Token) -> String {
    match &token.buf {
        Buffer::Span(span) => {
            let token_text = &input[span.first as usize..span.end as usize];
            let s = String::from(token_text);
            if s.chars().count() > 60 {
                s.chars().into_iter().take(60).collect()
            } else {
                s
            }
        },
        _ => format!("{:?}", token.buf)
    }
}

fn print_path(text: &str, input: &str, path: &[Token]) {
    if path.len() > 10 {
        println!("  path len {}", path.len());
        return;
    }
    print!("  {} path: ", text);
    for token in path.iter() {
        let s = tok_string(input, token);
        print!("{}.", s);
    }
    println!();
}

#[derive(Debug)]
struct ParseError {
    details: String
}
impl ParseError {
    pub fn new(msg: &str) -> ParseError {
        ParseError {details: msg.to_string()}
    }
}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}
impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        &self.details
    }
}
