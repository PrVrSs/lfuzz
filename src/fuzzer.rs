use std::cell::Cell;
use rand::Rng;

use crate::app::App;
use crate::error::Result;


#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    Click { idx: usize },
    KeyPress { key: usize },
    Close,
}


pub fn fuzzer(app: App, predicate: &dyn Fn() -> bool) -> Result<Vec<Action>> {
    let mut actions = Vec::new();
    let mut rng = rand::thread_rng();

    loop {
        if predicate() {
            return Ok(actions)
        }

        let key = ((rng.gen::<usize>() % 10) as u8 + b'0') as usize;
        actions.push(Action::KeyPress { key });

        actions.push(Action::Close);
        app.close();
    }
}
