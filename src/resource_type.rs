pub trait ResourceType: Copy {
    type Data;
    fn name(&self) -> &'static str;
}

#[macro_export]
macro_rules! resources {
    ( $set_name:ident, $macro_name:ident, { $(($enum:ident, $data:ty, $cname:ident, $name:expr),)* } ) => {
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
            ( |$n:ident| => $body:block ) => {
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
    }
}
