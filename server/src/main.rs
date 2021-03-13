use crate::lib::config::Config;
use crate::lib::db::log::create::create as create_log_db;
use crate::lib::db::user::create::create as create_user_db;
use crate::lib::default::default;
use actix_web::{dev::Service, middleware::errhandlers::ErrorHandlers};
use actix_web::{http, web, App, HttpServer};
use lib::file::default::{bulma, bulma_js, file_svg, folder_svg, indexhtml, wasm, wasmloader};
use logger::info;
use crate::http_handler::{default::{default_404, default_api_handler, p500}, users::{create_user, login_user}, files::{cli, save_file, deletef}};

mod lib;
mod http_handler;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config: Config = default();
    let mut database = config.get_db_config().to_datapool().await;
    create_log_db(&mut database).await;
    create_user_db(&mut database).await;
    let server_ip: &str = &config.get_server();
    lib::db::user::get::get_users(&mut database).await;
    if cfg!(feature = "log") {
        info(format!(
            "Server listening on {}:{}",
            config.server_ip, config.server_port
        ));
    } else {
        println!("Server running");
    }

    HttpServer::new(move || {
        App::new()
            .default_service(web::to(indexhtml))
            .service(
                web::scope("/pkg/")
                    .service(web::resource("package.js").to(wasmloader))
                    .service(web::resource("package_bg.wasm").to(wasm))
                    .service(web::resource("bulma/bulma.min.css").to(bulma))
                    .service(web::resource("bulma/bulma.js").to(bulma_js))
                    .service(web::resource("obj/file.svg").to(file_svg))
                    .service(web::resource("obj/folder.svg").to(folder_svg))
                    .default_service(web::to(default_404)),
            )
            .service(
                web::scope("/api")
                    .default_service(web::to(default_api_handler))
                    .service(cli)
                    .service(create_user)
                    .service(save_file)
                    .service(deletef)
                    .service(login_user),
            )
            .data(database.clone())
            .wrap_fn(|req, srv| {
                let fut = srv.call(req);
                async move {
                    let res = fut.await.unwrap();
                    let e = res.request();
                    if cfg!(feature = "log") {
                        info(format!(
                            "[{}] {}:{} {}",
                            time::PrimitiveDateTime::from(std::time::SystemTime::now())
                                .format("%F %T"),
                            &e.method(),
                            &res.status(),
                            &e.path()
                        ));
                    }
                    Ok(res)
                }
            })
            .wrap(ErrorHandlers::new().handler(http::StatusCode::INTERNAL_SERVER_ERROR, p500))
    })
    .bind(server_ip)?
    .run()
    .await
}
