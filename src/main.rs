use actix_web::{error, web, http, App, FromRequest, HttpResponse, HttpServer, Responder, Error, HttpRequest, Either, Result};

// curl http://localhost:8088/
// curl http://localhost:8088/app/index.html
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

// curl http://localhost:8088/again
async fn index2() -> impl Responder {
    HttpResponse::Ok().body("Hello world again!")
}

use actix_web::get;

// curl http://localhost:8088/hello
// curl http://localhost:8088/app/hello
// 使用宏解析
#[get("/hello")]
async fn index3() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

struct AppState {
    app_name: String,
}

// curl http://localhost:8088/app_state
async fn app_state(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(&data.app_name)
}

use std::sync::Mutex;

struct AppStateWithCounter {
    counter: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}

// curl http://localhost:8088/counter
async fn counter(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap(); // <- get counter's MutexGuard
    *counter += 1; // <- access counter inside MutexGuard

    format!("Request number: {}", counter) // <- response with count
}

// this function could be located in different module
// curl http://localhost:8088/app3/test
fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/test")
            .route(web::get().to(|| HttpResponse::Ok().body("test")))
            .route(web::head().to(|| HttpResponse::MethodNotAllowed())),
    );
}

// this function could be located in different module
// curl http://localhost:8088/app2
fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/app2")
            .route(web::get().to(|| HttpResponse::Ok().body("app2")))
            .route(web::head().to(|| HttpResponse::MethodNotAllowed())),
    );
}

// curl http://localhost:8088/responder/str
async fn responder_str() -> &'static str {
    "responder_str"
}

// curl http://localhost:8088/responder/string
async fn responder_string() -> String {
    "responder_string".to_owned()
}

// curl http://localhost:8088/responder/impl_responder
async fn responder_impl_responder() -> impl Responder{
    web::Bytes::from_static(b"responder_string")
}

use serde::Serialize;
use futures::future::{ready, Ready};

// 自定义 Response
#[derive(Serialize, Deserialize)]
struct ResponseWrapper<T> {
    code: i32,
    msg: String,
    data: Option<T>,
}

