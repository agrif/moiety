pub trait ResourceType: Copy {
    type Data;
    fn name(&self) -> &'static str;
}

// used to trick macro_rules! into looping when we only want a constant
macro_rules! constant_loop {
    ($tok:tt, $($_t:tt)*) => {
        $tok
    };
}

#[macro_export]
macro_rules! resources {
    ( $set_name:ident, $format_name:ident, $error_name:ident, $macro_name:ident, { $(($enum:ident, $data:ty, $cname:ident, $tyvar:ident, $fname:ident, $error_enum:ident, $name:expr),)* } ) => {
        pub enum $set_name<D> {
            $(
                $enum(refl::Id<D, $data>),
            )*
        }

        $(
            impl $set_name<$data> {
                pub const $cname: Self = $set_name::$enum(refl::Id::REFL);
            }
        )*

        #[macro_export]
        macro_rules! $macro_name {
            (|$n:ident| => $body:block ) => {
                $(
                    {
                        let $n = $set_name::$enum(refl::Id::REFL);
                        $body;
                    }
                )*
            }
        }

        impl<D> Copy for $set_name<D> {}

        impl<D> Clone for $set_name<D> {
            fn clone(&self) -> Self {
                match self {
                    $(
                        $set_name::$enum(ref id) => $set_name::$enum(id.clone()),
                    )*
                }
            }
        }

        impl<D> std::fmt::Debug for $set_name<D> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $(
                        $set_name::$enum(_) => {
                            write!(f, stringify!($set_name))?;
                            write!(f, "::")?;
                            write!(f, stringify!($cname))
                        },
                    )*
                }
            }
        }
        impl<D> $crate::ResourceType for $set_name<D> {
            type Data = D;
            fn name(&self) -> &'static str {
                match self {
                    $(
                        $set_name::$enum(_) => $name,
                    )*
                }
            }
        }

        #[derive(Debug, PartialEq, Eq, Hash, Clone)]
        pub struct $format_name<$($tyvar),*> {
            $(pub $fname : $tyvar),*
        }

        #[derive(Debug, PartialEq, Eq, Hash, Clone, Fail)]
        pub enum $error_name<$($tyvar: failure::Fail),*> {
            $(
                #[fail(display = "{}", _0)]
                $error_enum(#[cause] $tyvar),
            )*
        }

        impl<A> $format_name<$(constant_loop!(A, $enum)),*> where A: Clone {
            pub fn new(default: A) -> Self {
                $format_name {
                    $($fname : default.clone()),*
                }
            }
        }

        impl<I, $($tyvar),*> $crate::Format<I> for $format_name<$($tyvar),*> where $($tyvar : $crate::Format<I>),* {
            type Error = $error_name<$($tyvar::Error),*>;
        }

        resources!(@formatfor, $set_name, $format_name, $error_name, <$($tyvar),*>, {
            $(($enum, $data, $tyvar, $fname, $error_enum),)*
        });

        resources!(@formatwritefor, $set_name, $format_name, $error_name, <$($tyvar),*>, {
            $(($enum, $data, $tyvar, $fname, $error_enum),)*
        });
    };

    (@formatfor, $set_name:ident, $format_name:ident, $error_name:ident, <$($tyvars:ident),*>, { }) => {
    };

    (@formatfor, $set_name:ident, $format_name:ident, $error_name:ident, <$($tyvars:ident),*>, { ( $enum:ident, $data:ty, $tyvar:ident, $fname:ident, $error_enum:ident), $($rest:tt)* }) => {
        impl<I, $($tyvars),*> $crate::FormatFor<I, $set_name<$data>> for $format_name<$($tyvars),*> where $tyvar: $crate::FormatFor<I, $set_name<$data>>, $($tyvars: $crate::Format<I>),* {
            fn convert<'a>(&'a self, input: I) -> $crate::future::Fut<'a, Result<$data, Self::Error>>
            where
                I: 'a
            {
                fut!({
                    await!(self.$fname.convert(input)).map_err($error_name::$error_enum)
                })
            }

            fn extension<'a>(&'a self) -> Option<&'a str> {
                self.$fname.extension()
            }
        }

        resources!(@formatfor, $set_name, $format_name, $error_name, <$($tyvars),*>, {
            $($rest)*
        });
    };

    (@formatwritefor, $set_name:ident, $format_name:ident, $error_name:ident, <$($tyvars:ident),*>, { }) => {
    };

    (@formatwritefor, $set_name:ident, $format_name:ident, $error_name:ident, <$($tyvars:ident),*>, { ( $enum:ident, $data:ty, $tyvar:ident, $fname:ident, $error_enum:ident), $($rest:tt)* }) => {
        impl<I, F, $($tyvars),*> $crate::FormatWriteFor<I, $set_name<$data>, F> for $format_name<$($tyvars),*> where $tyvar: $crate::FormatWriteFor<I, $set_name<$data>, F>, F: $crate::FormatFor<I, $set_name<$data>> {
            type WriteError = $tyvar::WriteError;

            fn write<'a>(
                &'a self,
                input: I,
                fmt: &'a F,
            ) -> $crate::future::Fut<'a, Result<Vec<u8>, $crate::ConvertError<F::Error, Self::WriteError>>>
            where
                I: 'a,
                F: 'a
            {
                self.$fname.write(input, fmt)
            }
        }

        resources!(@formatwritefor, $set_name, $format_name, $error_name, <$($tyvars),*>, {
            $($rest)*
        });
    };
}
