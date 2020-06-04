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

/*
CREATE empty_env WITH \(cust, _).[ SEND ? TO cust ]
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