// Responder
impl <T> Responder for ResponseWrapper<T> where T: Serialize {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

// curl http://localhost:8088/responder/custom_responder
async fn responder_custom_responder() -> impl Responder {
    ResponseWrapper { 
        code: 0, 
        msg: "success".to_string(), 
        data: Some("custom_responder".to_string()) }
}

use futures::stream::once;
use futures::future::ok;

// curl http://localhost:8088/responder/stream
async fn responder_stream_responder() -> HttpResponse {
    let body = once(ok::<_, Error>(web::Bytes::from_static(b"test")));

    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(body)
}

type RegisterResult = Either<HttpResponse, Result<&'static str, Error>>;

// curl http://localhost:8088/responder/either
async fn responder_either_responder() -> RegisterResult {
    Either::A(HttpResponse::BadRequest().body("Bad data"))
    // Either::B(Ok("Hello!"))
}

trait CallFnWithTuple<T, R> {
    fn call_with_tuple(&self, param: T) -> R;
}

impl <Func, A, R> CallFnWithTuple<(A,), R> for Func where Func: Fn(A,) -> R {
    fn call_with_tuple(&self, param: (A,)) -> R { 
        (self)(param.0,)
     }

}

impl <Func, A, B, R> CallFnWithTuple<(A, B,), R> for Func where Func: Fn(A, B,) -> R {
    fn call_with_tuple(&self, param: (A, B,)) -> R { 
        (self)(param.0, param.1)
     }

}

fn proxy<T, R>(f: impl CallFnWithTuple<T, R>, p: T) -> R {
    f.call_with_tuple(p)
}

fn test_1(a: i32) -> i32 {
    a + 1
}
fn test_2(a: i32, b: i32) -> i32 {
    a + b
}

use serde::Deserialize;

// 提取器 extractors
#[derive(Deserialize, Debug)]
struct QueryInfo {
    username: String,
}

// curl http://localhost:8088/extractor/multiple/p1/p2?username=xiaoming
async fn extractor_multiple(p: web::Path<(String, String)>, q: web::Query<QueryInfo>) -> String {
    format!("p={:?}, q={:?}", p, q)
}

#[derive(Deserialize, Debug)]
struct PathInfo {
    user_id: u32,
    friend: String,
}

// curl http://localhost:8088/extractor/path/123/friend_name
async fn extractor_path(p: web::Path<PathInfo>) -> String {
    format!("path-param={:?}", p)
}

// curl http://localhost:8088/extractor/manual_path/123/friend_name
async fn extractor_manual_path(req: HttpRequest) -> String {
    let friend: String =
        req.match_info().get("friend").unwrap().parse().unwrap();
    let user_id: i32 = req.match_info().query("user_id").parse().unwrap();
    format!("user_id={}, friend={}", user_id, friend)
}

// curl http://localhost:8088/extractor/query?username=xiaoming
async fn extractor_query(info: web::Query<QueryInfo>) -> String {
    format!("{:?}", info)
}

#[derive(Deserialize, Debug)]
struct JsonInfo {
    username: String,
}

// curl -i -H 'Content-Type: application/json' -d '{"username": "xiaoming"}' -X POST http://localhost:8088/extractor/json 
// curl -i -H 'Content-Type: application/json' -d '{"username": 1}' -X POST http://localhost:8088/extractor/json 
async fn extractor_json(info: web::Json<JsonInfo>) -> String {
    format!("{:?}", info)
}

#[derive(Deserialize, Debug)]
struct FormData {
    username: String,
}

/// 使用serde提取表单数据
/// 仅当内容类型为*x-www-form-urlencoded*时，才会调用此处理程序
/// 并且请求的内容可以反序列化为FormData结构 
// curl -i -H 'Content-Type: application/x-www-form-urlencoded' -d 'username=xiaoming' -X POST http://localhost:8088/extractor/form 
async fn extractor_form(form: web::Form<FormData>) -> String {
    format!("{:?}", form)
}

use failure::Fail;

#[derive(Fail, Debug)]
#[fail(display = "my error")]
struct MyError {
    name: &'static str,
}

impl error::ResponseError for MyError {}

// curl -i http://localhost:8088/error/custom
async fn error_custom() -> Result<&'static str, MyError> {
    Err(MyError { name: "test" })
}

#[derive(Fail, Debug)]
#[allow(dead_code)]
enum MyErrorEnum {
    #[fail(display = "internal error")]
    InternalError,
    #[fail(display = "bad request")]
    BadClientData,
    #[fail(display = "timeout")]
    Timeout,
}

use actix_http::ResponseBuilder;

impl error::ResponseError for MyErrorEnum {
    fn error_response(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code())
            .set_header(http::header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> http::StatusCode {
        match *self {
            MyErrorEnum::InternalError => http::StatusCode::INTERNAL_SERVER_ERROR,
            MyErrorEnum::BadClientData => http::StatusCode::BAD_REQUEST,
            MyErrorEnum::Timeout => http::StatusCode::GATEWAY_TIMEOUT,
        }
    }
}
// curl -i http://localhost:8088/error/enum
async fn error_enum() -> Result<&'static str, MyErrorEnum> {
    Err(MyErrorEnum::BadClientData)
}

// curl -i http://localhost:8088/error/helper
async fn error_helper() -> Result<&'static str> {
    let result: Result<&'static str, MyError> = Err(MyError { name: "test error" });

    Ok(result.map_err(|e| error::ErrorBadRequest(e.name))?)
}


// 1. Middleware initialization, middleware factory gets called with next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.

// 中间件处理分为两个步骤。
// 1. 中间件初始化，使用链中的下一个服务作为参数调用中间件工厂。
// 2. 中间件的call方法被普通请求调用。 
pub struct SayHi;

use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse};
// use futures::Future;
use std::future::Future;

// 中间件工厂需要实现 `Transform` 来自 `actix-service` crate
// Transform 特质相当于如下函数声明（忽略错误）
// type Service = async fn<Req, Res, Err>(req: Req) -> Result<Res, Err>
// async fn new_transform<NextReq, NextRes, NextErr, Req, Res, Err, InitErr>(next_service: Service<NextReq, NextRes, NextErr>) -> Result<Service<Req, Res, Err>, InitErr>

