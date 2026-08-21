use std::collections::{HashMap, HashSet};

use crate::{client::Client, error::ServiceError};

use chrono::NaiveDate;
use rust_decimal::Decimal;

pub struct Service {
    clients: HashMap<u64, Client>,
    next_client_id: u64,
    registered_documents: HashSet<String>,
    file_counter: u64,
}

impl Default for Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Service {
    pub fn new() -> Self {
        Service {
            clients: HashMap::new(),
            next_client_id: 0,
            registered_documents: HashSet::new(),
            file_counter: 0,
        }
    }

    pub(super) fn create_client(
        &mut self,
        client_name: String,
        birth_date: NaiveDate,
        document_number: String,
        country: String,
    ) -> Result<u64, ServiceError> {
        // Every field is checked before the first write, so a rejected client
        // never leaves its document registered.
        let client_name = validate_non_empty(client_name, "client_name")?;
        let document_number = validate_non_empty(document_number, "document_number")?;
        let country = validate_non_empty(country, "country")?;
        validate_past_date(birth_date)?;

        // Note: only the surrounding spaces are removed. Two documents that
        // differ in any other character are different documents.
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

        self.next_client_id += 1;

        // As this id comes from a counter that only moves forward, the key
        // is always new and insert can never return a previous client.
        self.clients.insert(self.next_client_id, new_client);

        Ok(self.next_client_id)
    }

    pub(super) fn create_credit_transaction(
        &mut self,
        client_id: u64,
        credit_amount: Decimal,
    ) -> Result<Decimal, ServiceError> {
        validate_positive_amount(credit_amount)?;

        let client = self
            .clients
            .get_mut(&client_id)
            .ok_or(ServiceError::ClientDoesNotExist)?;

        client.balance += credit_amount;

        Ok(client.balance)
    }

    pub(super) fn create_debit_transaction(
        &mut self,
        client_id: u64,
        debit_amount: Decimal,
    ) -> Result<Decimal, ServiceError> {
        validate_positive_amount(debit_amount)?;

        let client = self
            .clients
            .get_mut(&client_id)
            .ok_or(ServiceError::ClientDoesNotExist)?;

        client.balance -= debit_amount;

        Ok(client.balance)
    }

    pub(super) fn store_balances(&mut self) -> (Vec<(u64, Decimal)>, u64) {
        let mut balances = Vec::new();

        for (client_id, client) in &mut self.clients {
            balances.push((*client_id, client.balance));
            client.balance = Decimal::ZERO;
        }

        // HashMap iteration order is arbitrary, so the cut is sorted by id to
        // keep the generated file stable across runs.
        balances.sort_by_key(|(id, _)| *id);
        self.file_counter += 1;

        (balances, self.file_counter)
    }

    /// Restores the balances of a cut that could not be persisted. The amounts are added, not assigned,
    /// so transactions that arrived after the cut are kept. The file number is not given back: a partially
    /// written file may already carry it.
    pub(super) fn restore_balances(&mut self, balances: Vec<(u64, Decimal)>) {
        for (client_id, balance) in balances {
            if let Some(client) = self.clients.get_mut(&client_id) {
                client.balance += balance;
            }
        }
    }

    pub(super) fn client_info(&self, client_id: u64) -> Result<Client, ServiceError> {
        self.clients
            .get(&client_id)
            .ok_or(ServiceError::ClientDoesNotExist)
            .cloned()
    }
}

/// Today is read here and not received as a parameter to keep `create_client` signature tied to the request body.
/// The tests stay deterministic because they use dates that are always in the past or always in the future.
fn validate_past_date(date: NaiveDate) -> Result<(), ServiceError> {
    if date > chrono::Local::now().date_naive() {
        return Err(ServiceError::FutureBirthDate);
    }

    Ok(())
}

/// Returns the value without its surrounding spaces, or an error naming the field when nothing is left.
fn validate_non_empty(value: String, field: &'static str) -> Result<String, ServiceError> {
    let value = value.trim();

    if value.is_empty() {
        return Err(ServiceError::EmptyField(field));
    }

    Ok(value.to_string())
}

