use reactor::*;
use std::cell::RefCell;

struct CallCounter {
    count: RefCell<usize>,
}
impl Behavior for CallCounter {
    fn react(&self, _event: Event) -> Effect {
        *self.count.borrow_mut() += 1;
        println!("CallCounter.count = {}", *self.count.borrow());
        Effect::new()
    }
}

#[test]
fn call_counter_behavior() {
    struct Boot;
    impl Behavior for Boot {
        fn react(&self, _event: Event) -> Effect {
            let mut effect = Effect::new();
            println!("call_counter_behavior::Boot");

            let counter = RefCell::new(0);
            let beh = Box::new(CallCounter {
                count: counter,
            });
            let actor = effect.create(beh);
            effect.send(&actor, Message::Empty);

            effect
        }
    }

    let mut config = Config::new();
    let count = config.boot(Box::new(Boot));
    assert_eq!(1, count);

    let count = config.dispatch(1);
    assert_eq!(0, count);
}
