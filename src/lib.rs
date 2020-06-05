//! # ReActor
//!
//! An [Actor](https://en.wikipedia.org/wiki/Actor_model) runtime for Rust.
//!

use std::collections::VecDeque;
use std::rc::Rc;
use std::cell::RefCell;

pub trait Behavior {
    fn react(&self, event: Event) -> Effect;
}

pub struct Actor {
    behavior: RefCell<Box<dyn Behavior>>,
}
impl Actor {
    fn new(behavior: Box<dyn Behavior>) -> Rc<Actor> {
        Rc::new(Actor {
            behavior: RefCell::new(behavior),
        })
    }

    fn dispatch(&self, event: Event) -> Effect {
        self.behavior.borrow().react(event)
    }
    fn update(&self, behavior: Box<dyn Behavior>) {
        *self.behavior.borrow_mut() = behavior;
    }
}

pub struct Event {
    target: Rc<Actor>,
    message: Message,
}
impl Event {
    fn new(target: &Rc<Actor>, message: Message) -> Event {
        Event {
            target: Rc::clone(target),
            message: message
        }
    }
}

type Error = &'static str;

pub enum Message {
    Empty,
    Nat(usize),
    Int(isize),
    Str(&'static str),
    Addr(Rc<Actor>),
    Maybe(Option<Box<Message>>),
    Pair(Box<Message>, Box<Message>),
    List(&'static [Box<Message>]),
    OkFail {
        ok: Rc<Actor>,
        fail: Rc<Actor>,
    },
    GetMsg {
        cust: Rc<Actor>,
        name: &'static str,
    },
    SetMsg {
        cust: Rc<Actor>,
        name: &'static str,
        value: Box<Message>,
    },
}

pub struct Effect {
    actors: Vec<Rc<Actor>>,
    events: VecDeque<Event>,
    state: Option<Box<dyn Behavior>>,
    error: Option<Error>,
}
impl Effect {
    pub fn new() -> Effect {
        Effect {
            actors: Vec::new(),
            events: VecDeque::new(),
            state: None,
            error: None,
        }
    }

    pub fn create(&mut self, behavior: Box<dyn Behavior>) -> Rc<Actor> {
        let actor = Actor::new(behavior);
        self.actors.push(Rc::clone(&actor));
        actor
    }
    pub fn send(&mut self, target: &Rc<Actor>, message: Message) {
        let event = Event::new(target, message);
        self.events.push_back(event);
    }
    pub fn update(&mut self, behavior: Box<dyn Behavior>) {
        self.state = Some(behavior);
    }
    pub fn throw(&mut self, reason: Error) {
        self.error = Some(reason);
    }
}

pub struct Config {
    actors: Vec<Rc<Actor>>,
    events: VecDeque<Event>,
}
impl Config {
    pub fn new() -> Config {
        Config {
            actors: Vec::new(),
            events: VecDeque::new(),
        }
    }

    /// Execute bootstrap `behavior` to initialize Config.
    ///
    /// Returns the number of events enqueued.
    pub fn boot(&mut self, behavior: Box<dyn Behavior>) -> usize {
        let actor = Actor::new(behavior);
        self.actors.push(Rc::clone(&actor));  // FIXME: do we need to retain the bootstrap actor?
        let event = Event::new(&actor, Message::Empty);
        self.events.push_back(event);
        self.dispatch(1)  // dispatch bootstrap message
    }

    /// Dispatch up to `limit` events.
    ///
    /// Returns the number of events still waiting in queue.
    pub fn dispatch(&mut self, mut limit: usize) -> usize {
        while limit > 0 {
            if let Some(event) = self.events.pop_front() {
                let target = Rc::clone(&event.target);
                let mut effect = target.dispatch(event);
                match effect.error {
                    None => {
                        if let Some(behavior) = effect.state.take() {
                            target.update(behavior);
                        }
                        self.actors.append(&mut effect.actors);  // FIXME: should convert to Weak references here...
                        self.events.append(&mut effect.events);
                    },
                    Some(reason) => {
                        println!("FAIL! {}", reason);  // FIXME: should deliver a signal to meta-controller
                    },
                }
            } else {
                break;
            }
            limit -= 1;
        }
        self.events.len()  // remaining event count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Sink;
    impl Behavior for Sink {
        fn react(&self, _event: Event) -> Effect {
            Effect::new()
        }
    }

    #[test]
    fn sink_behavior() {
        let sink = Actor::new(Box::new(Sink));

        let event = Event::new(&sink, Message::Empty);
        let effect = sink.dispatch(event);

        assert_eq!(0, effect.actors.len());
        assert_eq!(0, effect.events.len());
        assert_eq!(None, effect.error);
    }

    struct Once {
        cust: Rc<Actor>,
    }
    impl Behavior for Once {
        fn react(&self, event: Event) -> Effect {
            let mut effect = Effect::new();
            effect.send(&self.cust, event.message);
            effect.update(Box::new(Sink {}));
            effect
        }
    }

    #[test]
    fn once_behavior() {
        let sink = Actor::new(Box::new(Sink));
        let once = Actor::new(Box::new(Once {
            cust: Rc::clone(&sink)
        }));

        let event = Event::new(&once, Message::Empty);
        let effect = once.dispatch(event);

        assert_eq!(0, effect.actors.len());
        assert_eq!(1, effect.events.len());
        assert_eq!(None, effect.error);

        if let Some(behavior) = effect.state {
            once.update(behavior);
        } else {
            panic!("expected new state!");
        }

        let event = Event::new(&once, Message::Empty);
        let effect = once.dispatch(event);

        assert_eq!(0, effect.actors.len());
        assert_eq!(0, effect.events.len());
        assert_eq!(None, effect.error);
    }

    struct Maker;
    impl Behavior for Maker {
        fn react(&self, event: Event) -> Effect {
            let mut effect = Effect::new();
            match event.message {
                Message::Addr(cust) => {
                    let actor = effect.create(Box::new(Sink));
                    effect.send(&cust, Message::Addr(Rc::clone(&actor)));
                },
                _ => effect.throw("unknown message"),
            }
            effect
        }
    }

    #[test]
    fn maker_behavior() {
        let maker = Actor::new(Box::new(Maker));

        let event = Event::new(&maker, Message::Empty);
        let effect = maker.dispatch(event);

        assert_eq!(0, effect.actors.len());
        assert_eq!(0, effect.events.len());
        println!("Got error = {:?}", effect.error);
        assert_ne!(None, effect.error);

        let sink = Actor::new(Box::new(Sink));
        let event = Event::new(&maker, Message::Addr(Rc::clone(&sink)));
        let effect = maker.dispatch(event);

        assert_eq!(1, effect.actors.len());
        assert_eq!(1, effect.events.len());
        assert_eq!(None, effect.error);
    }
}
