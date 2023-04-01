#[derive(Debug)]
pub enum HttpErrorKind {
    InvalidDriver,
    InvalidHandle,
    MemoryAccessError,
    BufferTooSmall,
    HeaderNotFound,
    Utf8Error,
    DestinationNotAllowed,
    InvalidMethod,
    InvalidEncoding,
    InvalidUrl,
    RequestError,
    RuntimeError,
    TooManySessions,
    PermissionDeny,
}

impl std::error::Error for HttpErrorKind {}

impl std::fmt::Display for HttpErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::InvalidDriver => write!(f, "Invalid Driver"),
            Self::InvalidHandle => write!(f, "Invalid Error"),
            Self::MemoryAccessError => write!(f, "Memoery Access Error"),
            Self::BufferTooSmall => write!(f, "Buffer too small"),
            Self::HeaderNotFound => write!(f, "Header not found"),
            Self::Utf8Error => write!(f, "Utf8 error"),
            Self::DestinationNotAllowed => write!(f, "Destination not allowed"),
            Self::InvalidMethod => write!(f, "Invalid method"),
            Self::InvalidEncoding => write!(f, "Invalid encoding"),
            Self::InvalidUrl => write!(f, "Invalid url"),
            Self::RequestError => write!(f, "Request url"),
            Self::RuntimeError => write!(f, "Runtime error"),
            Self::TooManySessions => write!(f, "Too many sessions"),
            Self::PermissionDeny => write!(f, "Permision deny."),
        }
    }
}

impl From<u32> for HttpErrorKind {
    fn from(i: u32) -> HttpErrorKind {
        match i {
            1 => HttpErrorKind::InvalidHandle,
            2 => HttpErrorKind::MemoryAccessError,
            3 => HttpErrorKind::BufferTooSmall,
            4 => HttpErrorKind::HeaderNotFound,
            5 => HttpErrorKind::Utf8Error,
            6 => HttpErrorKind::DestinationNotAllowed,
            7 => HttpErrorKind::InvalidMethod,
            8 => HttpErrorKind::InvalidEncoding,
            9 => HttpErrorKind::InvalidUrl,
            10 => HttpErrorKind::RequestError,
            11 => HttpErrorKind::RuntimeError,
            12 => HttpErrorKind::TooManySessions,
            13 => HttpErrorKind::PermissionDeny,
            _ => HttpErrorKind::RuntimeError,
        }
    }
}
