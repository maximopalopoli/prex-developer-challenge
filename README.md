# Prex Challenge

## What is this

Mini payment processor written in Rust as a challenge. It keeps client balances in memory and persists them to a file on demand, exposed through a REST API.

## How to run

```sh
cargo test    # unit tests
cargo run     # starts the service
```

## Design decisions

### Balances are `Decimal`, not `f64`

Binary floating point cannot represent most decimal fractions exactly, so `0.1 + 0.2` does not equal `0.3`. Over a long run of credits and debits that error accumulates and balances stop reconciling, which is not acceptable in a payment processor. `rust_decimal` works in base 10, storing an integer along with its scale, so amounts survive every operation exactly as they were written. The statement recommends the Decimal format, and this is the reason behind it.

### Debits can leave a balance negative

A debit larger than the current balance goes through and the client ends up owing money. The sample file in the statement lists `03 -3000`, so a negative balance is a state the service is expected to reach, and rejecting the operation would contradict the example.
