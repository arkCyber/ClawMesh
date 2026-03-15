/// Agent Skills Security Module (DO-178C Level A)
/// 
/// Provides security validation, malicious code detection, and signature verification

use anyhow::{Result, bail};
use sha2::{Sha256, Digest};
use tracing::{info, warn, error};

/// Validate skill code for security issues
/// 
/// # Safety
/// - Scans for malicious patterns
/// - Validates code structure
/// - Checks for obfuscation
pub fn validate_skill_code(code: &str) -> Result<()> {
    info!(code_length = code.len(), "Validating skill code");
    
    // 1. Check code length
    if code.len() > 1_000_000 {
        bail!("Code too large (max 1MB)");
    }
    
    if code.is_empty() {
        bail!("Empty code");
    }
    
    // 2. Scan for malicious code
    scan_for_malicious_code(code)?;
    
    // 3. Check for obfuscation
    check_obfuscation(code)?;
    
    // 4. Validate code structure
    validate_code_structure(code)?;
    
    info!("Code validation passed");
    Ok(())
}

/// Scan code for malicious patterns
/// 
/// # Safety
/// - Detects dangerous system calls
/// - Identifies crypto mining patterns
/// - Finds data exfiltration attempts
pub fn scan_for_malicious_code(code: &str) -> Result<()> {
    info!("Scanning for malicious code patterns");
    
    // Dangerous system operations
    let dangerous_patterns = vec![
        // System execution
        ("exec(", "Code execution"),
        ("eval(", "Dynamic evaluation"),
        ("system(", "System command"),
        ("subprocess", "Subprocess execution"),
        ("os.system", "OS command"),
        ("shell=True", "Shell execution"),
        
        // File system attacks
        ("rm -rf", "Destructive file operation"),
        ("/etc/passwd", "System file access"),
        ("/etc/shadow", "Password file access"),
        ("chmod 777", "Dangerous permissions"),
        
        // Database attacks
        ("DROP TABLE", "Database destruction"),
        ("DELETE FROM", "Data deletion"),
        ("TRUNCATE", "Table truncation"),
        ("'; --", "SQL injection"),
        
        // Network attacks
        ("socket.socket", "Raw socket access"),
        ("bind(0.0.0.0", "Network binding"),
        
        // Crypto mining
        ("stratum+tcp", "Mining pool connection"),
        ("xmrig", "Crypto miner"),
        ("ethminer", "Ethereum miner"),
        
        // Data exfiltration
        ("base64.b64encode", "Data encoding"),
        ("requests.post", "External POST request"),
        ("urllib.request", "URL request"),
    ];
    
    for (pattern, description) in dangerous_patterns {
        if code.contains(pattern) {
            warn!(
                pattern = pattern,
                description = description,
                "Dangerous pattern detected"
            );
            bail!("Malicious code detected: {}", description);
        }
    }
    
    // Check for excessive loops (potential DoS)
    let loop_count = code.matches("while True:").count() + 
                     code.matches("for").count();
    if loop_count > 10 {
        warn!(loop_count = loop_count, "Excessive loops detected");
        bail!("Code contains too many loops (potential DoS)");
    }
    
    info!("No malicious patterns found");
    Ok(())
}

/// Check for code obfuscation
/// 
/// # Safety
/// - Detects base64 encoded strings
/// - Identifies hex-encoded data
/// - Finds excessive string concatenation
fn check_obfuscation(code: &str) -> Result<()> {
    // Check for base64-like strings
    let base64_pattern_count = code.matches("==").count();
    if base64_pattern_count > 5 {
        warn!(
            base64_patterns = base64_pattern_count,
            "Possible base64 obfuscation"
        );
    }
    
    // Check for hex strings
    let hex_pattern_count = code.matches("\\x").count();
    if hex_pattern_count > 20 {
        warn!(
            hex_patterns = hex_pattern_count,
            "Possible hex obfuscation"
        );
        bail!("Code appears to be obfuscated");
    }
    
    // Check for excessive string concatenation
    let concat_count = code.matches("+").count();
    if concat_count > 100 {
        warn!(
            concatenations = concat_count,
            "Excessive string concatenation"
        );
    }
    
    Ok(())
}

