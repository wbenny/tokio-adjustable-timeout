use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    sync::mpsc,
    time::{sleep_until, Duration, Instant, Sleep},
};

#[derive(Debug)]
pub struct Elapsed;

#[derive(Debug)]
pub struct Closed;

pub struct Handle {
    tx: mpsc::UnboundedSender<Command>,
}

impl Handle {
    pub(crate) fn new(tx: mpsc::UnboundedSender<Command>) -> Self {
        Self { tx }
    }

    pub fn increment(&self, value: Duration) -> Result<(), Closed> {
        self.tx.send(Command::Increment(value)).map_err(|_| Closed)
    }

    pub fn decrement(&self, value: Duration) -> Result<(), Closed> {
        self.tx.send(Command::Decrement(value)).map_err(|_| Closed)
    }

    pub fn update(&self, deadline: Instant) -> Result<(), Closed> {
        self.tx.send(Command::Update(deadline)).map_err(|_| Closed)
    }
}

enum Command {
    Increment(Duration),
    Decrement(Duration),
    Update(Instant),
}

pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct AdjustableTimeout<T> {
        #[pin]
        future: T,
        #[pin]
        delay: Sleep,

        tx: mpsc::UnboundedSender<Command>,
        rx: mpsc::UnboundedReceiver<Command>,
    }
}

impl<T> AdjustableTimeout<T> {
    pub fn handle(&self) -> Handle {
        Handle::new(self.tx.clone())
    }
}

impl<T> Future for AdjustableTimeout<T>
where
    T: Future,
{
    type Output = Result<T::Output, Elapsed>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        if let Poll::Ready(v) = this.future.poll(cx) {
            return Poll::Ready(Ok(v));
        }

        if let Poll::Ready(cmd) = this.rx.poll_recv(cx) {
            let deadline = match cmd.expect("shouldn't happen") {
                Command::Increment(value) => this.delay.deadline() + value,
                Command::Decrement(value) => this.delay.deadline() - value,
                Command::Update(deadline) => deadline,
            };

            this.delay.as_mut().reset(deadline);
        }

        if let Poll::Ready(()) = this.delay.poll(cx) {
            return Poll::Ready(Err(Elapsed));
        }

        Poll::Pending
    }
}

pub fn adjustable_timeout<T>(duration: Duration, future: T) -> AdjustableTimeout<T>
where
    T: Future,
{
    let (tx, rx) = mpsc::unbounded_channel();
    let deadline = Instant::now() + duration;
    let delay = sleep_until(deadline);

    AdjustableTimeout {
        future,
        delay,
        tx,
        rx,
    }
}
