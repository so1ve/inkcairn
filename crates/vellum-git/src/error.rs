#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Utf8(std::string::FromUtf8Error),
    Exit {
        args: Vec<String>,
        code: Option<i32>,
        stderr: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "failed to run git: {e}"),
            Self::Utf8(e) => write!(f, "invalid git output: {e}"),
            Self::Exit { args, code, stderr } => {
                write!(
                    f,
                    "`git {}` failed ({code:?}): {}",
                    args.join(" "),
                    stderr.trim()
                )
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::Utf8(e)
    }
}
