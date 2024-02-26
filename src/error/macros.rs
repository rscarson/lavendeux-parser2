#[macro_export]
macro_rules! error_matches {
    ($err:expr, $pat:ident) => {
        matches!(($err).details, crate::error::ErrorDetails::$pat { .. })
    };
}

macro_rules! oops {
    ($variant:ident, token = $context:expr, src = $src:expr) => {
        Err(crate::error::Error {
            details: crate::error::ErrorDetails::$variant,
            context: Some($context),
            source: Some(Box::new($src)),
        })
    };
    ($variant:ident, $context:expr) => {
        Err(crate::error::Error {
            details: crate::error::ErrorDetails::$variant,
            context: Some($context),
            source: None,
        })
    };
    ($variant:ident, src = $src:expr) => {
        Err(crate::error::Error {
            details: crate::error::ErrorDetails::$variant,
            context: None,
            source: Some(Box::new($src)),
        })
    };
    ($variant:ident) => {
        Err(crate::error::Error {
            details: crate::error::ErrorDetails::$variant,
            context: None,
            source: None,
        })
    };

    ($variant:ident { $($n:ident:$v:expr),+ }, token = $context:expr, src = $src:expr) => {
        Err(crate::error::Error {
            details: crate::error::ErrorDetails::$variant { $($n: $v),+ },
            context: Some($context),
            source: Some(Box::new($src)),
        })
    };
    ($variant:ident { $($n:ident:$v:expr),+ }, $context:expr) => {
        Err(crate::error::Error {
            details: crate::error::ErrorDetails::$variant { $($n: $v),+ },
            context: Some($context),
            source: None,
        })
    };
    ($variant:ident { $($n:ident:$v:expr),+ }, src = $src:expr) => {
        Err(crate::error::Error {
            details: crate::error::ErrorDetails::$variant { $($n: $v),+ },
            context: None,
            source: Some(Box::new($src)),
        })
    };
    ($variant:ident { $($n:ident:$v:expr),+ }) => {
        Err(crate::error::Error {
            details: crate::error::ErrorDetails::$variant { $($n: $v),+ },
            context: None,
            source: None,
        })
    };
}
