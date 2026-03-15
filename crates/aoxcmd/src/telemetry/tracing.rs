/// Basic tracing profile for node runtime diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceProfile {
    Minimal,
    Standard,
    Verbose,
}

impl TraceProfile {
    pub fn as_filter(self) -> &'static str {
        match self {
            Self::Minimal => "warn,aoxcmd=info",
            Self::Standard => "info,aoxcmd=debug,aoxcnet=debug",
            Self::Verbose => "trace",
        }
    }
}
