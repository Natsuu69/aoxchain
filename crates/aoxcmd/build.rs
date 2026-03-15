use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
    println!("cargo:rerun-if-env-changed=AOXC_EMBED_CERT_PATH");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");

    emit_env(
        "AOXC_BUILD_SEMVER",
        &env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into()),
    );
    emit_env(
        "AOXC_BUILD_SOURCE_DATE_EPOCH",
        &env::var("SOURCE_DATE_EPOCH").unwrap_or_else(|_| "not-set".into()),
    );

    let git_commit =
        git_output(&["rev-parse", "--short=12", "HEAD"]).unwrap_or_else(|| "unknown".into());
    emit_env("AOXC_BUILD_GIT_COMMIT", &git_commit);

    let dirty = git_output(&["status", "--porcelain"])
        .map(|value| {
            if value.trim().is_empty() {
                "false"
            } else {
                "true"
            }
            .to_string()
        })
        .unwrap_or_else(|| "unknown".into());
    emit_env("AOXC_BUILD_GIT_DIRTY", &dirty);

    match env::var("AOXC_EMBED_CERT_PATH") {
        Ok(path) => {
            println!("cargo:rerun-if-changed={path}");
            match fs::read(&path) {
                Ok(bytes) => {
                    let digest = Sha256::digest(bytes);
                    emit_env("AOXC_BUILD_CERT_SHA256", &hex::encode(digest));
                    emit_env("AOXC_BUILD_CERT_PATH", &path);
                }
                Err(error) => {
                    emit_env("AOXC_BUILD_CERT_SHA256", "unavailable");
                    emit_env("AOXC_BUILD_CERT_PATH", &path);
                    emit_env("AOXC_BUILD_CERT_ERROR", &error.to_string());
                }
            }
        }
        Err(_) => {
            emit_env("AOXC_BUILD_CERT_SHA256", "not-configured");
            emit_env("AOXC_BUILD_CERT_PATH", "not-configured");
        }
    }
}

fn emit_env(key: &str, value: &str) {
    println!("cargo:rustc-env={key}={value}");
}

fn git_output(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}
