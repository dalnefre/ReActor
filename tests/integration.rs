extern crate alloc;

use reactor::*;
//use reactor::Error;
use alloc::boxed::Box;
use alloc::rc::Rc;
//use alloc::collections::BTreeMap;

#[test]
#[ignore]
fn check_struct_sizes() {
    use core::mem;

    println!("sizeof<usize> = {:?}", mem::size_of::<usize>());
    println!("sizeof<Actor> = {:?}", mem::size_of::<Actor>());
    println!("sizeof<Message> = {:?}", mem::size_of::<Message>());
    println!("sizeof<Event> = {:?}", mem::size_of::<Event>());
    println!("sizeof<Effect> = {:?}", mem::size_of::<Effect>());
    println!("sizeof<Error> = {:?}", mem::size_of::<Error>());
    println!("sizeof<Config> = {:?}", mem::size_of::<Config>());
    println!("sizeof<Rc<Actor>> = {:?}", mem::size_of::<Rc<Actor>>());
    println!("sizeof<Box<Message>> = {:?}", mem::size_of::<Box<Message>>());
    println!("sizeof<Box<dyn Behavior>> = {:?}", mem::size_of::<Box<dyn Behavior>>());
    println!("sizeof<Option<Effect>> = {:?}", mem::size_of::<Option<Effect>>());
    println!("sizeof<Result<Effect,Error>> = {:?}", mem::size_of::<Result<Effect,Error>>());
    assert!(false);  // force failure!
}

#[test]
fn sink_ignores_all_messages() {
    struct Boot;
    impl Behavior for Boot {
        fn react(&self, _event: Event) -> Result<Effect, Error> {
            let mut effect = Effect::new();

            let sink = effect.create(Box::new(idiom::Sink));
            effect.send(&sink, Message::Empty);
            effect.send(&sink, Message::Empty);

            Ok(effect)
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
        fn react(&self, _event: Event) -> Result<Effect, Error> {
            let mut effect = Effect::new();

            let sink = effect.create(idiom::Sink::new());
            let forward = effect.create(idiom::Forward::new(&sink));
            effect.send(&forward, Message::Empty);
            effect.send(&forward, Message::Empty);
            effect.send(&sink, Message::Empty);

            Ok(effect)
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
        fn react(&self, _event: Event) -> Result<Effect, Error> {
            let mut effect = Effect::new();

            let cust = effect.create(Box::new(MockCust));
            let label = effect.create(idiom::Label::new(&cust, Message::Sym("Hello")));
            effect.send(&label, Message::Sym("World"));

            Ok(effect)
        }
    }
    struct MockCust;
    impl Behavior for MockCust {
        fn react(&self, event: Event) -> Result<Effect, Error> {
            println!("MockCust: message = {:?}", event.message);
            unsafe {
                MOCK_MESSAGE = event.message;
            }
            Ok(Effect::new())
        }
    }

    let mut config = Config::new();
    let count = config.boot(Box::new(Boot));
    assert_eq!(1, count);

    let count = config.dispatch(2);
    assert_eq!(0, count);
    let expect = Message::Pair(
        Box::new(Message::Sym("Hello")),
        Box::new(Message::Sym("World")),
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
        fn react(&self, _event: Event) -> Result<Effect, Error> {
            let mut effect = Effect::new();

            let cust = effect.create(Box::new(MockCust));
            let tag = effect.create(idiom::Tag::new(&cust));
            effect.send(&tag, Message::Sym("It's Me!"));

            Ok(effect)
        }
    }
    struct MockCust;
    impl Behavior for MockCust {
        fn react(&self, event: Event) -> Result<Effect, Error> {
            println!("MockCust: message = {:?}", event.message);
            unsafe {
                MOCK_MESSAGE = event.message;
            }
            Ok(Effect::new())
        }
    }

    let mut config = Config::new();
    let count = config.boot(Box::new(Boot));
    assert_eq!(1, count);

    let count = config.dispatch(2);
    assert_eq!(0, count);
    unsafe {
        match &MOCK_MESSAGE {
/*
            Message::Pair(box Message::Addr(_), box Message::Sym("It's Me!")) => {  // FIXME: experimental -- requires nightly
*/
            Message::Pair(a, b) => {
                match **a {
                    Message::Addr(_) => {
                        assert_eq!(Message::Sym("It's Me!"), **b);
                    },
                    _ => panic!("Expected Addr(_), Got {:?}", a),
                }
            },
            m => panic!("Unexpected {:?}", m)
        }
    }
}

/*
#[test]
fn can_send_struct_and_num() {
//  example: (7/5) / (2/3) = (7/5) * (3/2) = (21/10) = (21 * 10 ^ -1)
    struct Boot;
    impl Behavior for Boot {
        fn react(&self, _event: Event) -> Result<Effect, Error> {
            let mut effect = Effect::new();

            let calc = effect.create(Box::new(RationalDiv));
            let check = effect.create(AssertAnswer::new(Message::Num(21, 10, -1)));
            let m = [
                Message::Addr(Rc::clone(&check)),
                Message::Num(7, 5, -1),
                Message::Num(2, 3, -1)
            ];
            effect.send(&calc, Message::List(&m));  // <--- FIXME! "borrowed value does not live long enough"

            Ok(effect)
        }
    }

    fn rat_div(n: &Message, d: &Message) -> Message {
        match (n, d) {
            (Message::Num(ni, nb, -1), Message::Num(di, db, -1)) => {
                Message::Num(ni * db, nb * di, -1)
            },
            _ => Message::Empty,  // default/failure value
        }
    }

    struct RationalDiv;
    impl Behavior for RationalDiv {
        fn react(&self, event: Event) -> Result<Effect, Error> {
            let mut effect = Effect::new();

            match event.message {
                Message::List(m) => {
                    println!("RationalDiv: m = {:?}", m);
                    match &m[0] {
                        Message::Addr(cust) => {
                            let answer = rat_div(&m[1], &m[2]);
                            effect.send(&cust, answer);
                            Ok(effect)
                        },
                        _ => Err("[0] must be Addr(_)"),
                    }
                },
                _ => Err("Message not understood"),  // FIXME: need mechanism for runtime to report Errors...
            }

        }
    }

    struct AssertAnswer {
        expect: Message,
    };
    impl Behavior for AssertAnswer {
        fn react(&self, event: Event) -> Result<Effect, Error> {
            println!("AssertAnswer: message = {:?}", event.message);
            assert_eq!(self.expect, event.message);
            Ok(Effect::new())
        }
    }
    impl AssertAnswer {
        pub fn new(msg: Message) -> Box<dyn Behavior> {
            Box::new(AssertAnswer {
                expect: msg,
            })
        }
    }

    let mut config = Config::new();
    let count = config.boot(Box::new(Boot));
    assert_eq!(1, count);

    let count = config.dispatch(2);
    assert_eq!(0, count);
}
*/

/*
LET sink_beh = \_.[]
CREATE sink WITH sink_beh

LET forward_beh = \cust.\msg.[
    SEND msg TO cust
]

LET label_beh(cust, label) = \msg.[ SEND (label, msg) TO cust ]

LET once_beh(cust) = \msg.[
    SEND msg TO cust
    BECOME sink_beh
]

LET counter_beh(count) = \n.[
    BECOME counter_beh(add(count, n))
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

*/
