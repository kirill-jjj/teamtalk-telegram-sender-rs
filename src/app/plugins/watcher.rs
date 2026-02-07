use super::manager::PluginManagerHandle;
use notify::{Event, RecursiveMode, Watcher};
use std::path::Path;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub fn spawn_watcher(
    root: &Path,
    plugins: PluginManagerHandle,
    cancel_token: CancellationToken,
    disabled: Vec<String>,
) -> anyhow::Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
    let mut watcher = notify::recommended_watcher(move |result: notify::Result<Event>| {
        if let Ok(event) = result {
            let _ = tx.send(event);
        }
    })?;
    watcher.watch(root, RecursiveMode::Recursive)?;
    tokio::spawn(async move {
        let _watcher = watcher;
        loop {
            tokio::select! {
                () = cancel_token.cancelled() => break,
                fs_event = rx.recv() => {
                    let Some(event) = fs_event else {
                        break;
                    };
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    while rx.try_recv().is_ok() {}
                    if let Some(path) = event.paths.first() {
                        plugins.reload_changed(path, &disabled).await;
                    } else {
                        plugins.reload_all(&disabled).await;
                    }
                }
            }
        }
    });
    Ok(())
}
