/// CPU capability profile used to enable optional cryptographic fast paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuCapabilities {
    pub aes_ni: bool,
    pub avx2: bool,
    pub avx512f: bool,
}

impl CpuCapabilities {
    /// Detect supported CPU flags using compile/runtime hints.
    pub fn detect() -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            Self {
                aes_ni: std::is_x86_feature_detected!("aes"),
                avx2: std::is_x86_feature_detected!("avx2"),
                avx512f: std::is_x86_feature_detected!("avx512f"),
            }
        }

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            Self {
                aes_ni: false,
                avx2: false,
                avx512f: false,
            }
        }
    }

    /// Choose a deterministic profile name for logging and policy checks.
    pub fn profile_name(self) -> &'static str {
        match (self.aes_ni, self.avx2, self.avx512f) {
            (true, true, true) => "aes-ni+avx2+avx512",
            (true, true, false) => "aes-ni+avx2",
            (true, false, _) => "aes-ni",
            _ => "portable",
        }
    }
}
