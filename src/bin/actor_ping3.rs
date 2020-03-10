extern crate actix;
use actix::prelude::*;

// 测试 Actor 对象
struct MyActor {
}

// 所有的 Actor 必须实现 Actor 特质
impl Actor for MyActor {
    // 每个 Actor 都有一个执行上下文
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        println!("started");
        // 获取到自己的地址
       let addr = ctx.address();
       println!("connected = {}", addr.connected());
       // 设置邮箱容量
        ctx.set_mailbox_capacity(1);
    }
}

// Actor 接收的参数
#[derive(Message)]
#[rtype(result = "Result<actix::Addr<MyActor>, ()>")]
struct WhoAmI;


impl Handler<WhoAmI> for MyActor {
    type Result = Result<actix::Addr<MyActor>, ()>;

    fn handle(&mut self, _: WhoAmI, ctx: &mut Context<Self>) -> Self::Result {
        // ctx.address().try_send(Ping(10));
        Ok(ctx.address())
    }
}

#[actix_rt::main]
async fn main() {
    // 启动一个Actor，返回一个 Addr<MyActor>
    let addr = MyActor {}.start();

    // 发送消息并获取 Future 结果
    let addr2 = addr.recipient();
    // let res = addr2.send(Ping(10)).await;
    let res2 = addr2.send(WhoAmI{}).await;

    // handle() returns tokio handle
    // 返回一个结果
    println!("connected: {}", res2.unwrap().unwrap().connected());

    // Arbiter

    // stop system and exit
    System::current().stop();
}
