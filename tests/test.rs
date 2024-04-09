use log::debug;
use supertrees::{RestartPolicy, Restartable, Worker};
use test_log::test;
#[derive(Debug)]
struct W {
    num: i32,
}

impl W {
    fn new(num: i32) -> Self {
        Self { num }
    }
}

impl Worker for W {
    fn init(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>> {
        let num = self.num;
        Box::pin(async move {
            println!("hi, I'm woooorker num={num} :)");
        })
    }
}

impl Restartable for W {
    fn restart_policy(&self) -> RestartPolicy {
        RestartPolicy::Once
    }
}

#[test]
fn test_supertree() {
    use supertrees::Supertree;
    let root = Supertree::new()
        .add_worker(W::new(1))
        .add_worker(W::new(2))
        .add_worker(W::new(3))
        .add_supervisor(|s| {
            s.add_worker(W::new(4))
                .add_worker(W::new(5))
                .add_worker(W::new(6))
                .add_supervisor(|s| {
                    s.add_worker(W::new(7))
                        .add_worker(W::new(8))
                        .add_worker(W::new(9))
                })
        });
    debug!("supertree={root:#?}");
    root.start();
    println!("done");
}
