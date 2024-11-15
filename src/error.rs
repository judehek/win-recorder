use thiserror::Error;
use windows::core;
use windows::core::Error as WindowsError;
use windows::core::HRESULT;
use windows::core::HSTRING;

pub type Result<T> = std::result::Result<T, RecorderError>;

#[derive(Debug, Error)]
pub enum RecorderError {
    #[error("Windows API error: {0}")]
    Windows(#[from] core::Error),

    #[error("Generic Error: {0}")]
    Generic(String),

    #[error("Failed to Start the Recording Process, reason: {0}")]
    FailedToStart(String),

    #[error("Failed to Stop the Recording Process")]
    FailedToStop,

    #[error("Called to Stop when there is no Recorder Configured")]
    NoRecorderBound,

    #[error("Called to Stop when the Recorder is Already Stopped")]
    RecorderAlreadyStopped,

    #[error("No Process Specified for the Recorder")]
    NoProcessSpecified,

    #[error("Logger error: {0}")]
    LoggerError(String),

    #[error("No compatible hardware encoders found on the system")]
    NoEncodersFound,

    #[error("Failed to enumerate hardware encoders: {0}")]
    EncoderEnumerationFailed(String),

    #[error("Hardware encoder incompatibility: {0}")]
    EncoderIncompatibility(String),

    #[error("Failed to initialize hardware encoder: {0}")]
    EncoderInitializationFailed(String),

    #[error("No encoder selected for recording")]
    NoEncoderSelected,

    #[error("Unsupported encoder: {0}")]
    UnsupportedEncoder(String),
}

impl From<RecorderError> for WindowsError {
    fn from(err: RecorderError) -> Self {
        match err {
            // For Windows errors, pass through the original error
            RecorderError::Windows(e) => e,

            // Map specific encoder errors to appropriate HRESULTs
            RecorderError::NoEncodersFound => WindowsError::new(
                HRESULT(-2147220981), // MF_E_NO_CAPTURE_DEVICES_AVAILABLE
                HSTRING::from(err.to_string()),
            ),
            RecorderError::EncoderEnumerationFailed(_) => WindowsError::new(
                HRESULT(-2147220992), // MF_E_TRANSFORM_TYPE_NOT_SET
                HSTRING::from(err.to_string()),
            ),
            RecorderError::EncoderIncompatibility(_) => WindowsError::new(
                HRESULT(-2147220969), // MF_E_UNSUPPORTED_FORMAT
                HSTRING::from(err.to_string()),
            ),
            RecorderError::EncoderInitializationFailed(_) => WindowsError::new(
                HRESULT(-2147220991), // MF_E_TRANSFORM_CANNOT_CHANGE_MEDIATYPE_WHILE_PROCESSING
                HSTRING::from(err.to_string()),
            ),
            RecorderError::NoEncoderSelected => WindowsError::new(
                HRESULT(-2147220992), // MF_E_TRANSFORM_TYPE_NOT_SET
                HSTRING::from(err.to_string()),
            ),
            RecorderError::UnsupportedEncoder(_) => WindowsError::new(
                HRESULT(-2147220969), // MF_E_UNSUPPORTED_FORMAT
                HSTRING::from(err.to_string()),
            ),

            // For other errors, use E_FAIL as a generic error code
            _ => WindowsError::new(
                HRESULT(-2147467259), // 0x80004005 (E_FAIL)
                HSTRING::from(err.to_string()),
            ),
        }
    }
}
