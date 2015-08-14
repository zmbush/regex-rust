#![feature(drain)]
use std::rc::Rc;
use std::cell::{RefCell, RefMut};

type S = Rc<RefCell<Rc<State>>>;
#[derive(Debug)]
enum State {
    Any {
        next: S,
    },
    Lit {
        val: String,
        next: S,
    },
    Split {
        one: S,
        two: S,
    },
    Match
}

#[derive(Debug)]
struct Fragment {
    tips: Vec<S>,
    state: S
}

impl State {
    fn tip() -> S {
        Rc::new(RefCell::new(Rc::new(State::Match)))
    }
    /*
    fn patch_on(self, f: Fragment) -> Fragment {
        use Fragment::*;

        Rc::new(match self {
            Any { next: None } => Any { next: Some(f) },
            Any { next: Some(frag) } => Any { next: Some(frag.patch_on(f)) },

            Lit { val, next: None } => Lit { val: val, next: Some(f) },
            Lit { val, next: Some(frag) } => Lit { val: val, next: Some(frag.patch_on(f)) },

            Split { next: None } => Split { next: Some((f.clone(), f)) },
            Split { next: Some((a, b)) } => Split {
                next: Some(
                    (a.patch_on(f.clone()), b.patch_on(f))
                )
            }
        })
    }
    */
}

impl Fragment {
    fn patch(mut self, f: Fragment) -> Fragment {
        for t in self.tips.drain(..) {
            *t.borrow_mut() = f.state.borrow().clone();
        }
        self.tips = f.tips;
        self
    }

    fn new(s: State) -> Fragment {
        use State::*;

        let tips = match s {
            Any { ref next } => vec![next.clone()],
            Lit { ref next, .. } => vec![next.clone()],
            Split { ref one, ref two } => vec![one.clone(), two.clone()],
            Match => Vec::new()
        };

        Fragment {
            tips: tips,
            state: Rc::new(RefCell::new(Rc::new(s)))
        }
    }
}

#[derive(Debug)]
enum Tok {
    Lit(char),
    Any,
    OneOrMore,
    ZeroOrMore,
    OneOrZero,
    Or,
    Concat
}

fn tokenize(re: &str) -> Result<Vec<Tok>, String> {
    use Tok::*;
    #[derive(Clone, Debug)]
    struct Loc {
        num_atom: i64,
        num_alternatives: i64
    }

    macro_rules! has_atom { ($t:ident) => (if $t.num_atom == 0 {
        return Err("num_atom == 0".to_owned());
    }) }

    macro_rules! maybe_concat { ($t:ident, $r:ident) => (if $t.num_atom > 1 {
        $t.num_atom -= 1;
        $r.push(Concat);
    }) }

    let mut retval = Vec::new();
    let mut parens = Vec::new();
    let mut current = Loc { num_atom: 0, num_alternatives: 0 };
    for c in re.chars() {
        match c {
            '(' => {
                maybe_concat!(current, retval);
                parens.push(current.clone());
                current.num_atom = 0;
                current.num_alternatives = 0;
            },
            '|' => {
                has_atom!(current);
                maybe_concat!(current, retval);
                current.num_alternatives += 1;
            },
            ')' => {
                if parens.len() == 0 {
                    return Err("Mismatched parens".to_owned());
                }
                has_atom!(current);
                while current.num_atom > 1 {
                    current.num_atom -= 1;
                    retval.push(Concat);
                }
                while current.num_alternatives > 0 {
                    current.num_alternatives -= 1;
                    retval.push(Or);
                }
                current = match parens.pop() {
                    Some(v) => v,
                    None => return Err("Mismatched parens".to_owned())
                };
                current.num_atom += 1;
            },
            '*' | '+' | '?' => {
                has_atom!(current);
                retval.push(match c {
                    '*' => ZeroOrMore,
                    '+' => OneOrMore,
                    '?' => OneOrZero,
                    _ => unreachable!()
                })
            }
            a => {
                maybe_concat!(current, retval);
                retval.push(match a {
                    '.' => Any,
                    b => Lit(b)
                });
                current.num_atom += 1;
            }
        }
    }
    if parens.len() > 0 {
        return Err("Mismatched parens".to_owned());
    }
    while current.num_atom > 1 {
        current.num_atom -= 1;
        retval.push(Concat);
    }
    while current.num_alternatives > 0 {
        retval.push(Or);
    }
    Ok(retval)
}

fn main() {
    let re = tokenize("...").unwrap();
    println!("{:?}", re);
    let mut fragments: Vec<Fragment> = Vec::new();
    for tok in re {
        let newfrag = match tok {
            Tok::Concat => {
                let e2 = fragments.pop().expect("Found . without a previous fragment");
                let e1 = fragments.pop().expect("Found . without two previous fragment");
                e1.patch(e2)
            },
            Tok::OneOrMore => {
                let e = fragments.pop().expect("Found + without a previous fragment");
                let s = e.state.clone();
                e.patch(Fragment::new(State::Split {
                    one: s,
                    two: State::tip()
                }))
            },
            Tok::Any => {
                Fragment::new(State::Any {
                    next: State::tip()
                })
            },
            Tok::Lit(a) => {
                Fragment::new(State::Lit {
                    val: format!("{}", a),
                    next: State::tip()
                })
            },
            _ => unreachable!()
        };
        fragments.push(newfrag);
    }
    println!("{:?}", fragments);
}
