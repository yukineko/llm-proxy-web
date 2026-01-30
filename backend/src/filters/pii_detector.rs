use regex::Regex;
use once_cell::sync::Lazy;
use std::collections::HashMap;

static COMPANY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:株式会社|有限会社|合同会社|一般社団法人|一般財団法人)[\p{Hiragana}\p{Katakana}\p{Han}ー・a-zA-Z0-9]+|[\p{Hiragana}\p{Katakana}\p{Han}ー・a-zA-Z0-9]+(?:株式会社|有限会社|合同会社|Corp\.|Inc\.|Ltd\.|LLC|Co\.)").unwrap()
});

static PERSON_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // 日本人名のパターン（姓名の組み合わせ）
    Regex::new(r"[\p{Han}]{1,4}[\s　][\p{Han}]{1,4}").unwrap()
});

static ADDRESS_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:東京都|北海道|(?:京都|大阪)府|[\p{Han}]{2,3}県)[\p{Han}\p{Hiragana}\p{Katakana}0-9ー・\s　-]+(?:市|区|町|村)[\p{Han}\p{Hiragana}\p{Katakana}0-9ー・\s　-]*").unwrap()
});

static EMAIL_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap()
});

static PHONE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:0\d{1,4}-\d{1,4}-\d{4}|\d{3}-\d{4}-\d{4})").unwrap()
});

#[derive(Debug, Clone)]
pub struct PIIDetector {
    company_counter: usize,
    person_counter: usize,
    address_counter: usize,
    email_counter: usize,
    phone_counter: usize,
}

impl PIIDetector {
    pub fn new() -> Self {
        Self {
            company_counter: 0,
            person_counter: 0,
            address_counter: 0,
            email_counter: 0,
            phone_counter: 0,
        }
    }

    pub fn detect_and_mask(&mut self, text: &str) -> (String, HashMap<String, String>) {
        let mut masked_text = text.to_string();
        let mut mappings = HashMap::new();

        // 会社名をマスク
        for cap in COMPANY_PATTERN.find_iter(text) {
            let company_name = cap.as_str();
            if !masked_text.contains(company_name) {
                continue;
            }
            self.company_counter += 1;
            let mask = format!("[COMPANY_{}]", self.company_counter);
            masked_text = masked_text.replace(company_name, &mask);
            mappings.insert(mask, company_name.to_string());
        }

        // メールアドレスをマスク
        for cap in EMAIL_PATTERN.find_iter(text) {
            let email = cap.as_str();
            if !masked_text.contains(email) {
                continue;
            }
            self.email_counter += 1;
            let mask = format!("[EMAIL_{}]", self.email_counter);
            masked_text = masked_text.replace(email, &mask);
            mappings.insert(mask, email.to_string());
        }

        // 電話番号をマスク
        for cap in PHONE_PATTERN.find_iter(text) {
            let phone = cap.as_str();
            if !masked_text.contains(phone) {
                continue;
            }
            self.phone_counter += 1;
            let mask = format!("[PHONE_{}]", self.phone_counter);
            masked_text = masked_text.replace(phone, &mask);
            mappings.insert(mask, phone.to_string());
        }

        // 人名をマスク
        for cap in PERSON_PATTERN.find_iter(text) {
            let person_name = cap.as_str();
            if !masked_text.contains(person_name) {
                continue;
            }
            self.person_counter += 1;
            let mask = format!("[PERSON_{}]", self.person_counter);
            masked_text = masked_text.replace(person_name, &mask);
            mappings.insert(mask, person_name.to_string());
        }

        // 住所をマスク
        for cap in ADDRESS_PATTERN.find_iter(text) {
            let address = cap.as_str();
            if !masked_text.contains(address) {
                continue;
            }
            self.address_counter += 1;
            let mask = format!("[ADDRESS_{}]", self.address_counter);
            masked_text = masked_text.replace(address, &mask);
            mappings.insert(mask, address.to_string());
        }

        (masked_text, mappings)
    }

    pub fn unmask(&self, text: &str, mappings: &HashMap<String, String>) -> String {
        let mut unmasked_text = text.to_string();
        
        for (mask, original) in mappings.iter() {
            unmasked_text = unmasked_text.replace(mask, original);
        }
        
        unmasked_text
    }
}

impl Default for PIIDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_company_detection() {
        let mut detector = PIIDetector::new();
        let text = "株式会社サンプル商事とトヨタ自動車株式会社が契約しました。";
        let (masked, mappings) = detector.detect_and_mask(text);
        
        assert!(masked.contains("[COMPANY_1]"));
        assert!(masked.contains("[COMPANY_2]"));
        assert_eq!(mappings.len(), 2);
    }

    #[test]
    fn test_person_detection() {
        let mut detector = PIIDetector::new();
        let text = "山田 太郎さんと佐藤 花子さんが来ました。";
        let (masked, mappings) = detector.detect_and_mask(text);
        
        assert!(masked.contains("[PERSON_1]"));
        assert!(masked.contains("[PERSON_2]"));
    }

    #[test]
    fn test_address_detection() {
        let mut detector = PIIDetector::new();
        let text = "東京都渋谷区桜丘町1-1にあります。";
        let (masked, _) = detector.detect_and_mask(text);
        
        assert!(masked.contains("[ADDRESS_1]"));
    }

    #[test]
    fn test_email_detection() {
        let mut detector = PIIDetector::new();
        let text = "連絡先はtest@example.comです。";
        let (masked, mappings) = detector.detect_and_mask(text);
        
        assert!(masked.contains("[EMAIL_1]"));
        assert_eq!(mappings.get("[EMAIL_1]"), Some(&"test@example.com".to_string()));
    }

    #[test]
    fn test_unmask() {
        let detector = PIIDetector::new();
        let mut mappings = HashMap::new();
        mappings.insert("[COMPANY_1]".to_string(), "株式会社テスト".to_string());
        
        let masked = "[COMPANY_1]に連絡します。";
        let unmasked = detector.unmask(masked, &mappings);
        
        assert_eq!(unmasked, "株式会社テストに連絡します。");
    }
}
