extern crate actix;
extern crate futures;
use actix::prelude::*;
use futures::FutureExt;
use futures::TryFutureExt;

struct SumActor {}

impl Actor for SumActor {
    type Context = Context<Self>;
}

struct Value(usize, usize);

impl Message for Value {
    type Result = usize;
}

impl Handler<Value> for SumActor {
    type Result = usize;

    fn handle(&mut self, msg: Value, _ctx: &mut Context<Self>) -> Self::Result {
        msg.0 + msg.1
    }
}

struct DisplayActor {}

impl Actor for DisplayActor {
    type Context = Context<Self>;
}

struct Display(usize);

impl Message for Display {
    type Result = ();
}

impl Handler<Display> for DisplayActor {
    type Result = ();

    fn handle(&mut self, msg: Display, _ctx: &mut Context<Self>) -> Self::Result {
        println!("Got {:?}", msg.0);
    }
}

struct MySyncActor;

impl Actor for MySyncActor {
    type Context = SyncContext<Self>;
}

fn main() {
    let system = System::new("single-arbiter-example");

    // 创建 Addr
    let sum_addr = SumActor {}.start();
    let dis_addr = DisplayActor {}.start();

    // 定义一个执行流的Future
    // 起初发送 `Value(6, 7)` 给 `SumActor`
    // `Addr::send` 返回 `Request` 类型，该类型实现了 `Future`
    // Future::Output = Result<usize, MailboxError>
    let execution = sum_addr
        .send(Value(6, 7))
        // `.map_err` 转换 `Future<usize, MailboxError>` 为 `Future<usize, ()>`
        //   如果有错误将打印错误信息
        // 实现来自于 use futures::TryFutureExt;
        .map_err(|e| {
            eprintln!("Encountered mailbox error: {:?}", e);
        })
        // 假设发送成功，并成功返回，and_then将得到执行，其中参数为上一个Future的Result<T, E> 的 T
        // 实现来自于 use futures::TryFutureExt;
        .and_then(move |res| {
            // `res` 是 `SumActor` 参数为 `Value(6, 7)` 的返回值，类型为 `usize`

            // res 发送给 DisplayActor 展示
            dis_addr.send(Display(res)).map_err(|_| ())
        })
        .map(move |_| {
            // 当 DisplayActor 返回后停止，将关闭所有 Actor
            System::current().stop();
        });

    // 提交 Future 到 Arbiter/event 循环中
    Arbiter::spawn(execution);

    system.run().unwrap();

    // 同步执行器
    // 创建运行在指定线程中的Actor
    #[allow(unused)]
    let addr = SyncArbiter::start(2, || MySyncActor);
}