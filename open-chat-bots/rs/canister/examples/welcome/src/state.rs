use crate::model::messages::WelcomeMessages;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::default();
}

#[derive(Serialize, Deserialize)]
pub struct State {
    oc_public_key: String,
    pub messages: WelcomeMessages,
}

const STATE_ALREADY_INITIALIZED: &str = "State has already been initialized";
const STATE_NOT_INITIALIZED: &str = "State has not been initialized";

pub fn init(state: State) {
    STATE.with_borrow_mut(|s| {
        if s.is_some() {
            panic!("{}", STATE_ALREADY_INITIALIZED);
        } else {
            *s = Some(state);
        }
    })
}

pub fn read<F: FnOnce(&State) -> R, R>(f: F) -> R {
    STATE.with_borrow(|s| f(s.as_ref().expect(STATE_NOT_INITIALIZED)))
}

pub fn mutate<F: FnOnce(&mut State) -> R, R>(f: F) -> R {
    STATE.with_borrow_mut(|s| f(s.as_mut().expect(STATE_NOT_INITIALIZED)))
}

pub fn take() -> State {
    STATE.take().expect(STATE_NOT_INITIALIZED)
}

impl State {
    pub fn new(oc_public_key: String) -> State {
        State {
            oc_public_key,
            messages: WelcomeMessages::new(),
        }
    }

    pub fn update(&mut self, oc_public_key: Option<String>) {
        if let Some(oc_public_key) = oc_public_key {
            self.oc_public_key = oc_public_key;
        }
    }

    pub fn oc_public_key(&self) -> &str {
        &self.oc_public_key
    }
}
