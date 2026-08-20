use std::collections::HashMap;

use crate::error::ServiceError;

use rust_decimal::Decimal;

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
            balance: Decimal::ZERO,
        };

        let new_client_id = self.id_count;

        // As this id comes from a counter that only moves forward, the key
        // is always new and insert can never return a previous client.
        self.accounts.insert(new_client_id, new_client);

        self.id_count += 1;
        new_client_id
    }

    fn client_balance(&self, client_id: u64) -> Result<Decimal, ServiceError> {
        self.accounts
            .get(&client_id)
            .map(|client| client.balance)
            .ok_or(ServiceError::ClientDoesNotExist)
    }

    fn create_credit_transaction(
        &mut self,
        client_id: u64,
        credit_amount: Decimal,
    ) -> Result<Decimal, ServiceError> {
        let client = self
            .accounts
            .get_mut(&client_id)
            .ok_or(ServiceError::ClientDoesNotExist)?;

        client.balance += credit_amount;

        Ok(client.balance)
    }

    fn create_debit_transaction(
        &mut self,
        client_id: u64,
        debit_amount: Decimal,
    ) -> Result<Decimal, ServiceError> {
        let client = self
            .accounts
            .get_mut(&client_id)
            .ok_or(ServiceError::ClientDoesNotExist)?;

        client.balance -= debit_amount;

        Ok(client.balance)
    }
}

struct Client {
    balance: Decimal,
    client_name: String,
    birth_date: String,
    document_number: String,
    country: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add_account(service: &mut Service, document: &str) -> u64 {
        service.create_account(
            "First".to_string(),
            "birth_date".to_string(),
            document.to_string(),
            "Arg".to_string(),
        )
    }

    #[test]
    fn test_service_starts_empty() {
        let service = Service::new();

        let service_accounts: Vec<&Client> = service.accounts.values().collect();
        assert!(service_accounts.is_empty())
    }

    #[test]
    fn test_client_created_with_empty_balance() {
        let mut service = Service::new();

        let new_client_id = add_account(&mut service, "document_number_1");

        assert_eq!(service.client_balance(new_client_id), Ok(Decimal::ZERO));
    }

    #[test]
    fn test_different_clients_have_different_ids() {
        let mut service = Service::new();

        let first_client = add_account(&mut service, "document_number_1");

        let second_client = add_account(&mut service, "document_number_2");

        assert_ne!(first_client, second_client);
        assert_eq!(service.client_balance(first_client), Ok(Decimal::ZERO));
        assert_eq!(service.client_balance(second_client), Ok(Decimal::ZERO));
    }

    #[test]
    fn test_balance_on_nonexistent_client() {
        let service = Service::new();

        assert_eq!(
            service.client_balance(0),
            Err(ServiceError::ClientDoesNotExist)
        );
    }

    #[test]
    fn test_credit_increments_client_balance() {
        let mut service = Service::new();

        let client_id = add_account(&mut service, "document_number_1");

        assert_eq!(
            service.create_credit_transaction(client_id, Decimal::from(10)),
            Ok(Decimal::from(10))
        );

        assert_eq!(service.client_balance(client_id), Ok(Decimal::from(10)));
    }

    #[test]
    fn test_fractional_credits_are_added_exactly() {
        let mut service = Service::new();

        let client_id = add_account(&mut service, "document_number_1");

        assert_eq!(
            service.create_credit_transaction(client_id, Decimal::new(1, 1)),
            Ok(Decimal::new(1, 1))
        );

        assert_eq!(
            service.create_credit_transaction(client_id, Decimal::new(2, 1)),
            Ok(Decimal::new(3, 1))
        );

        assert_eq!(service.client_balance(client_id), Ok(Decimal::new(3, 1)));
    }

    #[test]
    fn test_credit_on_nonexistent_client() {
        let mut service = Service::new();

        assert_eq!(
            service.create_credit_transaction(0, Decimal::new(1, 1)),
            Err(ServiceError::ClientDoesNotExist)
        );
    }

    #[test]
    fn test_debit_decrements_client_balance() {
        let mut service = Service::new();

        let client_id = add_account(&mut service, "document_number_1");

        assert_eq!(
            service.create_credit_transaction(client_id, Decimal::from(10)),
            Ok(Decimal::from(10))
        );

        assert_eq!(
            service.create_debit_transaction(client_id, Decimal::from(5)),
            Ok(Decimal::from(5))
        );

        assert_eq!(service.client_balance(client_id), Ok(Decimal::from(5)));
    }

    #[test]
    fn test_fractional_credit_and_debit() {
        let mut service = Service::new();

        let client_id = add_account(&mut service, "document_number_1");

        assert_eq!(
            service.create_credit_transaction(client_id, Decimal::new(1, 1)),
            Ok(Decimal::new(1, 1))
        );

        assert_eq!(
            service.create_debit_transaction(client_id, Decimal::new(1, 1)),
            Ok(Decimal::ZERO)
        );

        assert_eq!(service.client_balance(client_id), Ok(Decimal::ZERO));
    }
}
