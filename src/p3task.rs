use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::{error::Error, path::PathBuf};
use tokio::{fs::File, io::AsyncWriteExt};

/// task need to be downloaded, including an url and a local file path.
pub struct P3Task {
    /// download url
    url: String,
    /// local file path
    path: PathBuf,
}

impl Default for P3Task {
    fn default() -> Self {
        Self::new()
    }
}

impl P3Task {
    pub fn new() -> P3Task {
        P3Task {
            url: String::from(""),
            path: PathBuf::from(""),
        }
    }

    pub fn from(url: String, path: String) -> P3Task {
        P3Task {
            url,
            path: PathBuf::from(path),
        }
    }

    pub fn from_str(url: &str, path: &str) -> P3Task {
        P3Task {
            url: url.to_owned(),
            path: PathBuf::from(path),
        }
    }

    fn pacman_pb_style() -> ProgressStyle {
        ProgressStyle::default_bar()
                .template("{wide_msg} {bytes:>11} {binary_bytes_per_sec:>11} {eta_precise} [{bar:38}] {percent:>3}%")
                .progress_chars("##-")
    }

    fn pacman_pb_finish_style() -> ProgressStyle {
        ProgressStyle::default_bar()
            .template("{wide_msg} {total_bytes:>11} {binary_bytes_per_sec:>11} {elapsed_precise} [{bar:38}] {percent:>3}%")
            .progress_chars("##-")
    }

    // download this task async
    pub async fn download(&self, client: Client, pb: &ProgressBar) -> Result<(), Box<dyn Error>> {
        pb.set_style(Self::pacman_pb_style());

        let res = client.get(&self.url).send().await?;
        let total_size = res.content_length().ok_or("Failed to get file length")?;
        pb.set_length(total_size);

        let filename = self
            .path
            .file_name()
            .ok_or("Invalid filename")?
            .to_string_lossy()
            .to_string();
        pb.set_message(filename);

        let mut file = File::create(&self.path).await?;
        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();
        while let Some(item) = stream.next().await {
            let chunk = item?;
            let len = file.write(&chunk).await?;
            downloaded = std::cmp::min(downloaded + len as u64, total_size);
            pb.set_position(downloaded);
        }
        pb.set_style(Self::pacman_pb_finish_style());
        pb.finish();
        Ok(())
    }

    #[allow(dead_code)]
    /// for test progress bar
    async fn fake_download(&self, _client: Client, pb: &ProgressBar) -> Result<(), Box<dyn Error>> {
        let total_size: u64 = 1000;
        pb.set_style(Self::pacman_pb_style());
        pb.set_length(total_size);
        let s = self.path.file_name().unwrap().to_string_lossy().to_string();
        pb.set_message(s);

        let mut downloaded = 0;
        while downloaded < total_size {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            let chunk = 50;
            downloaded = std::cmp::min(downloaded + chunk, total_size);
            pb.set_position(downloaded);
        }
        pb.set_style(Self::pacman_pb_finish_style());
        pb.finish();
        Ok(())
    }
}
