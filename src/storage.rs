use rust_decimal::Decimal;
use std::io::Error;

pub(crate) fn save_state(balances: Vec<(u64, Decimal)>, file_number: u64) -> Result<String, Error> {
    let today_string = chrono::Local::now().format("%d%m%Y").to_string();

    let file_name = format!("{}_{}.DAT", today_string, file_number);

    let balances_string = generate_balances_text(balances);

    std::fs::write(&file_name, balances_string)?;

    Ok(file_name)
}

fn generate_balances_text(balances: Vec<(u64, Decimal)>) -> String {
    let mut balances_text = "".to_string();

    for (client_id, balance) in balances {
        let client_row = format!("{} {}\n", client_id, balance);
        balances_text.push_str(&client_row)
    }

    balances_text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balances_text_follows_the_file_format() {
        let balances = vec![
            (1, Decimal::new(50012, 2)),
            (2, Decimal::new(1999935, 2)),
            (3, Decimal::from(-3000)),
            (4, Decimal::new(78976, 2)),
        ];

        assert_eq!(
            generate_balances_text(balances),
            "1 500.12\n2 19999.35\n3 -3000\n4 789.76\n"
        );
    }

    #[test]
    fn test_balances_text_is_empty_without_clients() {
        assert_eq!(generate_balances_text(vec![]), "");
    }
}
