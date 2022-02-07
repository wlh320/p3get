# p3get

`p3get` is a rust library for easily creating file downloader with **p3**: Parallel, Pacman-like Progressbar.

## Usage

Example:

```rust
use p3get::Downloader;
use p3get::P3Task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resources = vec![
        (
            "http://mirrors.163.com/archlinux/iso/2022.02.01/archlinux-2022.02.01-x86_64.iso.torrent",
            "/home/wlh/Downloads/a.torrent",
        ),
        (
            "http://mirrors.163.com/archlinux/iso/2022.02.01/archlinux-bootstrap-2022.02.01-x86_64.tar.gz",
            "/home/wlh/Downloads/b.tar.gz",
        ),
    ];
    // add multiple download tasks from iterator
    let mut downloader = Downloader::from_iter(resources.into_iter());
    // or add one task by hand
    downloader.add_task(P3Task::from_str(
        "https://www.baidu.com/",
        "/home/wlh/Downloads/index.html",
    ));
    // [optional] config customized reqwest client to use proxy or something
    let c = reqwest::Client::builder()
        .user_agent("p3get/0.0.1".to_owned())
        .build()?;
    downloader.client(c);
    // set number of parallel requests and download
    downloader.parallel(2).download().await?;
    Ok(())
}
```

Output will be like:

```
a.torrent     46.59KiB 110.78KiB/s 00:00:00 [######################################] 100%
b.tar.gz       4.54MiB   1.83MiB/s 00:01:27 [##------------------------------------]   3%
index.html        227B      468B/s 00:00:00 [######################################] 100%
Downloading...         Total (2/3) 00:00:01 [##########################------------]  67%
```

## TODO

This project is WIP.

- better error handling
- show total bytes and speed on total progress bar
