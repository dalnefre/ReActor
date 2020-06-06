extern crate alloc;

use reactor::*;
use alloc::boxed::Box;
use alloc::rc::Rc;

#[test]
#[ignore]
fn check_struct_sizes() {
    use core::mem;

    println!("sizeof<usize> = {:?}", mem::size_of::<usize>());
    println!("sizeof<Actor> = {:?}", mem::size_of::<Actor>());
    println!("sizeof<Message> = {:?}", mem::size_of::<Message>());
    println!("sizeof<Event> = {:?}", mem::size_of::<Event>());
    println!("sizeof<Effect> = {:?}", mem::size_of::<Effect>());
    println!("sizeof<Config> = {:?}", mem::size_of::<Config>());
    println!("sizeof<Rc<Actor>> = {:?}", mem::size_of::<Rc<Actor>>());
    println!("sizeof<Box<Message>> = {:?}", mem::size_of::<Box<Message>>());
    println!("sizeof<Box<dyn Behavior>> = {:?}", mem::size_of::<Box<dyn Behavior>>());
    println!("sizeof<Option<Effect>> = {:?}", mem::size_of::<Option<Effect>>());
    assert!(false);  // force failure!
}

#[test]
fn sink_ignores_all_messages() {
    struct Boot;
    impl Behavior for Boot {
        fn react(&self, _event: Event) -> Effect {
            let mut effect = Effect::new();

            let sink = effect.create(Box::new(idiom::Sink));
            effect.send(&sink, Message::Empty);
            effect.send(&sink, Message::Empty);

            effect
        }
    }

    let mut config = Config::new();
    let count = config.boot(Box::new(Boot));
    assert_eq!(2, count);

    let count = config.dispatch(2);
    assert_eq!(0, count);
}

#[test]
fn forward_proxies_all_messages() {
    struct Boot;
    impl Behavior for Boot {
        fn react(&self, _event: Event) -> Effect {
            let mut effect = Effect::new();

            let sink = effect.create(Box::new(idiom::Sink));
            let forward = effect.create(Box::new(idiom::Forward {
                subject: Rc::clone(&sink),
            }));
            effect.send(&forward, Message::Empty);
            effect.send(&forward, Message::Empty);
            effect.send(&sink, Message::Empty);

            effect
        }
    }

    let mut config = Config::new();
    let count = config.boot(Box::new(Boot));
    assert_eq!(3, count);

    let count = config.dispatch(3);
    assert_eq!(2, count);
}

#[test]
fn label_decorates_message() {
    static mut MOCK_MESSAGE: Message = Message::Empty;

    struct Boot;
    impl Behavior for Boot {
        fn react(&self, _event: Event) -> Effect {
            let mut effect = Effect::new();

            let cust = effect.create(Box::new(MockCust));
            let label = effect.create(Box::new(idiom::Label {
                cust: Rc::clone(&cust),
                label: Message::Str("Hello"),
            }));
            effect.send(&label, Message::Str("World"));

            effect
        }
    }
    struct MockCust;
    impl Behavior for MockCust {
        fn react(&self, event: Event) -> Effect {
            println!("MockCust: message = {:?}", event.message);
            unsafe {
                MOCK_MESSAGE = event.message;
            }
            Effect::new()
        }
    }

    let mut config = Config::new();
    let count = config.boot(Box::new(Boot));
    assert_eq!(1, count);

    let count = config.dispatch(2);
    assert_eq!(0, count);
    let expect = Message::Pair(
        Box::new(Message::Str("Hello")),
        Box::new(Message::Str("World")),
    );
    unsafe {
        assert_eq!(expect, MOCK_MESSAGE);
    }
}

#[test]
fn tag_decorates_with_self() {
    static mut MOCK_MESSAGE: Message = Message::Empty;

    struct Boot;
    impl Behavior for Boot {
        fn react(&self, _event: Event) -> Effect {
            let mut effect = Effect::new();

            let cust = effect.create(Box::new(MockCust));
            let tag = effect.create(Box::new(idiom::Tag {
                cust: Rc::clone(&cust),
            }));
            effect.send(&tag, Message::Str("It's Me!"));

            effect
        }
    }
    struct MockCust;
    impl Behavior for MockCust {
        fn react(&self, event: Event) -> Effect {
            println!("MockCust: message = {:?}", event.message);
            unsafe {
                MOCK_MESSAGE = event.message;
            }
            Effect::new()
        }
    }

    let mut config = Config::new();
    let count = config.boot(Box::new(Boot));
    assert_eq!(1, count);

    let count = config.dispatch(2);
    assert_eq!(0, count);
    unsafe {
        match &MOCK_MESSAGE {
            Message::Pair(a, b) => {
                match **a {
                    Message::Addr(_) => {
                        assert_eq!(Message::Str("It's Me!"), **b);
                    },
                    _ => panic!("Expected Addr(_), Got {:?}", a),
                }
            },
            m => panic!("Unexpected {:?}", m)
        }
    }
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
