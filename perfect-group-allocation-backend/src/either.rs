#[macro_export]
macro_rules! either_http_body {
    ($name: ident $($ident:literal)*) => {
        paste::paste! {
            #[pin_project(project = [<$name Proj>])]
            pub enum $name<
                $([<Error $ident>]: Into<AppError>,)*
                $([<Option $ident>]: http_body::Body<Data = Bytes, Error = [<Error $ident>]>,)*
            > {
                $([<Option $ident>](#[pin] [<Option $ident>]),)*
            }

            impl<
                $([<Error $ident>]: Into<AppError>,)*
                $([<Option $ident>]: http_body::Body<Data = Bytes, Error = [<Error $ident>]>,)*
            > Body for $name<$([<Error $ident>],)* $([<Option $ident>],)*>
            {
                type Data = Bytes;
                type Error = AppError;

                fn poll_frame(
                    self: Pin<&mut Self>,
                    cx: &mut std::task::Context<'_>,
                ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
                    let this = self.project();
                    match this {
                        $(
                            [<$name Proj>]::[<Option $ident>](option) => option
                                .poll_frame(cx)
                                .map(|poll| poll.map(|opt| opt.map_err(Into::into))),
                        )*
                    }
                }
            }
        }
    };
}
