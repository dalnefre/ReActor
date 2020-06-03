use std::collections::VecDeque;
use std::rc::Rc;

pub trait ReActor {  // FIXME: rename this to Behavior
    fn react(&self, event: Event) -> Effect;
}

pub struct Actor {
    behavior: Box<dyn ReActor>,
}
impl Actor {
    pub fn new(behavior: Box<dyn ReActor>) -> Rc<Actor> {
        Rc::new(Actor { behavior })
    }
    pub fn dispatch(&self, event: Event) -> Effect {
        self.behavior.react(event)
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
    error: Option<&'static str>,
}
impl Effect {
    pub fn new() -> Effect {
        Effect {
            actors: Vec::new(),
            events: VecDeque::new(),
            error: None,
        }
    }
    pub fn send(&mut self, target: &Rc<Actor>, message: Message) {
        let event = Event::new(target, message);
        self.events.push_back(event);
    }
    pub fn update(&mut self) {
        //panic!("behavior replacement not implemented");
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
    impl ReActor for Sink {
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
    impl ReActor for Once {
        fn react(&self, event: Event) -> Effect {
            let mut effect = Effect::new();
            effect.send(&self.cust, event.message);
            effect.update();
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
    }
}
