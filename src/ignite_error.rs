use std::convert::{From, Into};
use std::error::Error;
use std::fmt;

/// We keep all the content here
#[derive(Debug)]
struct IgniteErrorContents {
    message: String,
    cause: Option<Box<dyn Error + 'static>>,
}

impl IgniteErrorContents {
    /// Make new instance
    fn new(message: String, cause: Option<Box<Error>>) -> IgniteErrorContents {
        IgniteErrorContents {
            message: message,
            cause: cause,
        }
    }
}

/// Ignite error
#[derive(Debug)]
pub struct IgniteError {
    err: Box<IgniteErrorContents>,
}

impl IgniteError {
    /// Create new IgniteError instance
    pub fn new<S: Into<String>>(message: S) -> IgniteError {
        IgniteError {
            err: Box::new(IgniteErrorContents::new(message.into(), None)),
        }
    }

    /// Create new IgniteError instance with cause
    pub fn new_with_source<S: Into<String>>(message: S, cause: Box<Error>) -> IgniteError {
        IgniteError {
            err: Box::new(IgniteErrorContents::new(message.into(), Some(cause))),
        }
    }
}

impl Error for IgniteError {
    /// Get error description.
    fn description(&self) -> &str {
        self.err.message.as_ref()
    }

    /// Get a source of the error if any.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.err.cause.as_ref().map(|bs| bs.as_ref())
    }
}

impl fmt::Display for IgniteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.err.message)
    }
}

impl<S> From<S> for IgniteError
where
    S: Into<String>,
{
    fn from(serr: S) -> Self {
        IgniteError::new(serr)
    }
}

pub type IgniteResult<T> = Result<T, IgniteError>;

/// Trait that intended to be implemented for Result
/// types, enabling automatical wrapping of errors into
/// IgniteResult.
///
/// FIXME: Can cause overhead on hot (Ok) route of execution.
/// FIXME: Consider using macros instead.
pub trait WrapOnError<R> {
    fn wrap_on_error<S: Into<String>>(self, message: S) -> IgniteResult<R>;
}

impl<R, E> WrapOnError<R> for Result<R, E>
where
    E: Error + 'static,
{
    fn wrap_on_error<S: Into<String>>(self, message: S) -> IgniteResult<R> {
        match self {
            Ok(r) => Ok(r),
            Err(e) => Err(IgniteError::new_with_source(message, Box::new(e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use ignite_error::IgniteError;
    use std::error::Error;

    static TEST_MSG: &str = "Test error";

    fn get_err() -> Result<(), IgniteError> {
        Err(IgniteError::new(TEST_MSG))
    }

    fn handling() -> Result<(), IgniteError> {
        get_err()?;
        Ok(())
    }

    fn source() -> Result<(), IgniteError> {
        let err = get_err().unwrap_err();
        Err(IgniteError::new_with_source(TEST_MSG, Box::new(err)))
    }

    #[test]
    fn error_to_string() {
        let err = get_err().expect_err("Error is expected");

        assert_eq!(err.to_string(), TEST_MSG);
    }

    #[test]
    fn error_handling() {
        let err = handling().expect_err("Error is expected");

        assert_eq!(err.to_string(), TEST_MSG);
    }

    #[test]
    fn error_source_self() {
        let err = source().expect_err("Error is expected");
        let err_nested = err.source().expect("Nested error expected");

        assert_eq!(err.to_string(), TEST_MSG);
        assert_eq!(err_nested.to_string(), TEST_MSG);
    }
}
