const PATTERN_SYSTEM: &str = "[//]: # (SYSTEM)\n";
const PATTERN_USER: &str = "[//]: # (USER)\n";

fn cut_meta(content: String) -> String {
    let indices: Vec<(usize, &str)> = content.match_indices("---").collect();
    if indices.len() < 2 {
        return content;
    }
    if indices.first().unwrap().1 != "---" {
        return content;
    }
    if indices.get(1).unwrap().1 != "---" {
        return content;
    }
    let start = indices.get(1).unwrap().0;
    content[start + 4..].to_string()
}

pub fn parse_content(content: String) -> (String, String) {
    let content = cut_meta(content);
    let system_idx = content.find(PATTERN_SYSTEM);
    let user_idx = content.find(PATTERN_USER);

    match (system_idx, user_idx) {
        (Some(sys), None) => {
            let system = content[sys + PATTERN_SYSTEM.len()..content.len()].to_string();
            (system.trim().to_string(), "".to_string())
        }
        (Some(sys), Some(usr)) => {
            if usr < sys {
                let user = content[usr + PATTERN_USER.len()..sys].to_string();
                let system = content[sys + PATTERN_SYSTEM.len()..content.len()].to_string();
                return (system.trim().to_string(), user.trim().to_string());
            }
            // Need to check if usr > sys
            let system = content[sys + PATTERN_SYSTEM.len()..usr].to_string();
            let user = content[usr + PATTERN_USER.len()..content.len()].to_string();
            (system.trim().to_string(), user.trim().to_string())
        }
        (None, Some(usr)) => {
            let user = content[usr + PATTERN_USER.len()..content.len()].to_string();
            ("".to_string(), user.trim().to_string())
        }
        _ => (content.trim().to_string(), "".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_content() {
        let (system, user) = parse_content("Check\n[//]: # (SYSTEM)\nSystem content".to_string());
        assert_eq!(system, "System content");
        assert_eq!(user, "");

        let (system, user) = parse_content("User content".to_string());
        assert_eq!(system, "User content");
        assert_eq!(user, "");

        let (system, user) = parse_content("[//]: # (USER)\nUser content".to_string());
        assert_eq!(user, "User content");
        assert_eq!(system, "");

        let (system, user) = parse_content(
            "Check\n[//]: # (SYSTEM)\nSystem content \n\n [//]: # (USER)\nUser content sdf"
                .to_string(),
        );
        assert_eq!(system, "System content");
        assert_eq!(user, "User content sdf");

        let (system, user) = parse_content(" Check ".to_string());
        assert_eq!(system, "Check");
        assert_eq!(user, "");

        let (system, user) = parse_content(
            "Check\n[//]: # (USER)\nUser content sdf[//]: # (SYSTEM)\nSystem content \n\n "
                .to_string(),
        );
        assert_eq!(system, "System content");
        assert_eq!(user, "User content sdf");
    }

    #[test]
    fn test_cut_meta() {
        let new_content = cut_meta("Check\n[//]: # (SYSTEM)\nSystem content".to_string());
        assert_eq!(
            new_content,
            "Check\n[//]: # (SYSTEM)\nSystem content".to_string()
        );

        let new_content = cut_meta("---\n[//]: # (SYSTEM)\n---\nasdfas".to_string());
        assert_eq!(new_content, "asdfas".to_string());
    }
}