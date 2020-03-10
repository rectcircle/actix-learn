extern crate actix;
use actix::prelude::*;

// 测试 Actor 对象
struct MyActor {
    count: usize,
}

// 所有的 Actor 必须实现 Actor 特质
impl Actor for MyActor {
    // 每个 Actor 都有一个执行上下文
    type Context = Context<Self>;
}

// Actor 接收的参数
struct Ping(usize);

// Actor 接收的参数必须实现 Message 对象
impl Message for Ping {
    // 定义 该参数的 返回值类型
    type Result = usize;
}

// Actor 的具体处理函数
impl Handler<Ping> for MyActor {
    // 返回值
    type Result = usize;

    fn handle(&mut self, msg: Ping, _ctx: &mut Context<Self>) -> Self::Result {
        self.count += msg.0;

        self.count
    }
}

#[actix_rt::main]
async fn main() {
    // 启动一个Actor，返回一个 Addr<MyActor>
    let addr = MyActor { count: 10 }.start();

    // 发送消息并获取 Future 结果
    let res = addr.send(Ping(10)).await;

    // handle() returns tokio handle
    // 返回一个结果
    println!("RESULT: {}", res.unwrap() == 20);

    // stop system and exit
    System::current().stop();
}
