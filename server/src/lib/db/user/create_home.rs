use async_std::fs::*;
use logger::error;

pub struct Result {
    pub result: bool,
    pub body: String,
}

pub async fn create_home(name: String) -> Result {
    match create_dir(format!("./home/{}", name.clone())).await {
        Ok(_) => {
            if create_dir(format!("./home/{}/{}", name.clone(), "photo".to_string()))
                .await
                .is_err()
                && cfg!(feature = "log")
            {
                error("Error on create photo folder")
            }
            if create_dir(format!("./home/{}/{}", name.clone(), "video".to_string()))
                .await
                .is_err()
                && cfg!(feature = "log")
            {
                error("Error on create video folder")
            }
            if create_dir(format!("./home/{}/{}", name.clone(), "music".to_string()))
                .await
                .is_err()
                && cfg!(feature = "log")
            {
                error("Error on create music folder")
            }
            if create_dir(format!(
                "./home/{}/{}",
                name.clone(),
                "document".to_string()
            ))
            .await
            .is_err()
                && cfg!(feature = "log")
            {
                error("Error on create document folder")
            }

            Result {
                result: true,
                body: "Your request has been accepted".to_string(),
            }
        }
        Err(e) => match e.raw_os_error().unwrap_or_default() {
            17 => Result {
                result: false,
                body: "User Already Exist".to_string(),
            },
            _ => Result {
                result: false,
                body: "Unknow Error".to_string(),
            },
        },
    }
}
