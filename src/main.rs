use futures::StreamExt;
use chromiumoxide::{Browser, BrowserConfig};
// extern crate dirs;
use std::{thread, time};

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let (mut browser, mut handler) =
        Browser::launch(BrowserConfig::builder().with_head().build()?).await?;

    let handle = async_std::task::spawn(async move {
        loop {
            let _event = handler.next().await.unwrap();
        }
    });

    // switch to incognito mode and goto the url
    let page = browser.start_incognito_context().await?.new_page("https://app.keka.com/").await?;

    // type into email and password field and hit `Enter`,
    // this triggers a navigation to the keka homepage
    page.find_element("input#email")
            .await?
            .click()
            .await?
            .type_str("meganathan.p@calibraint.com")
            .await?
            .press_key("Enter")
            .await?;

    page.wait_for_navigation().await?.content().await?;        
    
    page.find_element("input#password")
            .await?
            .click()
            .await?
            .type_str("MP26@Calibraint")
            .await?            
            .press_key("Enter")
            .await?;

    let page_html = page.wait_for_navigation().await?.content().await?;

    let invalid_login = page_html.contains("Invalid login attempt.");

    // println!("{:?}", dirs::home_dir());

    if invalid_login {
        println!("invalid login");
    } else {
        println!("login successful");
    }

    thread::sleep(time::Duration::from_secs(5));

    handle.await;
    Ok(())
}