fn validate_positive_amount(amount: Decimal) -> Result<(), ServiceError> {
    if amount <= Decimal::ZERO {
        return Err(ServiceError::NonPositiveAmount);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn birth_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(1990, 5, 12).expect("the test date is valid")
    }

    fn add_client(service: &mut Service, document: &str) -> u64 {
        service
            .create_client(
                "First".to_string(),
                birth_date(),
                document.to_string(),
                "Arg".to_string(),
            )
            .unwrap()
    }

    #[test]
    fn test_client_created_with_empty_balance() {
        let mut service = Service::new();

        let new_client_id = add_client(&mut service, "document_number_1");

        assert_eq!(
            service.client_info(new_client_id).unwrap().balance,
            Decimal::ZERO
        );
    }

    #[test]
    fn test_different_clients_have_different_ids() {
        let mut service = Service::new();

        let first_client = add_client(&mut service, "document_number_1");

        let second_client = add_client(&mut service, "document_number_2");

        assert_ne!(first_client, second_client);
        assert_eq!(
            service.client_info(first_client).unwrap().balance,
            Decimal::ZERO
        );
        assert_eq!(
            service.client_info(second_client).unwrap().balance,
            Decimal::ZERO
        );
    }

    #[test]
    fn test_balance_on_nonexistent_client() {
        let service = Service::new();

        assert_eq!(
            service.client_info(0),
            Err(ServiceError::ClientDoesNotExist)
        );
    }

    #[test]
    fn test_credit_increments_client_balance() {
        let mut service = Service::new();

        let client_id = add_client(&mut service, "document_number_1");

        assert_eq!(
            service.create_credit_transaction(client_id, Decimal::from(10)),
            Ok(Decimal::from(10))
        );

        assert_eq!(
            service.client_info(client_id).unwrap().balance,
            Decimal::from(10)
        );
    }

    #[test]
    fn test_fractional_credits_are_added_exactly() {
        let mut service = Service::new();

        let client_id = add_client(&mut service, "document_number_1");

        assert_eq!(
            service.create_credit_transaction(client_id, Decimal::new(1, 1)),
            Ok(Decimal::new(1, 1))
        );

        assert_eq!(
            service.create_credit_transaction(client_id, Decimal::new(2, 1)),
            Ok(Decimal::new(3, 1))
        );

        assert_eq!(
            service.client_info(client_id).unwrap().balance,
            Decimal::new(3, 1)
        );
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

        let client_id = add_client(&mut service, "document_number_1");

        assert_eq!(
            service.create_credit_transaction(client_id, Decimal::from(10)),
            Ok(Decimal::from(10))
        );

        assert_eq!(
            service.create_debit_transaction(client_id, Decimal::from(5)),
            Ok(Decimal::from(5))
        );

        assert_eq!(
            service.client_info(client_id).unwrap().balance,
            Decimal::from(5)
        );
    }

    #[test]
    fn test_fractional_credit_and_debit() {
        let mut service = Service::new();

        let client_id = add_client(&mut service, "document_number_1");

        assert_eq!(
            service.create_credit_transaction(client_id, Decimal::new(1, 1)),
            Ok(Decimal::new(1, 1))
        );

        assert_eq!(
            service.create_debit_transaction(client_id, Decimal::new(1, 1)),
            Ok(Decimal::ZERO)
        );

        assert_eq!(
            service.client_info(client_id).unwrap().balance,
            Decimal::ZERO
        );
    }

    #[test]
    fn test_debit_bigger_than_balance_leaves_negative_balance() {
        let mut service = Service::new();

        let client_id = add_client(&mut service, "document_number_1");

        assert_eq!(
            service.create_credit_transaction(client_id, Decimal::new(1, 1)),
            Ok(Decimal::new(1, 1))
        );

        assert_eq!(
            service.create_debit_transaction(client_id, Decimal::new(2, 1)),
            Ok(Decimal::new(-1, 1))
        );

        assert_eq!(
            service.client_info(client_id).unwrap().balance,
            Decimal::new(-1, 1)
        );
    }

    #[test]
    fn test_credit_and_debit_reject_non_positive_amounts() {
        let mut service = Service::new();

        let client_id = add_client(&mut service, "document_number_1");

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

        add_client(&mut service, "document_number_1");

        assert_eq!(
            service.create_client(
                "client_name".to_string(),
                birth_date(),
                "document_number_1".to_string(),
                "Arg".to_string()
            ),
            Err(ServiceError::DuplicateDocument)
        );

        let new_client_id = add_client(&mut service, "document_number_2");
        assert_eq!(new_client_id, 2);
    }

    #[test]
    fn test_client_fields_cannot_be_empty() {
        let mut service = Service::new();

        assert_eq!(
            service.create_client(
                "   ".to_string(),
                birth_date(),
                "document_number_1".to_string(),
                "Arg".to_string()
            ),
            Err(ServiceError::EmptyField("client_name"))
        );

        assert_eq!(
            service.create_client(
                "Juan".to_string(),
                birth_date(),
                "".to_string(),
                "Arg".to_string()
            ),
            Err(ServiceError::EmptyField("document_number"))
        );

        assert_eq!(
            service.create_client(
                "Juan".to_string(),
                birth_date(),
                "document_number_1".to_string(),
                " ".to_string()
            ),
            Err(ServiceError::EmptyField("country"))
        );
    }

    #[test]
    fn test_the_birth_date_cannot_be_in_the_future() {
        let mut service = Service::new();

        let tomorrow = chrono::Local::now().date_naive() + chrono::Days::new(1);

        assert_eq!(
            service.create_client(
                "Juan".to_string(),
                tomorrow,
                "document_number_1".to_string(),
                "Arg".to_string()
            ),
            Err(ServiceError::FutureBirthDate)
        );

        assert!(
            service
                .create_client(
                    "Juan".to_string(),
                    chrono::Local::now().date_naive(),
                    "document_number_1".to_string(),
                    "Arg".to_string()
                )
                .is_ok()
        );
    }

    #[test]
    fn test_a_rejected_client_leaves_its_document_free() {
        let mut service = Service::new();

        let rejected = service.create_client(
            "".to_string(),
            birth_date(),
            "document_number_1".to_string(),
            "Arg".to_string(),
        );
        assert!(rejected.is_err());

        assert!(
            service
                .create_client(
                    "Juan".to_string(),
                    birth_date(),
                    "document_number_1".to_string(),
                    "Arg".to_string()
                )
                .is_ok()
        );
    }

    #[test]
    fn test_client_fields_are_stored_without_surrounding_spaces() {
        let mut service = Service::new();

        let client_id = service
            .create_client(
                "  Juan  ".to_string(),
                birth_date(),
                "  document_number_1 ".to_string(),
                " Arg ".to_string(),
            )
            .unwrap();

        let client = service.client_info(client_id).unwrap();

        assert_eq!(client.client_name, "Juan");
        assert_eq!(client.document_number, "document_number_1");
        assert_eq!(client.country, "Arg");
    }

    #[test]
    fn test_store_balances_empty() {
        let mut service = Service::new();

        assert!(service.store_balances().0.is_empty())
    }

    #[test]
    fn test_store_balances_multiple_accounts() {
        let mut service = Service::new();

        let client_1 = add_client(&mut service, "document_number_1");
        let client_2 = add_client(&mut service, "document_number_2");

        service
            .create_credit_transaction(client_2, Decimal::new(5, 1))
            .unwrap();

        let (balances, file_number) = service.store_balances();

        assert_eq!(file_number, 1);
        assert_eq!(balances.len(), 2);
        assert_eq!(balances[0], (client_1, Decimal::ZERO));
        assert_eq!(balances[1], (client_2, Decimal::new(5, 1)));

        assert_eq!(
            service.client_info(client_1).unwrap().balance,
            Decimal::ZERO
        );
        assert_eq!(
            service.client_info(client_2).unwrap().balance,
            Decimal::ZERO
        );
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
    fn test_restore_balances_puts_a_failed_cut_back() {
        let mut service = Service::new();

        let client_id = add_client(&mut service, "document_number_1");
        service
            .create_credit_transaction(client_id, Decimal::from(10))
            .unwrap();

        let (balances, _) = service.store_balances();
        service.restore_balances(balances);

        assert_eq!(
            service.client_info(client_id).unwrap().balance,
            Decimal::from(10)
        );
    }

    #[test]
    fn test_restore_balances_keeps_the_transactions_of_the_meantime() {
        let mut service = Service::new();

        let client_id = add_client(&mut service, "document_number_1");
        service
            .create_credit_transaction(client_id, Decimal::from(10))
            .unwrap();

        let (balances, _) = service.store_balances();

        service
            .create_credit_transaction(client_id, Decimal::from(4))
            .unwrap();
        service.restore_balances(balances);

        assert_eq!(
            service.client_info(client_id).unwrap().balance,
            Decimal::from(14)
        );
    }

    #[test]
    fn test_client_info_returns_client_data_and_balance() {
        let mut service = Service::new();

        let client_id = add_client(&mut service, "document_number_1");

        assert_eq!(
            service.create_credit_transaction(client_id, Decimal::new(1, 1)),
            Ok(Decimal::new(1, 1))
        );
        let client = service.client_info(client_id).unwrap();

        assert_eq!(client.client_name, "First");
        assert_eq!(client.birth_date, birth_date());
        assert_eq!(client.document_number, "document_number_1");
        assert_eq!(client.country, "Arg");
        assert_eq!(client.balance, Decimal::new(1, 1));
    }
}
