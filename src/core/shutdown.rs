use crate::core::types::TtCommand;

pub fn request_shutdown(
    shutdown_tx: &tokio::sync::watch::Sender<bool>,
    tx_tt_cmd: &std::sync::mpsc::Sender<TtCommand>,
) {
    tracing::info!("[SHUTDOWN] Shutdown requested.");
    let _ = shutdown_tx.send(true);
    let _ = tx_tt_cmd.send(TtCommand::Shutdown);
}
