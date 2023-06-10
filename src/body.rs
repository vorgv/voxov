use http_body_util::combinators::BoxBody;
use hyper::body::{Body, Bytes, Frame};
use std::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

pub enum ResponseBody {
    Box(BoxBody<Bytes, Infallible>),
    Stream(),
}

impl Body for ResponseBody {
    type Data = Bytes;
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match &mut *self.get_mut() {
            Self::Box(boxbody) => Pin::new(&mut *boxbody).poll_frame(cx),
            Self::Stream() => {
                //TODO: impl
                Poll::Ready(None)
            }
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            Self::Box(boxbody) => boxbody.is_end_stream(),
            Self::Stream() => false,
        }
    }

    fn size_hint(&self) -> hyper::body::SizeHint {
        match self {
            Self::Box(boxbody) => boxbody.size_hint(),
            Self::Stream() => hyper::body::SizeHint::default(),
        }
    }
}
