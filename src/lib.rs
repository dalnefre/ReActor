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
    pub fn new(behavior: Box<dyn Behavior>) -> Rc<Actor> {
        Rc::new(Actor {
            behavior: RefCell::new(behavior),
        })
    }
    pub fn dispatch(&self, event: Event) -> Effect {
        self.behavior.borrow().react(event)
    }
    pub fn update(&self, behavior: Box<dyn Behavior>) {
        *self.behavior.borrow_mut() = behavior;
    }
}

pub struct Event {
    target: Rc<Actor>,
    message: Message,
}
impl Event {
    pub fn new(target: &Rc<Actor>, message: Message) -> Event {
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
    pub fn actor_count(&self) -> usize {
        self.actors.len()
    }
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
    pub fn error(&self) -> Option<&'static str> {
        self.error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Sink {}
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
        assert_eq!(None, effect.error());
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
        assert_eq!(None, effect.error());

        if let Some(behavior) = effect.state {
            once.update(behavior);
        } else {
            panic!("expected new state!");
        }

        let event = Event::new(&once, Message::Empty);
        let effect = once.dispatch(event);

        assert_eq!(0, effect.actor_count());
        assert_eq!(0, effect.event_count());
        assert_eq!(None, effect.error());
    }
}

/*
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
