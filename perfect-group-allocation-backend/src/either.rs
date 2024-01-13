#[macro_export]
macro_rules! either_http_body {
    ($name: ident $($ident:literal)*) => {
        ::paste::paste! {
            #[::pin_project::pin_project(project = [<$name Proj>])]
            enum $name<
                $([<Option $ident>]: ::http_body::Body<Data = ::bytes::Bytes, Error = ::core::convert::Infallible>,)*
            > {
                $([<Option $ident>](#[pin] [<Option $ident>]),)*
            }

            impl<
                $([<Option $ident>]: ::http_body::Body<Data = ::bytes::Bytes, Error = ::core::convert::Infallible>,)*
            > Body for $name<$([<Option $ident>],)*>
            {
                type Data = ::bytes::Bytes;
                type Error = ::core::convert::Infallible;

                fn poll_frame(
                    self: ::core::pin::Pin<&mut Self>,
                    cx: &mut ::std::task::Context<'_>,
                ) -> ::std::task::Poll<Option<Result<::http_body::Frame<Self::Data>, Self::Error>>> {
                    let this = self.project();
                    match this {
                        $(
                            [<$name Proj>]::[<Option $ident>](option) => option.poll_frame(cx),
                        )*
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! either_future {
    ($name: ident $($ident:literal)*) => {
        ::paste::paste! {
            #[::pin_project::pin_project(project = [<$name Proj>])]
            enum $name<
                Output,
                $([<Option $ident>]: ::core::future::Future<Output = Output>,)*
            > {
                $([<Option $ident>](#[pin] [<Option $ident>]),)*
            }

            impl<
                Output,
                $([<Option $ident>]: ::core::future::Future<Output = Output>,)*
            > ::core::future::Future for $name<Output, $([<Option $ident>],)*>
            {
                type Output = Output;

                fn poll(
                    self: ::core::pin::Pin<&mut Self>,
                    cx: &mut ::std::task::Context<'_>
                ) -> ::std::task::Poll<Self::Output> {
                    let this = self.project();
                    match this {
                        $(
                            [<$name Proj>]::[<Option $ident>](option) => option.poll(cx),
                        )*
                    }
                }
            }
        }
    };
}
