use crate::P3Task;
use futures::{stream, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;
use std::{error::Error, sync::Arc};

/// P3 downloader, with Parallel Pacman-like Prograssbar
pub struct Downloader {
    /// tasks to download
    tasks: Vec<P3Task>,
    /// number of parallel requests
    parallel: usize,
    /// customized reqwest client
    client: Option<Client>,
}

impl FromIterator<P3Task> for Downloader {
    fn from_iter<T: IntoIterator<Item = P3Task>>(iter: T) -> Self {
        let mut d = Downloader::new();
        for t in iter {
            d.add_task(t);
        }
        d
    }
}

impl FromIterator<(String, String)> for Downloader {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        let mut d = Downloader::new();
        for (url, path) in iter {
            d.add_task(P3Task::from(url, path));
        }
        d
    }
}

impl<'a> FromIterator<(&'a str, &'a str)> for Downloader {
    fn from_iter<T: IntoIterator<Item = (&'a str, &'a str)>>(iter: T) -> Self {
        let mut d = Downloader::new();
        for (url, path) in iter {
            d.add_task(P3Task::from_str(url, path));
        }
        d
    }
}

impl From<Vec<P3Task>> for Downloader {
    fn from(tasks: Vec<P3Task>) -> Self {
        Downloader {
            tasks,
            parallel: 1,
            client: None,
        }
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}

impl Downloader {
    pub fn new() -> Downloader {
        Downloader {
            tasks: Vec::new(),
            parallel: 1,
            client: None,
        }
    }
    /// set number of parallel requests
    pub fn parallel(&mut self, n: usize) -> &mut Self {
        self.parallel = n;
        self
    }
    /// set customized HTTP/HTTPS client
    pub fn client(&mut self, c: Client) -> &mut Self {
        self.client = Some(c);
        self
    }
    /// add a download task
    pub fn add_task(&mut self, t: P3Task) -> &mut Self {
        self.tasks.push(t);
        self
    }

    fn pacman_pb_total_style() -> ProgressStyle {
        ProgressStyle::default_bar()
            .template("{wide_msg} Total ({pos}/{len}) {eta_precise} [{bar:38}] {percent:>3}%")
            .progress_chars("##-")
    }
    fn pacman_pb_total_finish_style() -> ProgressStyle {
        ProgressStyle::default_bar()
            .template("{wide_msg} Total ({pos}/{len}) {elapsed_precise} [{bar:38}] {percent:>3}%")
            .progress_chars("##-")
    }

    /// download all the task
    pub async fn download(&self) -> Result<(), Box<dyn Error>> {
        let total_pb = ProgressBar::new(self.tasks.len() as u64);
        let mp = Arc::new(MultiProgress::new());
        let total_pb = mp.add(total_pb);
        total_pb.set_style(Self::pacman_pb_total_style());
        total_pb.set_message("Downloading...");
        let mp_bg = Arc::clone(&mp);
        let handle_m = tokio::task::spawn_blocking(move || mp_bg.join().unwrap());
        let client = if let Some(client) = &self.client {
            client.clone()
        } else {
            reqwest::Client::builder()
                .user_agent("hyper/0.14.16".to_owned())
                .build()?
        };
        let pbs: Vec<_> = (0..self.tasks.len())
            .map(|i| mp.insert(i, ProgressBar::new(0)))
            .collect();
        let futs = self.tasks.iter().zip(pbs.iter()).map(|(task, pb)| async {
            if let Err(e) = task.download(client.clone(), pb).await {
                pb.set_message(e.to_string());
            }
            total_pb.inc(1);
        });

        // start tasks in parallel
        stream::iter(futs)
            .map(|task| async { task.await })
            .buffered(self.parallel)
            .collect::<Vec<_>>()
            .await;

        total_pb.set_style(Self::pacman_pb_total_finish_style());
        total_pb.finish_with_message("Done.");
        handle_m.await?;
        Ok(())
    }

    pub fn download_blocking(&self) -> Result<(), Box<dyn Error>> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async { self.download().await })?;
        Ok(())
    }
}
