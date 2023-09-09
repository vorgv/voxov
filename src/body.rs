use http_body_util::{combinators::BoxBody, StreamBody};
use hyper::body::{Body, Bytes, Frame, SizeHint};
use s3::error::S3Error;
use std::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};
use tokio_stream::Stream;

pub type S3StreamItem = Result<Bytes, S3Error>;
pub type BytesStream = Pin<Box<dyn Stream<Item = S3StreamItem> + Send>>;

pub enum ResponseBody {
    Box(BoxBody<Bytes, Infallible>),
    S3Stream(StreamBody<BytesStream>),
}

impl Body for ResponseBody {
    type Data = Bytes;
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match &mut *self.get_mut() {
            Self::Box(b) => Pin::new(&mut *b).poll_frame(cx),
            Self::S3Stream(s) => {
                let error = false;
                let poll = Pin::new(&mut *s).poll_next(cx).map(|maybe_item| {
                    maybe_item.map(|item| {
                        Ok(match item {
                            Ok(bytes) => Frame::data(bytes),
                            Err(_) => Frame::data(Bytes::default()),
                        })
                    })
                });
                if error {
                    return Poll::Ready(None);
                }
                poll
            }
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            Self::Box(b) => b.is_end_stream(),
            Self::S3Stream(_) => false,
        }
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            Self::Box(b) => b.size_hint(),
            Self::S3Stream(_) => SizeHint::default(),
        }
    }
}
