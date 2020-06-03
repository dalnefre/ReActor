use reactor::*;
use std::cell::RefCell;

struct CallCounter {
    count: RefCell<usize>,
}
impl Behavior for CallCounter {
    fn react(&self, _event: Event) -> Effect {
        *self.count.borrow_mut() += 1;
        Effect::new()
    }
}

#[test]
fn call_counter_behavior() {
    let counter = RefCell::new(0);
    let beh = Box::new(CallCounter {
        count: counter,
    });
    let actor = Actor::new(beh);
    //assert_eq!(0, *counter.borrow());

    let event = Event::new(&actor, Message::Empty);
    let effect = actor.dispatch(event);

    //assert_eq!(1, *counter.borrow());
    assert_eq!(0, effect.actor_count());
    assert_eq!(0, effect.event_count());
    assert_eq!(None, effect.error());
}
