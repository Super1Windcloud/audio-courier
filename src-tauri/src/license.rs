use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use log::{info, warn};
use rand::distr::{Alphanumeric, SampleString};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const LICENSE_FILE_NAME: &str = "license.json";
const LICENSE_PUBLIC_KEY: &str = "93GQjRCWsE0ZeNB8yE67/Ryh/ZvUTbwVR1D0YgIE1uc=";

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActivationRequest {
    pub app_id: String,
    pub app_version: String,
    pub user_id: String,
    pub device_fingerprint: String,
    pub device_hint: String,
    pub request_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LicensePayload {
    pub license_id: String,
    pub user_id: String,
    pub device_fingerprint: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub max_version: String,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignedLicense {
    #[serde(flatten)]
    pub payload: LicensePayload,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LicenseStatus {
    pub is_activated: bool,
    pub is_valid: bool,
    pub is_host_signer: bool,
    pub reason: String,
    pub checked_at: DateTime<Utc>,
    pub user_id: Option<String>,
    pub license_id: Option<String>,
    pub issued_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_version: Option<String>,
    pub features: Vec<String>,
    pub current_version: String,
    pub device_hint: String,
    pub device_fingerprint: String,
    pub public_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignerStatus {
    pub is_configured: bool,
    pub is_allowed: bool,
    pub reason: String,
    pub public_key: Option<String>,
    pub current_device_fingerprint: Option<String>,
    pub current_device_hint: String,
}

pub fn build_activation_request(user_id: Option<String>) -> Result<ActivationRequest, String> {
    let request = ActivationRequest {
        app_id: env!("CARGO_PKG_NAME").to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        user_id: user_id
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "customer".to_string()),
        device_fingerprint: compute_device_fingerprint()?,
        device_hint: device_hint(),
        request_time: Utc::now(),
    };
    info!(
        "activation request built user_id={} device_hint={}",
        request.user_id, request.device_hint
    );
    Ok(request)
}

pub fn sign_license(
    request: ActivationRequest,
    user_id: String,
    expires_at: DateTime<Utc>,
    max_version: String,
    features: Vec<String>,
    private_key_base64: &str,
) -> Result<SignedLicense, String> {
    let signing_key = signing_key_from_base64(private_key_base64)?;
    let payload = LicensePayload {
        license_id: random_license_id(),
        user_id,
        device_fingerprint: request.device_fingerprint,
        issued_at: Utc::now(),
        expires_at,
        max_version,
        features,
    };
    let payload_bytes = serde_json::to_vec(&payload).map_err(|err| err.to_string())?;
    let signature = signing_key.sign(&payload_bytes);

    Ok(SignedLicense {
        payload,
        signature: STANDARD.encode(signature.to_bytes()),
    })
}

pub fn sign_license_from_request_json(
    raw_request: &str,
    user_id: String,
    expires_at: DateTime<Utc>,
    max_version: String,
    features: Vec<String>,
) -> Result<SignedLicense, String> {
    ensure_signer_access()?;
    let request: ActivationRequest = serde_json::from_str(raw_request)
        .map_err(|err| format!("激活请求 JSON 解析失败: {err}"))?;
    let private_key = private_key_from_env()?;
    info!(
        "signing license from request user_id={} device_hint={}",
        user_id, request.device_hint
    );
    sign_license(
        request,
        user_id,
        expires_at,
        max_version,
        features,
        &private_key,
    )
}

pub fn verify_license(license: &SignedLicense) -> Result<(), String> {
    let public_key = public_key_from_env()?;
    let payload_bytes = serde_json::to_vec(&license.payload).map_err(|err| err.to_string())?;
    let signature_bytes = STANDARD
        .decode(&license.signature)
        .map_err(|err| format!("许可证签名不是有效的 Base64: {err}"))?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|err| format!("许可证签名长度无效: {err}"))?;

    public_key
        .verify(&payload_bytes, &signature)
        .map_err(|_| "许可证签名校验失败，可能是客户端内置公钥与签发私钥不匹配".to_string())
}

pub fn evaluate_license(license: &SignedLicense) -> LicenseStatus {
    let device_fingerprint = compute_device_fingerprint().unwrap_or_else(|_| "unknown".to_string());
    let device_hint = device_hint();
    let checked_at = Utc::now();

    if let Err(err) = verify_license(license) {
        warn!(
            "license verify failed license_id={}: {}",
            license.payload.license_id, err
        );
        return invalid_status(err, checked_at, device_hint, device_fingerprint, license);
    }

    if license.payload.device_fingerprint != device_fingerprint {
        warn!(
            "license device mismatch license_id={} current_device_hint={}",
            license.payload.license_id, device_hint
        );
        return invalid_status(
            "许可证绑定的设备与当前机器不匹配".to_string(),
            checked_at,
            device_hint,
            device_fingerprint,
            license,
        );
    }

    if checked_at > license.payload.expires_at {
        warn!("license expired license_id={}", license.payload.license_id);
        return invalid_status(
            "许可证已过期".to_string(),
            checked_at,
            device_hint,
            device_fingerprint,
            license,
        );
    }

    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|err| err.to_string())
        .ok();
    let max_version = Version::parse(&license.payload.max_version)
        .map_err(|err| err.to_string())
        .ok();

    if let (Some(current), Some(max)) = (current_version, max_version) {
        if current > max {
            warn!(
                "license version out of range license_id={} current={} max={}",
                license.payload.license_id, current, max
            );
            return invalid_status(
                "当前软件版本超出许可证授权范围".to_string(),
                checked_at,
                device_hint,
                device_fingerprint,
                license,
            );
        }
    }

    LicenseStatus {
        is_activated: true,
        is_valid: true,
        is_host_signer: false,
        reason: "许可证有效".to_string(),
        checked_at,
        user_id: Some(license.payload.user_id.clone()),
        license_id: Some(license.payload.license_id.clone()),
        issued_at: Some(license.payload.issued_at),
        expires_at: Some(license.payload.expires_at),
        max_version: Some(license.payload.max_version.clone()),
        features: license.payload.features.clone(),
        current_version: env!("CARGO_PKG_VERSION").to_string(),
        device_hint,
        device_fingerprint,
        public_key: current_public_key(),
    }
}

pub fn load_license_status(app: &AppHandle) -> Result<LicenseStatus, String> {
    let license_path = license_file_path(app)?;
    let device_fingerprint = compute_device_fingerprint()?;
    let device_hint = device_hint();
    let checked_at = Utc::now();

    if ensure_signer_access().is_ok() {
        info!("load_license_status resolved to host signer");
        return Ok(LicenseStatus {
            is_activated: true,
            is_valid: true,
            is_host_signer: true,
            reason: "当前机器是签名宿主机，已跳过许可证校验".to_string(),
            checked_at,
            user_id: Some("signer-host".to_string()),
            license_id: Some("signer-host".to_string()),
            issued_at: None,
            expires_at: None,
            max_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            features: vec!["host-signer".to_string(), "pro".to_string()],
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            device_hint,
            device_fingerprint,
            public_key: current_public_key(),
        });
    }

    if !license_path.exists() {
        info!(
            "load_license_status no license file at {}",
            license_path.display()
        );
        return Ok(LicenseStatus {
            is_activated: false,
            is_valid: false,
            is_host_signer: false,
            reason: "未导入许可证".to_string(),
            checked_at,
            user_id: None,
            license_id: None,
            issued_at: None,
            expires_at: None,
            max_version: None,
            features: Vec::new(),
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            device_hint,
            device_fingerprint,
            public_key: current_public_key(),
        });
    }

    let content =
        fs::read_to_string(&license_path).map_err(|err| format!("读取许可证失败: {}", err))?;
    let license = parse_license(&content)?;
    info!(
        "load_license_status evaluating persisted license license_id={}",
        license.payload.license_id
    );
    Ok(evaluate_license(&license))
}

pub fn persist_license(app: &AppHandle, raw_license: &str) -> Result<LicenseStatus, String> {
    let license = parse_license(raw_license)?;
    info!(
        "persist_license received license_id={} user_id={}",
        license.payload.license_id, license.payload.user_id
    );
    let status = evaluate_license(&license);
    if !status.is_valid {
        warn!(
            "persist_license rejected license_id={}: {}",
            license.payload.license_id, status.reason
        );
        return Err(status.reason);
    }

    let license_path = license_file_path(app)?;
    if let Some(parent) = license_path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建许可证目录失败: {}", err))?;
    }

    fs::write(
        &license_path,
        serde_json::to_vec_pretty(&license).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("写入许可证失败: {}", err))?;

    info!("persist_license wrote {}", license_path.display());

    Ok(status)
}

