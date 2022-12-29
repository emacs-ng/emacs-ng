use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use std::{os::unix::prelude::AsRawFd, ptr};

use futures::FutureExt;

use tokio::io::{unix::AsyncFd, Interest};

use crate::event_loop::{FdSet, Timespec};

pub struct SelectBatch<Fut> {
    inner: Vec<Fut>,
}

impl<Fut: Unpin> Unpin for SelectBatch<Fut> {}

fn batch_select<I>(iter: I) -> SelectBatch<I::Item>
where
    I: IntoIterator,
    I::Item: Future + Unpin,
{
    let ret = SelectBatch {
        inner: iter.into_iter().collect(),
    };
    ret
}

impl<Fut: Future + Unpin> Future for SelectBatch<Fut> {
    type Output = Vec<(Fut::Output, usize)>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let items = self
            .inner
            .iter_mut()
            .enumerate()
            .filter_map(|(i, f)| match f.poll_unpin(cx) {
                Poll::Pending => None,
                Poll::Ready(e) => Some((e, i)),
            })
            .collect::<Vec<_>>();

        if items.is_empty() {
            return Poll::Pending;
        }

        Poll::Ready(items)
    }
}

fn fd_set_to_async_fds(nfds: i32, fds: &FdSet, interest: Interest) -> Vec<AsyncFd<i32>> {
    if fds.0 == ptr::null_mut() {
        return Vec::new();
    }

    let mut async_fds = Vec::new();

    for fd in 0..nfds {
        unsafe {
            if libc::FD_ISSET(fd, fds.0) {
                let async_fd_result = AsyncFd::with_interest(fd, interest);
                if async_fd_result.is_err() {
                    println!("AsyncFd err: {:?}", async_fd_result.unwrap_err());
                    continue;
                }

                async_fds.push(async_fd_result.unwrap())
            }
        }
    }

    async_fds
}

fn async_fds_to_fd_set(fds: Vec<i32>, fd_set: &FdSet) {
    if fd_set.0 == ptr::null_mut() {
        return;
    }

    unsafe { libc::FD_ZERO(fd_set.0) }

    for f in fds {
        unsafe { libc::FD_SET(f, fd_set.0) }
    }
}

pub async fn tokio_select_fds(
    nfds: i32,
    readfds: &FdSet,
    writefds: &FdSet,
    _timeout: &Timespec,
) -> i32 {
    let read_fds = fd_set_to_async_fds(nfds, readfds, Interest::READABLE);
    let write_fds = fd_set_to_async_fds(nfds, writefds, Interest::WRITABLE);

    let mut fd_futures = Vec::new();

    for f in read_fds.iter() {
        fd_futures.push(f.readable().boxed())
    }

    for f in write_fds.iter() {
        fd_futures.push(f.writable().boxed())
    }

    let read_fds_count = read_fds.len();

    let readliness = batch_select(fd_futures).await;

    let mut readable_result = Vec::new();
    let mut writable_result = Vec::new();

    for (result, index) in readliness {
        if result.is_err() {
            continue;
        }

        if index < read_fds_count {
            readable_result.push(read_fds[index].as_raw_fd())
        } else {
            writable_result.push(write_fds[index - read_fds_count].as_raw_fd())
        }
    }

    let nfds = readable_result.len() + writable_result.len();

    async_fds_to_fd_set(readable_result, readfds);
    async_fds_to_fd_set(writable_result, writefds);

    nfds as i32
}
