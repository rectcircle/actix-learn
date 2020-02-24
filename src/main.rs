use actix_web::{error, web, App, FromRequest, HttpResponse, HttpServer, Responder, Error, HttpRequest, Either};

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
#[derive(Serialize)]
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

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    println!("{}", proxy(test_1, (1,)));
    println!("{}", proxy(test_2, (1,2)));
    println!("{}", test_2.call_with_tuple((1,2)));

    let c = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });
    HttpServer::new(move || {
        App::new()
            // 由于 HttpServer::new 接收的是 App 工厂函数
            // 所以不同线程的 data 不是同一个实例，所以不是进程级别共享数据，而是线程级别的共享数据
            // 因此只能用于访问只读数据，如全局配置等
            .data(AppState {
                app_name: String::from("Actix-web"),
            })
            .app_data(c.clone())
            .route("/", web::get().to(index))
            .route("/again", web::get().to(index2))
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
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}