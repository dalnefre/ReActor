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

pub enum Message {
    Empty,
    Nat(usize),
    Int(isize),
    Str(&'static str),
    Addr(Rc<Actor>),
    OkFail {
        ok: Rc<Actor>,
        fail: Rc<Actor>,
    },
    List(Vec<Message>),
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
    error: Option<&'static str>,
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
    pub fn throw(&mut self, reason: &'static str) {
        self.error = Some(reason);
    }

    fn actor_count(&self) -> usize {
        self.actors.len()
    }
    fn event_count(&self) -> usize {
        self.events.len()
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
        let sink = Actor::new(Box::new(Sink {}));

        let event = Event::new(&sink, Message::Empty);
        let effect = sink.dispatch(event);

        assert_eq!(0, effect.actor_count());
        assert_eq!(0, effect.event_count());
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
        let sink = Actor::new(Box::new(Sink {}));
        let once = Actor::new(Box::new(Once {
            cust: Rc::clone(&sink)
        }));

        let event = Event::new(&once, Message::Empty);
        let effect = once.dispatch(event);

        assert_eq!(0, effect.actor_count());
        assert_eq!(1, effect.event_count());
        assert_eq!(None, effect.error);

        if let Some(behavior) = effect.state {
            once.update(behavior);
        } else {
            panic!("expected new state!");
        }

        let event = Event::new(&once, Message::Empty);
        let effect = once.dispatch(event);

        assert_eq!(0, effect.actor_count());
        assert_eq!(0, effect.event_count());
        assert_eq!(None, effect.error);
    }

    struct Maker;
    impl Behavior for Maker {
        fn react(&self, event: Event) -> Effect {
            let mut effect = Effect::new();
            match event.message {
                Message::Addr(cust) => {
                    let actor = effect.create(Box::new(Sink {}));
                    effect.send(&cust, Message::Addr(Rc::clone(&actor)));
                },
                _ => effect.throw("unknown message"),
            }
            effect
        }
    }

    #[test]
    fn maker_behavior() {
        let maker = Actor::new(Box::new(Maker {}));

        let event = Event::new(&maker, Message::Empty);
        let effect = maker.dispatch(event);

        assert_eq!(0, effect.actor_count());
        assert_eq!(0, effect.event_count());
        println!("Got error = {:?}", effect.error);
        assert_ne!(None, effect.error);

        let sink = Actor::new(Box::new(Sink {}));
        let event = Event::new(&maker, Message::Addr(Rc::clone(&sink)));
        let effect = maker.dispatch(event);

        assert_eq!(1, effect.actor_count());
        assert_eq!(1, effect.event_count());
        assert_eq!(None, effect.error);
    }
}

/*
CREATE undefined WITH \(cust, _).[ SEND ? TO cust ]
LET empty_env_beh = \(cust, req).[
  SEND ? TO cust
  SEND #undefined, req TO warning
]
LET env_beh(ident, value, next) = \(cust, req).[
  CASE req OF
  (#lookup, $ident) : [ SEND value TO cust ]
  _ : [ SEND (cust, req) TO next ]
  END
]
LET mutable_env_beh(next) = \(cust, req).[
  CASE req OF
  (#bind, ident, value) : [
    CREATE next' WITH env_beh(ident, value, next)
    BECOME mutable_env_beh(next')
    SEND SELF TO cust
  ]
  _ : [ SEND (cust, req) TO next ]
  END
]

LET race_beh(list) = \(cust, req).[
    CREATE once WITH once_beh(cust)
    send_to_all((once, req), list)
]
LET send_to_all(msg, list) = [
    CASE list OF
    () : []
    (first, rest) : [
        SEND msg TO first
        send_to_all(msg, rest)
    ]
    (last) : [ SEND msg TO last ]
    END
]

LET tag_beh(cust) = \msg.[ SEND (SELF, msg) TO cust ]
LET join_beh(cust, k_first, k_rest) = \msg.[
  CASE msg OF
  ($k_first, first) : [
    BECOME \($k_rest, rest).[ SEND (first, rest) TO cust ]
  ]
  ($k_rest, rest) : [
    BECOME \($k_first, first).[ SEND (first, rest) TO cust ]
  ]
  END
]
LET fork_beh(cust, head, tail) = \(h_req, t_req).[
  CREATE k_head WITH tag_beh(SELF)
  CREATE k_tail WITH tag_beh(SELF)
  SEND (k_head, h_req) TO head
  SEND (k_tail, t_req) TO tail
  BECOME join_beh(cust, k_head, k_tail)
]
*/
