use std::collections::{HashMap, HashSet};

use crate::error::ServiceError;

use rust_decimal::Decimal;

struct Service {
    accounts: HashMap<u64, Client>,
    next_client_id: u64,
    registered_documents: HashSet<String>,
    file_counter: u64,
}

impl Service {
    fn new() -> Self {
        Service {
            accounts: HashMap::new(),
            next_client_id: 0,
            registered_documents: HashSet::new(),
            file_counter: 0,
        }
    }

    fn create_account(
        &mut self,
        client_name: String,
        birth_date: String,
        document_number: String,
        country: String,
    ) -> Result<u64, ServiceError> {
        // Note: Here we may normalize the number before comparison
        // as two strings may refer to the same number
        if !self.registered_documents.insert(document_number.clone()) {
            return Err(ServiceError::DuplicateDocument);
        }

        let new_client = Client {
            client_name,
            birth_date,
            document_number,
            country,
            balance: Decimal::ZERO,
        };

        let new_client_id = self.next_client_id;

        // As this id comes from a counter that only moves forward, the key
        // is always new and insert can never return a previous client.
        self.accounts.insert(new_client_id, new_client);

        self.next_client_id += 1;
        Ok(new_client_id)
    }

    // Check if this method is really needed as Service::client_info already exists
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
        validate_positive_amount(credit_amount)?;

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
        validate_positive_amount(debit_amount)?;

        let client = self
            .accounts
            .get_mut(&client_id)
            .ok_or(ServiceError::ClientDoesNotExist)?;

        client.balance -= debit_amount;

        Ok(client.balance)
    }

    fn store_balances(&mut self) -> (Vec<(u64, Decimal)>, u64) {
        let mut balances = Vec::new();

        for (client_id, client) in &mut self.accounts {
            balances.push((*client_id, client.balance));
            client.balance = Decimal::ZERO;
        }

        // HashMap iteration order is arbitrary, so the cut is sorted by id to
        // keep the generated file stable across runs.
        balances.sort_by_key(|(id, _)| *id);
        self.file_counter += 1;

        (balances, self.file_counter)
    }

    fn client_info(&self, client_id: u64) -> Result<Client, ServiceError> {
        self.accounts
            .get(&client_id)
            .ok_or(ServiceError::ClientDoesNotExist)
            .cloned()
    }
}

fn validate_positive_amount(amount: Decimal) -> Result<(), ServiceError> {
    if amount <= Decimal::ZERO {
        return Err(ServiceError::NonPositiveAmount);
    }

    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
struct Client {
    balance: Decimal,
    client_name: String,
    birth_date: String, // Change to NaiveDate
    document_number: String,
    country: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add_account(service: &mut Service, document: &str) -> u64 {
        service
            .create_account(
                "First".to_string(),
                "birth_date".to_string(),
                document.to_string(),
                "Arg".to_string(),
            )
            .unwrap()
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

    #[test]
    fn test_debit_bigger_than_balance_leaves_negative_balance() {
        let mut service = Service::new();

        let client_id = add_account(&mut service, "document_number_1");

        assert_eq!(
            service.create_credit_transaction(client_id, Decimal::new(1, 1)),
            Ok(Decimal::new(1, 1))
        );

        assert_eq!(
            service.create_debit_transaction(client_id, Decimal::new(2, 1)),
            Ok(Decimal::new(-1, 1))
        );

        assert_eq!(service.client_balance(client_id), Ok(Decimal::new(-1, 1)));
    }

    #[test]
    fn test_credit_and_debit_reject_non_positive_amounts() {
        let mut service = Service::new();

        let client_id = add_account(&mut service, "document_number_1");

        assert_eq!(
            service.create_credit_transaction(client_id, Decimal::new(-1, 1)),
            Err(ServiceError::NonPositiveAmount)
        );

        assert_eq!(
            service.create_debit_transaction(client_id, Decimal::new(-1, 1)),
            Err(ServiceError::NonPositiveAmount)
        );

        assert_eq!(
            service.create_credit_transaction(client_id, Decimal::ZERO),
            Err(ServiceError::NonPositiveAmount)
        );

        assert_eq!(
            service.create_debit_transaction(client_id, Decimal::ZERO),
            Err(ServiceError::NonPositiveAmount)
        );
    }

    #[test]
    fn test_cannot_insert_same_document_twice() {
        let mut service = Service::new();

        add_account(&mut service, "document_number_1");

        assert_eq!(
            service.create_account(
                "client_name".to_string(),
                "birth_date".to_string(),
                "document_number_1".to_string(),
                "Arg".to_string()
            ),
            Err(ServiceError::DuplicateDocument)
        );

        let new_client_id = add_account(&mut service, "document_number_2");
        assert_eq!(new_client_id, 1);
    }

    #[test]
    fn test_store_balances_empty() {
        let mut service = Service::new();

        assert!(service.store_balances().0.is_empty())
    }

    #[test]
    fn test_store_balances_multiple_accounts() {
        let mut service = Service::new();

        let client_1 = add_account(&mut service, "document_number_1");
        let client_2 = add_account(&mut service, "document_number_2");

        service
            .create_credit_transaction(client_2, Decimal::new(5, 1))
            .unwrap();

        let (balances, file_number) = service.store_balances();

        assert_eq!(file_number, 1);
        assert_eq!(balances.len(), 2);
        assert_eq!(balances[0], (client_1, Decimal::ZERO));
        assert_eq!(balances[1], (client_2, Decimal::new(5, 1)));

        assert_eq!(service.client_balance(client_1), Ok(Decimal::ZERO));
        assert_eq!(service.client_balance(client_2), Ok(Decimal::ZERO));
    }

    #[test]
    fn test_successive_cuts_increase_the_file_number() {
        let mut service = Service::new();

        let (balances, file_number) = service.store_balances();

        assert!(balances.is_empty());
        assert_eq!(file_number, 1);

        let (balances, file_number) = service.store_balances();
        assert!(balances.is_empty());
        assert_eq!(file_number, 2);
    }

    #[test]
    fn test_client_info_returns_client_data_and_balance() {
        let mut service = Service::new();

        let client_id = add_account(&mut service, "document_number_1");

        assert_eq!(
            service.create_credit_transaction(client_id, Decimal::new(1, 1)),
            Ok(Decimal::new(1, 1))
        );
        let client = service.client_info(client_id).unwrap();

        assert_eq!(client.client_name, "First");
        assert_eq!(client.birth_date, "birth_date");
        assert_eq!(client.document_number, "document_number_1");
        assert_eq!(client.country, "Arg");
        assert_eq!(client.balance, Decimal::new(1, 1));
    }
}
