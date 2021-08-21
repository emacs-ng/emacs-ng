use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::future::FutureExt;

pub struct SelectBatch<Fut> {
    inner: Vec<Fut>,
}

impl<Fut: Unpin> Unpin for SelectBatch<Fut> {}

pub fn batch_select<I>(iter: I) -> SelectBatch<I::Item>
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
