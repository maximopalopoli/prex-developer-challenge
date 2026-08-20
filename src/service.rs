use std::collections::HashMap;

struct Service {
    accounts: HashMap<String, i64>,
}

impl Service {
    fn new() -> Self {
        let accounts = HashMap::new();
        Service { accounts }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_starts_empty() {
        let service = Service::new();

        let service_accounts: Vec<&String> = service.accounts.keys().collect();
        assert!(service_accounts.is_empty())
    }
}
