use std::{sync::Arc, time::Duration};

use anyhow::Result;
use chromiumoxide::{
    browser::{Browser, BrowserConfig},
    handler::viewport::Viewport,
};
use futures::StreamExt;
use tokio::task::JoinHandle;

pub struct Ecd {
    pub login: String,
    pub password: String,
    pub browser: Browser,
    pub handle: Option<JoinHandle<()>>,
}

impl Ecd {
    pub async fn new(
        login: impl AsRef<str>,
        password: impl AsRef<str>,
        with_head: bool,
        chrome_path: Option<impl AsRef<str>>,
    ) -> Result<Self> {
        let login = login.as_ref().to_string();
        let password = password.as_ref().to_string();
        let viewport = Viewport {
            width: 1024,
            height: 768,
            ..Default::default()
        };
        let mut browser_config = BrowserConfig::builder()
            .no_sandbox()
            .window_size(1024, 768)
            .viewport(viewport)
            .arg("--disable-setuid-sandbox")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-site-isolation-trials");
        if with_head {
            browser_config = browser_config.with_head();
        }
        if let Some(chrome_path) = chrome_path {
            browser_config = browser_config.chrome_executable(chrome_path.as_ref());
        }

        let (mut browser, mut handler) = Browser::launch(browser_config.build().unwrap())
            .await
            .unwrap();

        let handle = Some(tokio::task::spawn(async move {
            loop {
                let _ = handler.next().await;
            }
        }));

        Ok(Self {
            login,
            password,
            browser,
            handle,
        })
    }
    pub async fn new_from_env() -> Result<Self> {
        let login = std::env::var("ECD_LOGIN")?;
        let password = std::env::var("ECD_PASSWORD")?;
        let with_head = std::env::var("WITH_HEAD")
            .map(|v| v == "true")
            .unwrap_or(false);
        let chrome_path = std::env::var("ECD_CHROME_PATH").ok();
        Self::new(login, password, with_head, chrome_path).await
    }
}

impl Ecd {
    pub async fn login(&mut self) -> Result<String> {
        let page = self
            .browser
            .start_incognito_context()
            .await?
            .new_page("https://www.eurocash.pl")
            .await?;
        page.wait_for_navigation_response().await?;
        if let Ok(elem) = page.find_element("#c-p-bn").await {
            elem.click().await?;
            page.wait_for_navigation_response().await?;
        };
        let mut i = 0;
        loop {
            if i < 10 {
                if let Ok(elem) = page.find_element("#ecHeader > div.fi.relative > div.menu.menu--desktop > div > a.btn.btn--green-login.m-r-25").await {
                    elem.click().await?;
                    page.wait_for_navigation_response().await?;
                    break;
                } else {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    i += 1;
                }
            } else {
                self.browser.close().await?;
                return Err(anyhow::anyhow!("Couldn't find login button"));
            }
        }

        loop {
            if i < 10 {
                let login = page.find_element("#login").await;
                let password = page.find_element("#password").await;
                if let (Ok(login), Ok(password)) = (login, password) {
                    login.click().await?.type_str(self.login.as_str()).await?;
                    password
                        .click()
                        .await?
                        .type_str(self.password.as_str())
                        .await?
                        .press_key("Enter")
                        .await?;
                    page.wait_for_navigation_response().await?;
                    i = 0;
                    break;
                } else {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    i += 1;
                }
            } else {
                self.browser.close().await?;
                return Err(anyhow::anyhow!("Couldn't find login or password field"));
            }
        }

        loop {
            if i < 10 {
                if let Ok(Some(token)) = page
                    .evaluate("localStorage.access_token")
                    .await?
                    .into_value::<Option<String>>()
                {
                    println!("Token: {:#?}", token);
                    self.browser.close().await?;
                    return Ok(token);
                } else {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    i += 1;
                }
            } else {
                return Err(anyhow::anyhow!("Couldn't find token"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn login() {
        let mut ecd = Ecd::new_from_env().await.unwrap();
        let token = ecd.login().await.unwrap();
        drop(ecd);
        dbg!(token);
    }
}