pub fn license_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("获取应用数据目录失败: {}", err))?;
    Ok(app_dir.join(LICENSE_FILE_NAME))
}

pub fn parse_license(raw_license: &str) -> Result<SignedLicense, String> {
    serde_json::from_str(raw_license).map_err(|err| format!("许可证 JSON 解析失败: {}", err))
}

pub fn read_activation_request(path: &Path) -> Result<ActivationRequest, String> {
    let content = fs::read_to_string(path).map_err(|err| format!("读取激活请求失败: {}", err))?;
    serde_json::from_str(&content).map_err(|err| format!("激活请求 JSON 解析失败: {}", err))
}

pub fn write_signed_license(path: &Path, license: &SignedLicense) -> Result<(), String> {
    let content = serde_json::to_vec_pretty(license).map_err(|err| err.to_string())?;
    fs::write(path, content).map_err(|err| format!("写入许可证失败: {}", err))
}

pub fn public_key_from_env() -> Result<VerifyingKey, String> {
    let bytes = STANDARD
        .decode(LICENSE_PUBLIC_KEY)
        .map_err(|err| format!("LICENSE_PUBLIC_KEY 不是有效的 Base64: {err}"))?;
    let key_bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| "LICENSE_PUBLIC_KEY 必须是 32 字节公钥".to_string())?;

    VerifyingKey::from_bytes(&key_bytes).map_err(|err| format!("公钥无效: {err}"))
}

