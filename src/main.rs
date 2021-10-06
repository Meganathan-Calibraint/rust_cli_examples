use std::env;
use std::io;
use std::fs;
use std::process;
use std::str;
use std::path::Path;
use std::{thread, time};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use futures::StreamExt;
use chromiumoxide::{Browser, BrowserConfig};
extern crate dirs;
extern crate rpassword;

#[derive(Serialize, Deserialize)]
struct Credentials {
    email: String,
    pwd: String,
}

const KEKA_URL: &str = "https://app.keka.com/";

fn check_credential() -> String {
    let path: String = dirs::home_dir().unwrap().into_os_string().into_string().unwrap().to_owned() + "/.keka/credential.json";
    let is_file_exist = Path::new(&path).exists();

    if !is_file_exist {
        "not_exists".to_string()
    } else {
        // read json file from the root path
        let file_response = fs::read_to_string(path).expect("Unable to read file");
        let credentials: serde_json::Value = serde_json::from_str(&file_response).expect("Unable to parse");
        credentials.to_string()
    }
}

fn store_credential(email: &str, pwd: &str) -> std::io::Result<()> {
    // create .keka folder in root
    let mut home_dir: String = dirs::home_dir().unwrap().into_os_string().into_string().unwrap().to_owned();
    let folder_name: String = "/.keka".to_owned();

    home_dir.push_str(&folder_name);

    fs::create_dir_all(home_dir)?;

    let credentials = Credentials {
        email: email.trim().to_string(),
        pwd: pwd.trim().to_string(),
    };

    // serialize it to a JSON string.
    let serialize_credentials = serde_json::to_string(&credentials)?;

    let keka_file_dir: String = dirs::home_dir().unwrap().into_os_string().into_string().unwrap().to_owned() + &folder_name + "/credential.json";

    fs::write(keka_file_dir, serialize_credentials).expect("Unable to write file");
    Ok(())
}

async fn run() {
    let args: Vec<String> = env::args().collect();
    let query = &args[1];

    let mut email = String::new();
    let mut pwd = String::new();

    let result: String = check_credential();

    if result == "not_exists" {
        println!("Please enter your email address: ");

        match io::stdin().read_line(&mut email) {
            Ok(_) => {
                // success
            }
            Err(e) => println!("Oops! Something went wrong: {}", e)
        }

        println!("Please enter your password: ");

        match rpassword::prompt_password_stdout("Password: ") {
            Ok(res) => {
                pwd = res;
                let _response = store_credential(&email, &pwd);
                println!("Your credentials stored in the root path")
            }
            Err(e) => println!("Oops! Something went wrong: {}", e)
        }
    } else {
        let info: Value = serde_json::from_str(&result).unwrap();
        email = info["email"].as_str().unwrap().to_string();
        pwd = info["pwd"].as_str().unwrap().to_string();
    }

    let _response = connect_keka(&email, &pwd, &query).await;
}

async fn connect_keka(email: &str, pwd: &str, clock_type: &str) -> Result<(), Box<dyn std::error::Error>> {

    // headless browser
    let (mut browser, mut handler) =
        Browser::launch(BrowserConfig::builder().build()?).await?;

    let _handle = async_std::task::spawn(async move {
        loop {
            let _event = handler.next().await.unwrap();
        }
    });

    // switch to incognito mode and goto the url
    let page = browser
    .start_incognito_context()
    .await?
    .new_page(KEKA_URL)
    .await?;

    // type into email and password field and hit `Enter`,
    // this triggers a navigation to the keka homepage
    page.find_element("input#email")
            .await?
            .click()
            .await?
            .type_str(email.trim())
            .await?
            .press_key("Enter")
            .await?;

    page.wait_for_navigation().await?.content().await?;        
    
    page.find_element("input#password")
            .await?
            .click()
            .await?
            .type_str(pwd.trim())
            .await?            
            .press_key("Enter")
            .await?;

    println!("loading...");

    let page_html = page.wait_for_navigation().await?.content().await?;

    let invalid_login = page_html.contains("Invalid login attempt.");

    if invalid_login {
        println!("Invalid login attempt.");
        process::exit(0x0100);
    } else {
        // need to change not a permanent solution
        thread::sleep(time::Duration::from_secs(5));

        let home_page = page.wait_for_navigation().await?.content().await?;

        if home_page.contains("Web Clock-In") && clock_type == "clockin" {
            // trigger clock-in button
            page.find_element("home-attendance-clockin-widget button").await?.click().await?;
            page.find_element("home-attendance-clockin-widget button").await?.click().await?;

            // need to change not a permanent solution
            thread::sleep(time::Duration::from_secs(5));

            // trigger cancel modal button
            page.find_element("modal-container button").await?.click().await?;
            println!("keka clock-in successfully");
        } else if home_page.contains("Clock-out") && clock_type == "clockout" {
            // trigger clock-out button
            page.find_element("home-attendance-clockin-widget button").await?.click().await?;
            page.find_element("home-attendance-clockin-widget button").await?.click().await?;

            // need to change not permanent solution
            thread::sleep(time::Duration::from_secs(5));
            
            // trigger cancel modal button
            page.find_element("modal-container button").await?.click().await?;
            println!("keka clock-out successfully");
        } else {
            println!("Oops, already keka has been {}", clock_type);
        }
    }
    process::exit(0x0100);
}

#[async_std::main]
async fn main() {
    run().await;
}