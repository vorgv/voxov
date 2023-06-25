use http_body_util::{combinators::BoxBody, StreamBody};
use hyper::body::{Body, Bytes, Frame, SizeHint};
use std::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};
use tokio_stream::Stream;

pub type BytesStream = Pin<Box<dyn Stream<Item = Bytes> + Send>>;

pub enum ResponseBody {
    Box(BoxBody<Bytes, Infallible>),
    Stream(StreamBody<BytesStream>),
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
            Self::Stream(s) => Pin::new(&mut *s)
                .poll_next(cx)
                .map(|maybe_bytes| maybe_bytes.map(|bytes| Ok(Frame::data(bytes)))),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            Self::Box(b) => b.is_end_stream(),
            Self::Stream(_) => false,
        }
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            Self::Box(b) => b.size_hint(),
            Self::Stream(_) => SizeHint::default(),
        }
    }
}
