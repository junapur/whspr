use color_eyre::{eyre::OptionExt, eyre::Result, eyre::WrapErr, eyre::eyre};
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use tracing::{error, info, warn};

#[allow(dead_code)]
pub struct Model {
    pub name: &'static str,
    pub sha_256: &'static str,
    pub size_mb: f64,
}

#[allow(dead_code)]
pub fn get_model(name: &str) -> Option<&'static Model> {
    MODELS.iter().find(|model| model.name == name)
}

#[allow(dead_code)]
pub fn download_model(name: &str) -> Result<()> {
    let model = get_model(name).ok_or_eyre("Model not found")?;

    let model_dir = dirs::data_dir()
        .ok_or_eyre("Failed to get data directory")?
        .join("whspr/models");

    let model_path = model_dir.join(format!("{}.bin", model.name));

    fs::create_dir_all(&model_dir)
        .wrap_err_with(|| format!("Failed to create models dir at {}", model_path.display()))?;

    if model_path.exists() {
        if validate_hash(&model_path, model.sha_256)? {
            info!("Model already exists at {}", model_path.display());
            return Ok(());
        }

        warn!("Model already exists but has invalid hash, removing and re-downloading");
        fs::remove_file(&model_path)
            .wrap_err_with(|| format!("Failed to remove model at {}", model_path.display()))?;
    }

    let url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
        model.name
    );

    info!("Downloading model from {}", url);

    let client = Client::new();

    let mut response = client
        .get(&url)
        .send()
        .and_then(|r| r.error_for_status())
        .wrap_err_with(|| format!("Failed to download from {}", url))?;

    let mut file = File::create(&model_path)
        .wrap_err_with(|| format!("Failed to create file at {}", model_path.display()))?;

    io::copy(&mut response, &mut file)
        .wrap_err_with(|| format!("Failed to write to {}", model_path.display()))?;

    if !validate_hash(&model_path, model.sha_256)? {
        error!("Downloaded model has invalid hash, removing it");

        fs::remove_file(&model_path)
            .wrap_err_with(|| format!("Failed to remove model at {}", model_path.display()))?;

        return Err(eyre!("Downloaded model has invalid hash"));
    }

    info!("Model '{}' downloaded successfully!", name);
    Ok(())
}

#[allow(dead_code)]
fn validate_hash(file_path: impl AsRef<Path>, expected: &str) -> Result<bool> {
    let path = file_path.as_ref();

    let mut file =
        File::open(path).wrap_err_with(|| format!("Failed to open {}", path.display()))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .wrap_err_with(|| format!("Failed to read from {}", path.display()))?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let result = format!("{:x}", hasher.finalize());
    Ok(result.eq_ignore_ascii_case(expected))
}

#[allow(dead_code)]
pub const MODELS: [Model; 5] = [
    Model {
        name: "tiny",
        sha_256: "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
        size_mb: 77.7,
    },
    Model {
        name: "base",
        sha_256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
        size_mb: 148.0,
    },
    Model {
        name: "small",
        sha_256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
        size_mb: 488.0,
    },
    Model {
        name: "medium",
        sha_256: "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
        size_mb: 1530.0,
    },
    Model {
        name: "large-v3",
        sha_256: "64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2",
        size_mb: 3100.0,
    },
];