// `S` - 下一个 service 的 类型
// `B` - response body 的类型
impl<S, B> Transform<S> for SayHi
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    // 当前 Service 的请求
    type Request = ServiceRequest;
    // 当前 Service 的响应
    type Response = ServiceResponse<B>;
    // 当前 Service 的错误类型
    type Error = Error;
    // 创建 当前 Service 时可能出现的错误
    type InitError = ();
    // 当前 Transform 的类型
    type Transform = SayHiMiddleware<S>;
    // 异步的包装
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    // 工厂方法
    fn new_transform(&self, service: S) -> Self::Future {
        ok(SayHiMiddleware { service })
    }
}

pub struct SayHiMiddleware<S> {
    service: S,
}

// Service 中间件/服务，基本等价于
// 一个异步处理函数：async fn<Req, Res, Err>(req: Req) -> Result<Res, Err>
// 一个异步就绪判断：async fn poll_ready<Err>() -> Result<(), Err>
// 服务是 Actix-web 的核心抽象
impl<S, B> Service for SayHiMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    // 一个异步函数
    // 确定当前 Service 是否可以处理请求，不可以处理时，返回 Pending
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    // 处理函数，不应该调用 poll_ready。允许
    // actix可能在不调用poll_ready的情况下调用call，因此实现上必须要考虑这一点
    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        println!("Hi from start. You requested: {}", req.path());

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            println!("Hi from response");
            Ok(res)
        })
    }
}

use futures::future::FutureExt;

// async fn my_wrapper<B, S> (req: ServiceRequest, srv: &mut S) -> Result<ServiceResponse<B>, Error> 
// where
//     S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
// {
//     println!("Hi from start. You requested: {}", req.path());
//     srv.call(req).await
//     // let res = srv.call(req).await;
//     // println!("Hi from response");
//     // res
// }

use actix_web::middleware::Logger;
use env_logger;

use actix_session::{CookieSession /*, Session*/};

use actix::{Actor, StreamHandler};
use actix_web_actors::ws;

/// 定义Http Actor
struct MyWs;

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

/// 一个 ws::Message 消息的 处理器
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

// ws://localhost:8088/ws/echo
/*
curl --include \
     --no-buffer \
     --header "Connection: Upgrade" \
     --header "Upgrade: websocket" \
     --header "Host: echo.websocket.org" \
     --header "Origin: https://echo.websocket.org" \
     --header "Sec-WebSocket-Key: NVwjmQUcWCenfWu98asDmg==" \
     --header "Sec-WebSocket-Version: 13" \
     http://localhost:8088/ws/echo
*/
async fn ws_echo(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(MyWs {}, &req, stream);
    println!("{:?}", resp);
    resp
}

use listenfd::ListenFd;
use actix_learn::*;
use diesel::prelude::*;

