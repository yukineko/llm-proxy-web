use regex::Regex;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use fake::Fake;
use fake::faker::name::raw::*;
use fake::faker::company::raw::*;
use fake::faker::phone_number::raw::*;
use fake::faker::internet::raw::*;
use fake::locales::JA_JP;
use rand::rngs::SmallRng;
use rand::SeedableRng;

static COMPANY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:株式会社|有限会社|合同会社|一般社団法人|一般財団法人)[\p{Hiragana}\p{Katakana}\p{Han}ー・a-zA-Z0-9]+|[\p{Hiragana}\p{Katakana}\p{Han}ー・a-zA-Z0-9]+(?:株式会社|有限会社|合同会社|Corp\.|Inc\.|Ltd\.|LLC|Co\.)").unwrap()
});

static PERSON_PATTERN: Lazy<Regex> = Lazy::new(|| {
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

// 住所はfakeクレートに日本語実装がないため自前プール
const FAKE_ADDRESSES: &[&str] = &[
    "東京都千代田区霞が関1-1-1",
    "大阪府大阪市北区空町2-2-2",
    "神奈川県横浜市西区星川3-3-3",
    "愛知県名古屋市中区月見4-4-4",
    "福岡県福岡市博多区風花5-5-5",
    "北海道札幌市中央区雪原6-6-6",
    "京都府京都市左京区花園7-7-7",
    "兵庫県神戸市中央区潮風8-8-8",
    "広島県広島市中区朝日9-9-9",
    "宮城県仙台市青葉区若葉10-10-10",
    "埼玉県さいたま市大宮区星空11-11-11",
    "千葉県千葉市中央区虹色12-12-12",
    "静岡県静岡市葵区清風13-13-13",
    "新潟県新潟市中央区白雲14-14-14",
    "岡山県岡山市北区桃園15-15-15",
];

#[derive(Debug)]
pub struct PIIDetector {
    rng: SmallRng,
    address_counter: usize,
}

impl PIIDetector {
    pub fn new() -> Self {
        Self {
            rng: SmallRng::from_os_rng(),
            address_counter: 0,
        }
    }

    fn gen_fake_company(&mut self) -> String {
        CompanyName(JA_JP).fake_with_rng(&mut self.rng)
    }

    fn gen_fake_person(&mut self) -> String {
        Name(JA_JP).fake_with_rng(&mut self.rng)
    }

    fn gen_fake_email(&mut self) -> String {
        FreeEmail(JA_JP).fake_with_rng(&mut self.rng)
    }

    fn gen_fake_phone(&mut self) -> String {
        PhoneNumber(JA_JP).fake_with_rng(&mut self.rng)
    }

    fn gen_fake_address(&mut self) -> String {
        let addr = FAKE_ADDRESSES[self.address_counter % FAKE_ADDRESSES.len()];
        self.address_counter += 1;
        addr.to_string()
    }

    /// テキスト中のPIIを架空の固有名詞に置換する。
    /// 返り値: (置換済みテキスト, 架空→実名のマッピング)
    pub fn detect_and_mask(&mut self, text: &str) -> (String, HashMap<String, String>) {
        let mut masked_text = text.to_string();
        let mut mappings = HashMap::new();

        // 会社名
        for cap in COMPANY_PATTERN.find_iter(text) {
            let real = cap.as_str();
            if !masked_text.contains(real) {
                continue;
            }
            let fake = self.gen_fake_company();
            masked_text = masked_text.replace(real, &fake);
            mappings.insert(fake, real.to_string());
        }

        // メールアドレス
        for cap in EMAIL_PATTERN.find_iter(text) {
            let real = cap.as_str();
            if !masked_text.contains(real) {
                continue;
            }
            let fake = self.gen_fake_email();
            masked_text = masked_text.replace(real, &fake);
            mappings.insert(fake, real.to_string());
        }

        // 電話番号
        for cap in PHONE_PATTERN.find_iter(text) {
            let real = cap.as_str();
            if !masked_text.contains(real) {
                continue;
            }
            let fake = self.gen_fake_phone();
            masked_text = masked_text.replace(real, &fake);
            mappings.insert(fake, real.to_string());
        }

        // 人名
        for cap in PERSON_PATTERN.find_iter(text) {
            let real = cap.as_str();
            if !masked_text.contains(real) {
                continue;
            }
            let fake = self.gen_fake_person();
            masked_text = masked_text.replace(real, &fake);
            mappings.insert(fake, real.to_string());
        }

        // 住所
        for cap in ADDRESS_PATTERN.find_iter(text) {
            let real = cap.as_str();
            if !masked_text.contains(real) {
                continue;
            }
            let fake = self.gen_fake_address();
            masked_text = masked_text.replace(real, &fake);
            mappings.insert(fake, real.to_string());
        }

        (masked_text, mappings)
    }

    /// 架空名を実名に復元する
    pub fn unmask(&self, text: &str, mappings: &HashMap<String, String>) -> String {
        let mut unmasked_text = text.to_string();
        for (fake, real) in mappings.iter() {
            unmasked_text = unmasked_text.replace(fake, real);
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

        assert!(!masked.contains("サンプル商事"));
        assert!(!masked.contains("トヨタ自動車"));
        assert_eq!(mappings.len(), 2);
        let unmasked = detector.unmask(&masked, &mappings);
        assert!(unmasked.contains("株式会社サンプル商事"));
        assert!(unmasked.contains("トヨタ自動車株式会社"));
    }

    #[test]
    fn test_person_detection() {
        let mut detector = PIIDetector::new();
        let text = "山田 太郎さんと佐藤 花子さんが来ました。";
        let (masked, mappings) = detector.detect_and_mask(text);

        assert!(!masked.contains("山田 太郎"));
        assert!(!masked.contains("佐藤 花子"));
        let unmasked = detector.unmask(&masked, &mappings);
        assert!(unmasked.contains("山田 太郎"));
        assert!(unmasked.contains("佐藤 花子"));
    }

    #[test]
    fn test_roundtrip() {
        let mut detector = PIIDetector::new();
        let original = "株式会社テストの山田 太郎（yamada@test.co.jp、03-1234-5678）は東京都渋谷区桜丘町1-1にいます。";
        let (masked, mappings) = detector.detect_and_mask(original);

        assert!(!masked.contains("株式会社テスト"));
        assert!(!masked.contains("山田 太郎"));
        assert!(!masked.contains("yamada@test.co.jp"));
        assert!(!masked.contains("03-1234-5678"));

        let restored = detector.unmask(&masked, &mappings);
        assert!(restored.contains("株式会社テスト"));
        assert!(restored.contains("山田 太郎"));
        assert!(restored.contains("yamada@test.co.jp"));
        assert!(restored.contains("03-1234-5678"));
    }

    #[test]
    fn test_each_call_generates_different_fakes() {
        let mut detector = PIIDetector::new();
        let (masked1, _) = detector.detect_and_mask("株式会社テスト");
        let (masked2, _) = detector.detect_and_mask("株式会社テスト");
        // ランダムなので毎回異なる架空名
        assert_ne!(masked1, masked2);
    }
}
