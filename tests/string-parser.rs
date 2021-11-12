#![deny(clippy::all)]

use std::fs;

extern crate json_tools;

use json_tools::{Buffer, BufferType, Lexer, Token, TokenType};

type TT = TokenType;


//const inputs: &[&str; 1] = &[
//    r#"  {"face": "bruce", ["one", "two", "three", "😂", 1.8e23]}"#,
//];

const TEXT: &str = r#"  {"face": "bruce", "array": ["one", "t\"wo", "three", "😂", 1.8e23, true, null]}"#;

#[test]
fn visualize() {
    println!("{}", TEXT.len());
    for token in Lexer::new(TEXT.bytes(), BufferType::Span) {
        match token.buf {
            Buffer::Span(span) => {
                let token_text = &TEXT[span.first as usize..span.end as usize];
                println!("{:?} from {} to {}: {}", token.kind, span.first, span.end, token_text);
            },
            _ => println!("{:?}", token.buf)
        }
    }
}

#[test]
fn test_parser() -> Result<(), Box<dyn std::error::Error>> {
    let bytes: String = fs::read_to_string(&"./tests/issue-38.json")?;
    string_parser(&bytes);

    Ok(())
}


fn string_parser(text: &str) {
    //let mut path = &Vec::<Token>::new();
    //let mut state_stack = &Vec::<State>::new();
    let mut parser = Parser {};

    parser.parse(text);
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum State<'a> {
    ExpectAtom,
    Object(ObjectState),
    Array(ArrayState),
    ExpectNothing,
    Done,
    BadJson(&'a str),
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ObjectState {
    MaybeKey,
    NeedColon,
    NeedValue,
    MaybeMore,
    NeedKey,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ArrayState {
    MaybeValue,
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

fn tok_string(input: &str, token: &Token) -> String {
    match &token.buf {
        Buffer::Span(span) => {
            let token_text = &input[span.first as usize..span.end as usize];
            String::from(token_text)
        },
        _ => format!("{:?}", token.buf)
    }

    //String::from(&input.[token.buf.first..token.buf.end])
}

impl Parser {
    fn parse(&mut self, input: &str) {
        let mut path = Vec::<Token>::new();
        //let state_stack = StateStack {stack: &mut Vec::<State>::new()};
        //let state_stack = &mut Vec::<State>::new();
        let mut state_stack: Vec::<State> = vec![State::ExpectNothing];

        let mut state: State = State::ExpectAtom;
        let mut prev_state: State = state;


        let mut pop: bool = false;

        let mut lex_iterator = Lexer::new(input.bytes(), BufferType::Span).peekable();

        'new_token: loop {
            let tok: Token;
            if let Some(token) = lex_iterator.next() {
                tok = token;
            } else {
                //state = State::Done;
                break;
            }

            'same_token: loop {
                if pop {
                    pop = false;
                    let pstate = state;
                    state = state_stack.pop().unwrap();
                    println!("  popping {:?} => {:?}", pstate, state);
                }
                println!("{:?} => {:?} with {:?}", prev_state, state, tok_string(input, &tok));
                prev_state = state;

                match state {
                    State::ExpectAtom => {
                        match tok.kind {
                            TT::BooleanTrue | TT::BooleanFalse | TT::Null | TT::Number => {
                                // have standalone value that we don't care about
                                pop = true;
                            },
                            TT::String => {
                                // TODO scan string for badness and then nothing more should be
                                // present.
                                pop = true;
                            },
                            TT::CurlyOpen => {
                                // now parsing an object, possibly empty
                                state = State::Object(ObjectState::MaybeKey);
                            },
                            TT::BracketOpen => {
                                // parse an array, possibly empty
                                state = State::Array(ArrayState::MaybeValue);
                            },
                            _ => {
                                state = State::BadJson("unexpected @ ExpectAtom");
                            }
                        }
                    },
                    State::Object(object_state) => {
                        match object_state {
                            ObjectState::MaybeKey => {
                                match tok.kind {
                                    TT::String => {
                                        // TODO scan string key for badness
                                        path.push(tok.clone());
                                        state = State::Object(ObjectState::NeedColon);
                                    },
                                    TT::CurlyClose => {
                                        // pop state. i don't think it's possible for
                                        // the stack to be empty because we're in a state
                                        // that implies something is on the stack.
                                        pop = true;
                                    },
                                    _ => {
                                        state = State::BadJson("key or close brace required");
                                    }
                                }
                            },
                            ObjectState::NeedColon => {
                                if tok.kind == TT::Colon {
                                    state = State::Object(ObjectState::NeedValue);
                                } else {
                                    state = State::BadJson("missing colon");
                                }
                            },
                            ObjectState::NeedValue => {
                                state_stack.push(State::Object(ObjectState::MaybeMore));
                                state = State::ExpectAtom;
                                continue 'same_token;
                                /*
                                match tok.kind {
                                    TT::String => {
                                        // TODO scan string
                                        state = State::Object(ObjectState::MaybeMore);
                                    },
                                    TT::BooleanTrue | TT::BooleanFalse | TT::Number => {
                                        state = State::Object(ObjectState::MaybeMore);
                                    },
                                    TT::CurlyOpen => {
                                        state_stack.push(state);
                                        state = State::Object(ObjectState::MaybeKey);

                                    },
                                    TT::BracketOpen => {
                                        state_stack.push(state);
                                        state = State::Array(ArrayState::MaybeValue);
                                    },
                                    _ => {
                                        state = State::BadJson("key requires value");
                                    }
                                }
                                // */
                            },
                            ObjectState::MaybeMore => {
                                match tok.kind {
                                    TT::Comma => {
                                        path.pop().unwrap();
                                        state = State::Object(ObjectState::NeedKey);
                                    },
                                    TT::CurlyClose => {
                                        pop = true;
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
                            ArrayState::MaybeValue => {
                                match tok.kind {
                                    TT::BracketClose => {
                                        pop = true;
                                    },
                                    _ => {
                                        state_stack.push(State::Array(ArrayState::MaybeMore));
                                        state = State::ExpectAtom;
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
                                        pop = true;
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
                    State::Done => {
                        break 'new_token;

                    },
                    State::BadJson(text) => {
                        panic!("{}", text);
                    },
                }
                continue 'new_token;

            }
        }

        let final_state = state_stack.pop();
        if final_state.is_none() {
            panic!("no final state found");
        }
        let final_state = final_state.unwrap();
        if final_state != State::ExpectNothing {
            panic!("final state must be State::ExpectNothing");
        }

        println!("path len {:?}", path.len());
    }
}
