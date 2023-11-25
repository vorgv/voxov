use super::{get_header, Client, Result};
use crate::handle_error;
use std::{fs::File, io::Read};

impl Client {
    /// Get functions metadata.
    pub async fn gene_meta(&self, fed: Option<String>, gid: &str) -> Result<String> {
        let response = self
            .post_head(fed)
            .header("type", "GeneMeta")
            .header("gid", gid)
            .send()
            .await?;
        handle_error!(response);
        self.eprint_cost(&response)?;
        Ok(response.text().await?)
    }

    /// Call function.
    pub async fn gene_call(
        &self,
        fed: Option<String>,
        gid: &str,
        arg: Option<String>,
    ) -> Result<String> {
        let response = self
            .post_head(fed)
            .header("type", "GeneCall")
            .header("gid", gid)
            .header("arg", arg.unwrap_or_default())
            .send()
            .await?;
        handle_error!(response);
        self.eprint_cost(&response)?;
        Ok(response.text().await?)
    }

    /// Map operations.
    pub async fn gene_map_1(&self, file: Option<String>) -> Result<String> {
        match file {
            Some(file) => {
                let mut file = File::open(file)?;
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                self.gene_call(None, "map_1", Some(buf)).await
            }
            None => {
                let mut buf = Vec::<u8>::new();
                std::io::stdin().read_to_end(&mut buf)?;
                let buf = String::from_utf8(buf)?;
                self.gene_call(None, "map_1", Some(buf)).await
            }
        }
    }
}
