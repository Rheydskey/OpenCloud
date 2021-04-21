use crate::lib::file::{get_file_as_byte_vec, get_file_preview};
use actix_web::http::ContentEncoding;
use actix_web::{dev::BodyEncoding, HttpResponse};
use async_std::fs as afs;
use logger::error;
use std::fs::File;
use std::path::PathBuf;
use zip::write::FileOptions;
use zip::CompressionMethod;
use zip_extensions::zip_create_from_directory_with_options;
pub enum ArchiveType {
    Targz,
    Zip,
}

pub async fn download(path: String, atype: ArchiveType) -> HttpResponse {
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
                HttpResponse::Ok().body("No file")
            }
        }
        Err(_) => HttpResponse::Ok().body("Error"),
    }
}
pub async fn get_zip(path: String) -> HttpResponse {
    println!("{}", path.clone());
    HttpResponse::Ok()
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
        .body(get_file_as_byte_vec(path.clone(), ArchiveType::Zip).await)
}

pub async fn get_tar(path: String) -> HttpResponse {
    HttpResponse::Ok()
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
        .body(get_file_as_byte_vec(path.clone(), ArchiveType::Targz).await)
}

async fn async_zip_archive(name: String, dir: String) -> afs::File {
    let file_name = format!("./temp/{}.zip", name);
    File::create(file_name.clone()).unwrap();
    if cfg!(debug_assertions) {
        println!("filename => {}", dir);
    }
    match async_std::task::block_on(async {
        zip_create_from_directory_with_options(
            &PathBuf::from(file_name),
            &PathBuf::from(dir),
            FileOptions::default().compression_method(CompressionMethod::Bzip2),
        )
    }) {
        Ok(_) => {}
        Err(e) => {
            if cfg!(all(feature = "log")) {
                match e {
                    zip::result::ZipError::Io(_) => error("I/O Error"),
                    zip::result::ZipError::InvalidArchive(_) => error("Invalid Archive"),
                    zip::result::ZipError::UnsupportedArchive(_) => error("Unsupported Archive"),
                    zip::result::ZipError::FileNotFound => error("File not found"),
                }
            }
        }
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
    if extention == *"tar.gz" {
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