// curl http://localhost:8088/block/user/create
async fn create_user(pool: web::Data<PoolConnection>) -> String {
    let conn = pool.get().expect("couldn't get db connection from pool");

    let r = web::block(move ||  {
        use schema::users;
        let user = model::UserForInsert {
            name: "name".to_string(),
            hair_color: Some("blank".to_string()),
        };
        diesel::insert_into(users::table)
            .values(&user)
            .execute(&conn)
    }).await;
    if let Err(e) = r {
        String::from(format!("{:?}", e))
    } else {
        String::from("create_success")
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    println!("{}", proxy(test_1, (1,)));
    println!("{}", proxy(test_2, (1,2)));
    println!("{}", test_2.call_with_tuple((1,2)));

    let c = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let mut listenfd = ListenFd::from_env();

    let pool = new_connection_pool();

    let mut server = HttpServer::new(move || {

        App::new()
            .wrap(actix_web::middleware::NormalizePath)
            .wrap(SayHi{})
            .wrap(Logger::default())
            .wrap(CookieSession::signed(&[0; 32]) // <- create cookie based session middleware
                    .secure(false))
            .wrap(actix_web::middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap_fn(|req, srv| {
                println!("Hi from start. You requested: {}", req.path());
                srv.call(req).map(|res| {
                    println!("Hi from response");
                    res
                })
            })
            .wrap_fn(|req, srv| {
                println!("Hi from start. You requested: {}", req.path());
                let fut = srv.call(req);
                async {
                    let res = fut.await;
                    println!("Hi from response");
                    res
                }
            })
            // .wrap_fn(my_wrapper) // 这种写法不允许，因为 wrap_fn 声明声明周期存在问题
            // 由于 HttpServer::new 接收的是 App 工厂函数
            // 所以不同线程的 data 不是同一个实例，所以不是进程级别共享数据，而是线程级别的共享数据
            // 因此只能用于访问只读数据，如全局配置等
            .data(AppState {
                app_name: String::from("Actix-web"),
            })
            .data(pool.clone())
            .app_data(c.clone())
            .route("/", web::get().to(index))
            .route("/again/", web::get().to(index2))
            .service(index3)
            .service(
                web::scope("/app")
                    .route("/index.html", web::get().to(index))
                    .service(index3)
            )
            .route("/app_state", web::get().to(app_state))
            .route("/counter", web::get().to(counter))
            .configure(config)
            .service(
                web::scope("/app3")
                    .configure(scoped_config)
            )
            .service(
                web::scope("/responder")
                    .route("/str", web::get().to(responder_str))
                    .route("/string", web::get().to(responder_string))
                    .route("/impl_responder", web::get().to(responder_impl_responder))
                    .route("/custom_responder", web::get().to(responder_custom_responder))
                    .route("/stream", web::get().to(responder_stream_responder))
                    .route("/either", web::get().to(responder_either_responder))
            )
            // 配置 Json Extractor
            .app_data(web::Json::<JsonInfo>::configure(|cfg| {
                    cfg.limit(4096).error_handler(|err, _req| {
                        error::InternalError::from_response(
                            err,
                            HttpResponse::Conflict().finish(),
                        )
                        .into()
                    })
                }))
            .service(
                web::scope("/extractor")
                    .route("/multiple/{p1}/{p2}", web::get().to(extractor_multiple))
                    .route("/path/{user_id}/{friend}", web::get().to(extractor_path))
                    .route("/manual_path/{user_id}/{friend}", web::get().to(extractor_manual_path))
                    .route("/query", web::get().to(extractor_query))
                    .route("/json", web::post().to(extractor_json))
                    .route("/form", web::post().to(extractor_form))
            )
            .service(
                web::scope("/error")
                    .route("/custom", web::get().to(error_custom))
                    .route("/enum", web::get().to(error_enum))
                    .route("/helper", web::get().to(error_helper))
            )
            .service(
                web::scope("/ws")
                    .route("/echo", web::get().to(ws_echo))
            )
            .service(
                web::scope("/block")
                    .route("/user/create", web::get().to(create_user))
            )
    });
    // .bind("127.0.0.1:8088")?
    // .run()
    // .await

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:8088")?
    };

    server.run().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    // 单元测试
    #[actix_rt::test]
    async fn test_index_ok(){
        // 构建测试的TestRequest
        let req = test::TestRequest::get()
            .header("content-type", "text/plain")
            // .get()
            .to_http_request();

        println!("{:?}", req);

        // 执行测试函数
        let resp = index().await.respond_to(&req).await.ok().unwrap();

        // 断言
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    // 继承测试
    #[actix_rt::test]
    async fn test_index_get() {
        let mut app = test::init_service(App::new().route("/", web::get().to(index))).await;
        let req = test::TestRequest::with_header("content-type", "text/plain").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_index_post() {
        let mut app = test::init_service(App::new().route("/", web::get().to(index))).await;
        let req = test::TestRequest::post().uri("/").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_json_response() {
        let mut app = test::init_service(
            App::new()
                .data(AppState { app_name: "app_name".to_string() })
                .route("/responder/custom_responder", web::get().to(responder_custom_responder)),
        ).await;
        let req = test::TestRequest::get().uri("/responder/custom_responder").to_request();
        let resp: ResponseWrapper<String> = test::read_response_json(&mut app, req).await;

        assert_eq!(resp.code, 0);
    }
}
