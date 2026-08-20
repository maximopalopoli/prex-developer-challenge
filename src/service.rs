use std::collections::HashMap;

use crate::error::ServiceError;

struct Service {
    accounts: HashMap<u64, Client>,
    id_count: u64,
}

impl Service {
    fn new() -> Self {
        let accounts = HashMap::new();
        Service {
            accounts,
            id_count: 0,
        }
    }

    fn create_account(
        &mut self,
        client_name: String,
        birth_date: String,
        document_number: String,
        country: String,
    ) -> u64 {
        let new_client = Client {
            client_name,
            birth_date,
            document_number,
            country,
            balance: 0,
        };

        let new_client_id = self.id_count;

        // As this id comes from a counter that only moves forward, the key
        // is always new and insert can never return a previous client.
        self.accounts.insert(new_client_id, new_client);

        self.id_count += 1;
        new_client_id
    }

    fn client_balance(&self, client_id: u64) -> Result<i64, ServiceError> {
        self.accounts
            .get(&client_id)
            .map(|client| client.balance)
            .ok_or(ServiceError::ClientDoesNotExist)
    }
}

struct Client {
    balance: i64,
    client_name: String,
    birth_date: String,
    document_number: String,
    country: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_starts_empty() {
        let service = Service::new();

        let service_accounts: Vec<&Client> = service.accounts.values().collect();
        assert!(service_accounts.is_empty())
    }

    #[test]
    fn test_client_created_with_empty_balance() {
        let mut service = Service::new();

        let new_client_id = service.create_account(
            "Name".to_string(),
            "birth_date".to_string(),
            "document_number".to_string(),
            "Arg".to_string(),
        );

        assert_eq!(service.client_balance(new_client_id), Ok(0));
    }
}
