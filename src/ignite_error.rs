use log::Level;
use std::convert::Into;
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
        IgniteErrorContents { message, cause }
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

/// Converts error to a string chaining all causes in a column
pub fn unwind_error(err: &dyn Error) -> String {
    match err.source() {
        None => format!("{}", err),
        Some(e) => format!("{}. Caused by: \n\t{}", err, unwind_error(e)),
    }
}

impl fmt::Display for IgniteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.err.message)
    }
}

pub type IgniteResult<T> = Result<T, IgniteError>;

/// Trait that intended to be implemented for Result
/// types, enabling automatic handling of errors.
///
/// For internal use only.
pub trait LogResult<R> {
    fn log_on_error<S: Into<String>>(self, lvl: Level, message: S) -> Option<R>;
    fn log_e_on_error<S: Into<String>>(self, message: S) -> Option<R>;
    fn log_w_on_error<S: Into<String>>(self, message: S) -> Option<R>;
    fn log_i_on_error<S: Into<String>>(self, message: S) -> Option<R>;
    fn log_d_on_error<S: Into<String>>(self, message: S) -> Option<R>;
}

impl<R, E> LogResult<R> for Result<R, E>
    where
        E: Error,
{
    fn log_on_error<S: Into<String>>(self, lvl: Level, message: S) -> Option<R> {
        match self {
            Ok(r) => Some(r),
            Err(ref e) => {
                log!(
                    lvl,
                    "{}. Caused by: \n\t{}",
                    message.into(),
                    unwind_error(e)
                );
                None
            }
        }
    }

    fn log_e_on_error<S: Into<String>>(self, message: S) -> Option<R> {
        self.log_on_error(Level::Error, message)
    }

    fn log_w_on_error<S: Into<String>>(self, message: S) -> Option<R> {
        self.log_on_error(Level::Warn, message)
    }

    fn log_i_on_error<S: Into<String>>(self, message: S) -> Option<R> {
        self.log_on_error(Level::Info, message)
    }

    fn log_d_on_error<S: Into<String>>(self, message: S) -> Option<R> {
        self.log_on_error(Level::Debug, message)
    }
}

/// Trait that intended to be implemented for Result types, allowing for
/// wrapping any results with IgniteResult.
/// For internal use only.
pub trait RewrapResult<R> {
    fn rewrap_on_error<S: Into<String>>(self, message: S) -> IgniteResult<R>;
}

impl<R, E> RewrapResult<R> for Result<R, E>
    where
        E: Error + 'static,
{
    /// FIXME: Can cause overhead on hot (Ok) route of execution. Consider using macros instead.
    fn rewrap_on_error<S: Into<String>>(self, message: S) -> IgniteResult<R> {
        match self {
            Ok(r) => Ok(r),
            Err(e) => Err(IgniteError::new_with_source(message, Box::new(e))),
        }
    }
}

/// Trait that intended to be implemented for Result types, allowing for
/// replacing any results with IgniteResult.
/// For internal use only.
pub trait ReplaceResult<R> {
    fn replace_on_error<S: Into<String>>(self, message: S) -> IgniteResult<R>;
}

impl<R, E> ReplaceResult<R> for Result<R, E>
    where
        E: Error,
{
    /// FIXME: Can cause overhead on hot (Ok) route of execution. Consider using macros instead.
    fn replace_on_error<S: Into<String>>(self, message: S) -> IgniteResult<R> {
        match self {
            Ok(r) => Ok(r),
            Err(_) => Err(IgniteError::new(message)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ignite_error::IgniteError;
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
