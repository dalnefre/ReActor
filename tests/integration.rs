use reactor::*;
use std::rc::Rc;
use std::cell::RefCell;

struct CallCounter {
    count: RefCell<usize>,
}
impl CallCounter {
    fn new() -> Rc<dyn ReActor> {
        Rc::new(CallCounter {
            count: RefCell::new(0),
        })
    }
}
impl ReActor for CallCounter {
    fn react(&self, _event: Event) -> Effect {
        *self.count.borrow_mut() += 1;
        Effect::new()
    }
}

#[test]
fn call_counter_behavior() {
    let actor = CallCounter::new();
    //assert_eq!(0, actor.count);  <-- no visibility to `count` through actor reference!

    let event = Event::new(&actor, Message::Empty);
    let effect = actor.react(event);

    //assert_eq!(1, actor.count);  <-- no visibility to `count` through actor reference!
}
