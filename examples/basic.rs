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
