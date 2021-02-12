use crate::lib::{
    file::{get_file_as_byte_vec, get_file_preview},
    log::error,
};
use actix_http::Response;
use actix_utils::mpsc;
use actix_web::dev::BodyEncoding;
use actix_web::http::ContentEncoding;
use actix_web::web;
use async_std::fs as afs;
use std::fs::File;
use std::io::Error;
use std::path::PathBuf;
use zip::write::FileOptions;
use zip::CompressionMethod;
use zip_extensions::zip_create_from_directory_with_options;
pub enum ArchiveType {
    Targz,
    Zip
}

pub async fn download(
    path: String,
    atype: ArchiveType,
) -> Result<actix_web::HttpResponse, std::io::Error> {
    match async_std::fs::metadata(path.clone()).await {
        Ok(e) => {
            if e.is_file() {
                get_file_preview(path.clone()).await
            } else if e.is_dir() {
                match atype {
                    ArchiveType::Targz => get_tar(path.clone()).await,
                    ArchiveType::Zip => get_zip(path.clone()).await,
                }
            } else {
                Ok(Response::Ok().body("No file"))
            }
        }
        Err(_) => Ok(Response::Ok().body("Error")),
    }
}
pub async fn get_zip(path: String) -> std::io::Result<Response> {
    let (tx, rx_body) = mpsc::channel();
    let _ = tx.send(Ok::<_, Error>(actix_web::web::Bytes::from(
        get_file_as_byte_vec(path.clone(), &"zip").await,
    )));
    println!("{}", path.clone());
    Ok(Response::Ok()
        .header("Access-Control-Allow-Origin", "*")
        .header("charset", "utf-8")
        .header(
            "Content-Disposition",
            format!(
                "\"attachment\";filename=\"{}.zip\"",
                path.clone().split('/').last().unwrap_or("default_name")
            ),
        )
        .content_type("application/zip")
        .encoding(ContentEncoding::Gzip)
        .streaming(rx_body))
}

pub async fn get_tar(path: String) -> std::io::Result<Response> {
    let (tx, rx_body) = mpsc::channel();
    let _ = tx.send(Ok::<_, Error>(actix_web::web::Bytes::from(
        get_file_as_byte_vec(path.clone(), &"tar").await,
    )));
    Ok(Response::Ok()
        .header("Access-Control-Allow-Origin", "*")
        .header("charset", "utf-8")
        .header(
            "Content-Disposition",
            format!(
                "\"attachment\";filename=\"{}.tar.gz\"",
                path.clone().split('/').last().unwrap_or("default_name")
            ),
        )
        .content_type("application/x-tar")
        .encoding(ContentEncoding::Gzip)
        .streaming(rx_body))
}

async fn async_zip_archive(name: String, dir: String) -> afs::File {
    let file_name = format!("./temp/{}.zip", name);
    File::create(file_name.clone()).unwrap();
    if cfg!(debug_assertions) {
        println!("filename => {}", dir);
    }
    match web::block(|| {
        zip_create_from_directory_with_options(
            &PathBuf::from(file_name),
            &PathBuf::from(dir),
            FileOptions::default().compression_method(CompressionMethod::Bzip2),
        )
    })
    .await
    {
        Ok(_) => {}
        Err(e) => match e {
            actix_http::error::BlockingError::Error(ziperror) => match ziperror {
                zip::result::ZipError::Io(_) => error("I/O Error"),
                zip::result::ZipError::InvalidArchive(_) => error("Invalid Archive"),
                zip::result::ZipError::UnsupportedArchive(_) => error("Unsupported Archive"),
                zip::result::ZipError::FileNotFound => error("File not found"),
            },
            actix_http::error::BlockingError::Canceled => {}
        },
    };

    afs::File::open(format!("./temp/{}.zip", name))
        .await
        .expect("Error")
}

async fn async_tar_archive(name: String, dir: String) -> afs::File {
    let file_name = format!("./temp/{}.tar.gz", name);
    if cfg!(debug_assertions) {
        println!("{} dir : {}", file_name, dir);
    }
    File::create(&file_name).expect("Error");
    let file = afs::File::open(&file_name);
    tar::Builder::new(File::open(&file_name).expect("no file found"))
        .append_dir_all(file_name.as_str(), dir.clone().as_str())
        .expect("Error");
    file.await.expect("Error")
}

pub async fn random_archive(extention: String, dir: String) -> afs::File {
    let name: String = random_name();
    let dir: &str = dir.as_ref();
    if extention == String::from("tar.gz") {
        async_tar_archive(name, dir.to_string()).await
    } else {
        async_zip_archive(name, dir.to_string()).await
    }
}

fn random_name() -> String {
    use rand::Rng;
    let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCEDFGHIJKLMNOPQRSTUVWXYZ123456789";
    let mut rng = rand::thread_rng();
    (0..10)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}
