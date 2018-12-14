#[macro_export] macro_rules! _parser_from_patt {
    ($fun: ident, $cls:ident, $ctr:ident, $patt:expr, $suff:expr, $($field:ident: $ty:ty),*) => {
        fn $fun(s: &str) -> std::io::Result<$cls> {
            let ($($field),*) =
                scan_fmt!(s, $patt, $($ty),*);

            {
                $(
                    let $field = $field
                        .ok_or_else(|| std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            concat!(
                                stringify!($cls),
                                ": parse failed on ",
                                stringify!($field), "!")))?
                );*;

                if s.ends_with($suff) {
                    Ok($ctr { $($field),* })
                } else {
                    Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            concat!(
                                stringify!($cls),
                                ": parse failed on suffix!")))?
                }
            }
        }
    }
}

/**
 * Defines a struct/enum that can be constructed from a string, based on a
 * format pattern.
 */
#[macro_export] macro_rules! input {
    (
        #[$patt:expr; $suff:expr]
        struct $name:ident {
            $($field:ident: $ty:ty),*
        }
    ) => {
        #[macro_use] extern crate scan_fmt;

        #[derive(Debug)]
        struct $name {
            $($field: $ty),*
        }

        impl $name {
            _parser_from_patt! { new, $name, $name, $patt, $suff, $($field: $ty),* }
        }
    };

    (
        enum $name:ident {
            $(
                #[$patt:expr; $suff:expr]
                $label:ident {
                    $($field:ident: $ty:ty),*
                }
            ),*
        }
    ) => {
        #[macro_use] extern crate scan_fmt;

        #[derive(Debug)]
        enum $name {
            $($label { $($field: $ty),* }),*
        }

        impl $name {
            fn new(s: &str) -> std::io::Result<$name> {
                $(
                    {
                        use $name::$label;
                        _parser_from_patt! {
                            _new_variant, $name, $label,
                            $patt, $suff, $($field: $ty),*
                        };

                        let res = _new_variant(s);
                        if res.is_ok() {
                            return res;
                        }
                    }
                )*;

                Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        concat!(
                            stringify!($name),
                            ": parse failed on all variants!")))
            }
        }
    }
}
