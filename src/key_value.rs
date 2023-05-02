use std::collections::HashMap;

#[derive(Debug)]
pub struct KeyValue(HashMap<String, String>);

impl KeyValue {
    pub fn profile_for(email: &str) -> Self {
        let email = email
            .chars()
            .filter(|ch| *ch != '=' && *ch != '&')
            .collect();

        Self(HashMap::from([
            (String::from("email"), email),
            (String::from("uid"), String::from("10")),
            (String::from("role"), String::from("user")),
        ]))
    }

    pub fn decode(encoding: &str) -> Self {
        Self(
            encoding
                .split('&')
                .map(|pair| {
                    let mut kv = pair.split('=');
                    (
                        String::from(kv.next().expect("no key")),
                        String::from(kv.next().expect("no value")),
                    )
                })
                .collect(),
        )
    }

    pub fn encode(&self) -> String {
        self.0
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<String>>()
            .join("&")
    }
}