pub fn signer_status() -> SignerStatus {
    let public_key = current_public_key();
    let current_device_fingerprint = compute_device_fingerprint().ok();
    let current_device_hint = device_hint();
    let access_result = ensure_signer_access();
    if let Err(err) = &access_result {
        warn!("signer_status access denied: {}", err);
    }

    match (private_key_from_env(), access_result) {
        (Ok(_), Ok(_)) => SignerStatus {
            is_configured: true,
            is_allowed: true,
            reason: "当前机器允许打开签名器，且私钥已配置".to_string(),
            public_key,
            current_device_fingerprint,
            current_device_hint,
        },
        (Err(err), Ok(_)) => SignerStatus {
            is_configured: false,
            is_allowed: true,
            reason: err,
            public_key,
            current_device_fingerprint,
            current_device_hint,
        },
        (_, Err(err)) => SignerStatus {
            is_configured: false,
            is_allowed: false,
            reason: err,
            public_key,
            current_device_fingerprint,
            current_device_hint,
        },
    }
}

pub fn compute_device_fingerprint() -> Result<String, String> {
    let mut hasher = Sha256::new();
    for part in device_parts()? {
        hasher.update(part.as_bytes());
        hasher.update([0u8]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn device_hint() -> String {
    format!(
        "{} / {} / {}",
        env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown-host".to_string()),
        env::consts::OS,
        env::consts::ARCH
    )
}

fn invalid_status(
    reason: String,
    checked_at: DateTime<Utc>,
    device_hint: String,
    device_fingerprint: String,
    license: &SignedLicense,
) -> LicenseStatus {
    LicenseStatus {
        is_activated: true,
        is_valid: false,
        is_host_signer: false,
        reason,
        checked_at,
        user_id: Some(license.payload.user_id.clone()),
        license_id: Some(license.payload.license_id.clone()),
        issued_at: Some(license.payload.issued_at),
        expires_at: Some(license.payload.expires_at),
        max_version: Some(license.payload.max_version.clone()),
        features: license.payload.features.clone(),
        current_version: env!("CARGO_PKG_VERSION").to_string(),
        device_hint,
        device_fingerprint,
        public_key: current_public_key(),
    }
}

fn current_public_key() -> Option<String> {
    Some(LICENSE_PUBLIC_KEY.to_string())
}

fn device_parts() -> Result<Vec<String>, String> {
    let mut parts = vec![
        env::consts::OS.to_string(),
        env::consts::ARCH.to_string(),
        env::var("COMPUTERNAME").unwrap_or_default(),
        env::var("PROCESSOR_IDENTIFIER").unwrap_or_default(),
    ];

    #[cfg(windows)]
    {
        use winreg::RegKey;
        use winreg::enums::HKEY_LOCAL_MACHINE;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let crypto = hklm
            .open_subkey("SOFTWARE\\Microsoft\\Cryptography")
            .map_err(|err| format!("读取 MachineGuid 失败: {}", err))?;
        let machine_guid: String = crypto
            .get_value("MachineGuid")
            .map_err(|err| format!("读取 MachineGuid 失败: {}", err))?;
        parts.push(machine_guid);
    }

    if parts.iter().all(|part| part.trim().is_empty()) {
        return Err("无法生成设备指纹".to_string());
    }

    Ok(parts)
}

fn signing_key_from_base64(raw: &str) -> Result<SigningKey, String> {
    let bytes = STANDARD
        .decode(raw.trim())
        .map_err(|err| format!("私钥不是有效的 Base64: {err}"))?;
    let secret: [u8; 32] = bytes
        .try_into()
        .map_err(|_| "私钥必须是 32 字节种子".to_string())?;
    Ok(SigningKey::from_bytes(&secret))
}

fn private_key_from_env() -> Result<String, String> {
    let raw = env::var("LICENSE_PRIVATE_KEY")
        .map_err(|_| "未设置 LICENSE_PRIVATE_KEY，当前窗口不能签发许可证".to_string())?;
    if raw.trim().is_empty() {
        return Err("LICENSE_PRIVATE_KEY 为空，当前窗口不能签发许可证".to_string());
    }
    Ok(raw)
}

pub fn ensure_signer_access() -> Result<(), String> {
    let allowed_fingerprint = env::var("SIGNER_DEVICE_FINGERPRINT")
        .map_err(|_| "未设置 SIGNER_DEVICE_FINGERPRINT，签名器默认禁用".to_string())?;
    if allowed_fingerprint.trim().is_empty() {
        return Err("SIGNER_DEVICE_FINGERPRINT 为空，签名器默认禁用".to_string());
    }

    let current_fingerprint = compute_device_fingerprint()?;
    if allowed_fingerprint.trim() != current_fingerprint {
        warn!("signer access fingerprint mismatch");
        return Err("当前机器未被授权打开签名器".to_string());
    }

    info!("signer access granted");
    Ok(())
}

fn random_license_id() -> String {
    let suffix = Alphanumeric
        .sample_string(&mut rand::rng(), 10)
        .to_lowercase();
    format!("lic_{}", suffix)
}
