use super::{get_header, Client, Result};
use crate::handle_error;
use bytes::Bytes;
use std::{
    fs::File,
    io::{Read, Write},
    process::exit,
};

impl Client {
    /// Get metadata of a meme.
    pub async fn meme_meta(&self, hash: String) -> Result<String> {
        let response = self
            .post_head(None)
            .header("type", "MemeMeta")
            .header("hash", hash)
            .send()
            .await?;
        handle_error!(response);
        self.eprint_cost(&response)?;
        Ok(response.text().await?)
    }

    /// Upload bytes.
    pub async fn meme_put(&self, days: u32, bytes: Bytes) -> Result<String> {
        let builder = self
            .post_head(None)
            .header("type", "MemePut")
            .header("days", days)
            .body(bytes);
        let response = builder.send().await?;
        handle_error!(response);
        self.eprint_cost(&response)?;
        let hash = get_header(&response, "hash");
        Ok(hash)
    }

    /// Upload a file.
    pub async fn meme_put_file(&self, days: u32, file: Option<String>) -> Result<String> {
        let mut builder = self
            .post_head(None)
            .header("type", "MemePut")
            .header("days", days);
        builder = match file {
            Some(file) => {
                let mut file = File::open(file)?;
                let mut buf = Vec::<u8>::new();
                file.read_to_end(&mut buf)?;
                builder.body(buf)
            }
            None => builder.body({
                let mut buf = Vec::<u8>::new();
                std::io::stdin().read_to_end(&mut buf)?;
                buf
            }),
        };
        let response = builder.send().await?;
        handle_error!(response);
        self.eprint_cost(&response)?;
        let hash = get_header(&response, "hash");
        Ok(hash)
    }

    /// Download bytes.
    pub async fn meme_get(&self, public: bool, hash: String) -> Result<Bytes> {
        let mut builder = self
            .post_head(None)
            .header("type", "MemeGet")
            .header("hash", hash);
        builder = match public {
            true => builder.header("public", "true"),
            false => builder.header("public", "false"),
        };
        let response = builder.send().await?;
        handle_error!(response);
        self.eprint_cost(&response)?;
        Ok(response.bytes().await?)
    }

    /// Download a file.
    pub async fn meme_get_file(
        &self,
        public: bool,
        hash: String,
        file: Option<String>,
    ) -> Result<String> {
        let mut builder = self
            .post_head(None)
            .header("type", "MemeGet")
            .header("hash", hash);
        builder = match public {
            true => builder.header("public", "true"),
            false => builder.header("public", "false"),
        };
        let response = builder.send().await?;
        handle_error!(response);
        self.eprint_cost(&response)?;
        match file {
            Some(file) => {
                let mut file = File::create(file)?;
                file.write_all(&response.bytes().await?)?;
                Ok("".into())
            }
            None => {
                std::io::stdout().write_all(&response.bytes().await?)?;
                exit(0);
            }
        }
    }
}