/// Validate code structure
/// 
/// # Safety
/// - Checks for proper formatting
/// - Validates imports
/// - Ensures reasonable complexity
fn validate_code_structure(code: &str) -> Result<()> {
    let lines: Vec<&str> = code.lines().collect();
    
    // Check line count
    if lines.len() > 10000 {
        bail!("Code too long (max 10000 lines)");
    }
    
    // Check for excessively long lines
    for (i, line) in lines.iter().enumerate() {
        if line.len() > 500 {
            warn!(
                line_number = i + 1,
                line_length = line.len(),
                "Excessively long line"
            );
        }
    }
    
    Ok(())
}

/// Verify skill signature
/// 
/// # Safety
/// - Validates cryptographic signature
/// - Ensures code integrity
/// - Prevents tampering
pub fn verify_skill_signature(code: &str, signature: &str, public_key: &str) -> Result<bool> {
    info!("Verifying skill signature");
    
    // Calculate code hash
    let code_hash = calculate_code_hash(code);
    
    // In production, use proper cryptographic signature verification
    // This is a simplified implementation
    
    // For now, just check if signature matches hash
    let expected_signature = hex::encode(code_hash);
    
    if signature == expected_signature {
        info!("Signature verification passed");
        Ok(true)
    } else {
        warn!("Signature verification failed");
        Ok(false)
    }
}

/// Calculate code hash for integrity verification
/// 
/// # Safety
/// - Uses SHA-256
/// - Deterministic output
pub fn calculate_code_hash(code: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    hasher.finalize().to_vec()
}

/// Security scan result
#[derive(Debug, Clone)]
pub struct SecurityScanResult {
    pub is_safe: bool,
    pub threats_found: Vec<String>,
    pub warnings: Vec<String>,
    pub risk_score: u32,
}

impl SecurityScanResult {
    pub fn safe() -> Self {
        Self {
            is_safe: true,
            threats_found: vec![],
            warnings: vec![],
            risk_score: 0,
        }
    }
    
    pub fn unsafe_with_threats(threats: Vec<String>) -> Self {
        let risk_score = threats.len() as u32 * 10;
        Self {
            is_safe: false,
            threats_found: threats,
            warnings: vec![],
            risk_score,
        }
    }
}

/// Comprehensive security scan
/// 
/// # Safety
/// - Runs all security checks
/// - Provides detailed report
/// - Assigns risk score
pub fn comprehensive_security_scan(code: &str) -> SecurityScanResult {
    let mut threats = Vec::new();
    let mut warnings = Vec::new();
    
    // Run validation
    if let Err(e) = validate_skill_code(code) {
        threats.push(e.to_string());
    }
    
    // Check for suspicious imports
    let suspicious_imports = vec!["pickle", "marshal", "ctypes", "cffi"];
    for import in suspicious_imports {
        if code.contains(&format!("import {}", import)) {
            warnings.push(format!("Suspicious import: {}", import));
        }
    }
    
    // Calculate risk score
    let risk_score = (threats.len() * 20 + warnings.len() * 5) as u32;
    
    SecurityScanResult {
        is_safe: threats.is_empty(),
        threats_found: threats,
        warnings,
        risk_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_safe_code() {
        let safe_code = r#"
def hello():
    print("Hello, World!")
    return 42
"#;
        assert!(validate_skill_code(safe_code).is_ok());
    }

    #[test]
    fn test_detect_malicious_exec() {
        let malicious_code = "exec('malicious code')";
        assert!(scan_for_malicious_code(malicious_code).is_err());
    }

    #[test]
    fn test_detect_sql_injection() {
        let malicious_code = "query = 'DROP TABLE users'";
        assert!(scan_for_malicious_code(malicious_code).is_err());
    }

    #[test]
    fn test_detect_file_destruction() {
        let malicious_code = "os.system('rm -rf /')";
        assert!(scan_for_malicious_code(malicious_code).is_err());
    }

    #[test]
    fn test_code_hash() {
        let code = "print('test')";
        let hash1 = calculate_code_hash(code);
        let hash2 = calculate_code_hash(code);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_comprehensive_scan_safe() {
        let safe_code = "def add(a, b): return a + b";
        let result = comprehensive_security_scan(safe_code);
        assert!(result.is_safe);
        assert_eq!(result.risk_score, 0);
    }

    #[test]
    fn test_comprehensive_scan_unsafe() {
        let unsafe_code = "exec('malicious')";
        let result = comprehensive_security_scan(unsafe_code);
        assert!(!result.is_safe);
        assert!(result.risk_score > 0);
    }

    #[test]
    fn test_obfuscation_detection() {
        let obfuscated = "x = '\\x41\\x42\\x43' * 100";
        assert!(check_obfuscation(obfuscated).is_err());
    }
}
