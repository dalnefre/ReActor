use std::collections::VecDeque;
use std::rc::Rc;

pub trait ReActor {
    fn react(&self, event: Event) -> Effect;
}

pub struct Event {
    target: Rc<dyn ReActor>,
    message: Message,
}
impl Event {
	pub fn new(target: &Rc<dyn ReActor>, message: Message) -> Event {
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
    actors: Vec<Rc<dyn ReActor>>,
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
    pub fn send(&mut self, target: &Rc<dyn ReActor>, message: Message) {
        let event = Event::new(target, message);
        self.events.push_back(event);
    }
    pub fn update(&mut self) {
    	panic!("behavior replacement not implemented");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

	struct Sink {}
	impl Sink {
		fn new() -> Rc<dyn ReActor> {
			Rc::new(Sink {})
		}
	}
	impl ReActor for Sink {
	    fn react(&self, _event: Event) -> Effect {
	    	Effect::new()
	    }
	}

    #[test]
    fn sink_behavior() {
    	let sink = Sink::new();
        let event = Event::new(&sink, Message::Empty);
    	let effect = sink.react(event);

    	assert_eq!(0, effect.actors.len());
    	assert!(effect.actors.is_empty());
    	assert_eq!(0, effect.events.len());
    	assert!(effect.events.is_empty());
    	assert_eq!(None, effect.error);
    }

	struct Once {
		cust: Rc<dyn ReActor>,
	}
	impl Once {
		fn new(cust: &Rc<dyn ReActor>) -> Rc<dyn ReActor> {
			Rc::new(Once {
				cust: Rc::clone(cust),
			})
		}
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
    #[ignore]
    fn once_behavior() {
    	let sink = Sink::new();
    	let once = Once::new(&sink);
        let event = Event::new(&once, Message::Empty);
    	let effect = once.react(event);

    	assert_eq!(0, effect.actors.len());
    	assert!(effect.actors.is_empty());
    	assert_eq!(1, effect.events.len());
    	assert!(!effect.events.is_empty());
    	assert_eq!(None, effect.error);
    }
}
