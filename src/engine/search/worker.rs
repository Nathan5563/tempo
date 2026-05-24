use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError},
    },
    thread::{self, JoinHandle},
};

use crate::engine::position::{Move, Position};

use super::{SearchLimits, first_legal_move, search};

pub(crate) struct SearchWorker
{
    stop: Arc<AtomicBool>,
    receiver: Receiver<Option<Move>>,
    handle: Option<JoinHandle<()>>,
    fallback: Option<Move>,
}

impl SearchWorker
{
    pub(crate) fn start(mut position: Position, limits: SearchLimits) -> Self
    {
        let fallback = first_legal_move(&mut position);
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move ||
        {
            let result = search(&mut position, &limits, thread_stop.as_ref());
            let _ = sender.send(result);
        });
        Self { stop, receiver, handle: Some(handle), fallback }
    }

    fn request_stop(&self)
    {
        self.stop.store(true, Ordering::Relaxed);
    }

    pub(crate) fn try_finish(&mut self) -> Option<Option<Move>>
    {
        match self.receiver.try_recv()
        {
            Ok(result) =>
            {
                self.join();
                Some(result.or(self.fallback))
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) =>
            {
                self.join();
                Some(self.fallback)
            }
        }
    }

    pub(crate) fn finish(mut self) -> Option<Move>
    {
        self.request_stop();
        self.join();
        self.receiver
            .try_recv()
            .unwrap_or(None)
            .or(self.fallback)
    }

    fn join(&mut self)
    {
        if let Some(handle) = self.handle.take()
        {
            let _ = handle.join();
        }
    }
}

impl Drop for SearchWorker
{
    fn drop(&mut self)
    {
        self.request_stop();
        self.join();
    }
}
