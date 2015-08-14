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

fn main() {
    let re = "ab.a.";
    let mut fragments: Vec<Fragment> = Vec::new();
    for ch in re.chars() {
        let newfrag = match ch {
            '.' => {
                let e2 = fragments.pop().expect("Found . without a previous fragment");
                let e1 = fragments.pop().expect("Found . without two previous fragment");
                e1.patch(e2)
            },
            '+' => {
                let e = fragments.pop().expect("Found + without a previous fragment");
                let s = e.state.clone();
                e.patch(Fragment::new(State::Split {
                    one: s,
                    two: State::tip()
                }))
            },
            a => {
                Fragment::new(State::Lit {
                    val: format!("{}", a),
                    next: State::tip()
                })
            }
        };
        fragments.push(newfrag);
    }
}
