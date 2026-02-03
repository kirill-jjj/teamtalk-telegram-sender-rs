//! Async wrapper around the polling client.
use crate::client::{Client, Message};
use crate::events::Event;
use futures::stream::Stream;
use futures::task::AtomicWaker;
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};
use std::time::Duration;

#[cfg(not(feature = "async-tokio"))]
use std::thread::{self, JoinHandle};

#[cfg(feature = "async-tokio")]
use tokio::task::JoinHandle;

#[cfg(feature = "async-tokio")]
use tokio::time;

/// Configuration for the async polling loop.
#[derive(Clone, Copy)]
pub struct AsyncConfig {
    pub poll_timeout_ms: i32,
    pub buffer: usize,
}

impl Default for AsyncConfig {
    fn default() -> Self {
        Self {
            poll_timeout_ms: 100,
            buffer: 512,
        }
    }
}

impl AsyncConfig {
    /// Creates a configuration with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the polling timeout in milliseconds.
    pub fn poll_timeout_ms(mut self, timeout_ms: i32) -> Self {
        self.poll_timeout_ms = timeout_ms;
        self
    }

    /// Sets the channel buffer size for events.
    pub fn buffer(mut self, buffer: usize) -> Self {
        self.buffer = buffer;
        self
    }
}

/// Async stream of client events backed by a worker thread.
pub struct AsyncClient {
    client: Option<Client>,
    stop: Arc<AtomicBool>,
    poll_timeout_ms: i32,
    wake_pending: Arc<AtomicBool>,
    waker: Arc<AtomicWaker>,
    buffer: VecDeque<(Event, Message)>,
    buffer_cap: usize,
    #[cfg(not(feature = "async-tokio"))]
    ticker: Option<JoinHandle<()>>,
    #[cfg(feature = "async-tokio")]
    ticker: Option<JoinHandle<()>>,
}

impl AsyncClient {
    /// Creates an async client with default configuration.
    pub fn new(client: Client) -> Self {
        Self::with_config(client, AsyncConfig::default())
    }

    /// Creates an async client with custom configuration.
    pub fn with_config(client: Client, config: AsyncConfig) -> Self {
        let buffer_cap = config.buffer.max(1);
        let stop = Arc::new(AtomicBool::new(false));
        let wake_pending = Arc::new(AtomicBool::new(false));
        let waker = Arc::new(AtomicWaker::new());
        let delay = Duration::from_millis(config.poll_timeout_ms.max(1) as u64);
        let ticker_stop = Arc::clone(&stop);
        let ticker_pending = Arc::clone(&wake_pending);
        let ticker_waker = Arc::clone(&waker);
        #[cfg(feature = "async-tokio")]
        let ticker = Some(tokio::spawn(async move {
            while !ticker_stop.load(Ordering::Relaxed) {
                time::sleep(delay).await;
                if ticker_pending.swap(false, Ordering::Relaxed) {
                    ticker_waker.wake();
                }
            }
        }));
        #[cfg(not(feature = "async-tokio"))]
        let ticker = Some(thread::spawn(move || {
            while !ticker_stop.load(Ordering::Relaxed) {
                thread::sleep(delay);
                if ticker_pending.swap(false, Ordering::Relaxed) {
                    ticker_waker.wake();
                }
            }
        }));
        Self {
            client: Some(client),
            stop,
            poll_timeout_ms: config.poll_timeout_ms,
            wake_pending,
            waker,
            buffer: VecDeque::with_capacity(buffer_cap),
            buffer_cap,
            ticker,
        }
    }

    /// Runs a closure with a shared client reference.
    pub fn with_client<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&Client) -> R,
    {
        let client = self.client.as_ref()?;
        Some(f(client))
    }

    /// Runs a closure with a mutable client reference.
    pub fn with_client_mut<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut Client) -> R,
    {
        let client = self.client.as_mut()?;
        Some(f(client))
    }

    /// Stops the async polling loop.
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    /// Stops the loop and returns the underlying client.
    pub fn into_client(mut self) -> Option<Client> {
        self.stop();
        self.client.take()
    }
}

impl Stream for AsyncClient {
    type Item = (Event, Message);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.stop.load(Ordering::Relaxed) {
            return Poll::Ready(None);
        }
        let Some(client) = this.client.as_ref() else {
            return Poll::Ready(None);
        };
        if let Some(item) = this.buffer.pop_front() {
            return Poll::Ready(Some(item));
        }
        if let Some((event, message)) = client.poll(this.poll_timeout_ms) {
            this.buffer.push_back((event, message));
            while this.buffer.len() < this.buffer_cap {
                let Some((event, message)) = client.poll(0) else {
                    break;
                };
                this.buffer.push_back((event, message));
            }
            let item = this.buffer.pop_front();
            return Poll::Ready(item);
        }
        this.waker.register(cx.waker());
        this.wake_pending.store(true, Ordering::Relaxed);
        Poll::Pending
    }
}

impl Drop for AsyncClient {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(ticker) = self.ticker.take() {
            #[cfg(feature = "async-tokio")]
            ticker.abort();
            #[cfg(not(feature = "async-tokio"))]
            let _ = ticker.join();
        }
    }
}

impl Client {
    /// Converts the client into an async event stream.
    pub fn into_async(self) -> AsyncClient {
        AsyncClient::new(self)
    }

    /// Converts the client into an async event stream with configuration.
    pub fn into_async_with_config(self, config: AsyncConfig) -> AsyncClient {
        AsyncClient::with_config(self, config)
    }
}
