#[macro_export]
macro_rules! either_http_body {
    (boxed $name: ident $($ident:literal)*) => {
        ::paste::paste! {
            pub struct $name(::core::pin::Pin<Box<dyn ::http_body::Body<Data = ::bytes::Bytes, Error = ::core::convert::Infallible> + Send>>);

            impl $name {
                $(
                    #[allow(non_snake_case)]
                    pub fn [<Option $ident>](body: impl ::http_body::Body<Data = ::bytes::Bytes, Error = ::core::convert::Infallible> + Send + 'static) -> Self {
                        Self(Box::pin(body))
                    }
                )*
            }

            impl Body for $name
            {
                type Data = ::bytes::Bytes;
                type Error = ::core::convert::Infallible;

                fn poll_frame(
                    mut self: ::core::pin::Pin<&mut Self>,
                    cx: &mut ::std::task::Context<'_>,
                ) -> ::std::task::Poll<Option<Result<::http_body::Frame<Self::Data>, Self::Error>>> {
                    self.0.as_mut().poll_frame(cx)
                }
            }
        }
    };
    (either $name: ident $($ident:literal)*) => {
        ::paste::paste! {
            #[::pin_project::pin_project(project = [<$name Proj>])]
            pub enum $name<
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
    (boxed $name: ident $($ident:literal)*) => {
        ::paste::paste! {
            pub struct $name<Output>(::core::pin::Pin<Box<dyn ::core::future::Future<Output = Output> + Send>>);

            impl<Output> $name<Output> {
                $(
                    #[allow(non_snake_case)]
                    pub fn [<Option $ident>](future: impl ::core::future::Future<Output = Output> + Send + 'static) -> Self {
                        Self(Box::pin(future))
                    }
                )*
            }

            impl<Output> ::core::future::Future for $name<Output>
            {
                type Output = Output;

                fn poll(
                    mut self: ::core::pin::Pin<&mut Self>,
                    cx: &mut ::std::task::Context<'_>
                ) -> ::std::task::Poll<Self::Output> {
                    self.0.as_mut().poll(cx)
                }
            }
        }
    };
    (either $name: ident $($ident:literal)*) => {
        ::paste::paste! {
            #[::pin_project::pin_project(project = [<$name Proj>])]
            pub enum $name<
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
