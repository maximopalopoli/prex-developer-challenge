# Prex Challenge

## What is this

Mini payment processor written in Rust as a challenge. It keeps client balances in memory and persists them to a file on demand, exposed through a REST API.

## How to run

```sh
make run     # starts the service on 127.0.0.1:8080
make test    # unit tests
make lint    # clippy

`HOST` and `PORT` override the defaults: `HOST=0.0.0.0 PORT=9000 make run`.
```

## API

| Method | Path | Input | Success |
|---|---|---|---|
| POST | `/new_client` | `client_name`, `birth_date`, `document_number`, `country` | 201 with the new `client_id` |
| POST | `/new_credit_transaction` | `client_id`, `credit_amount` | 200 with the new balance |
| POST | `/new_debit_transaction` | `client_id`, `debit_amount` | 200 with the new balance |
| GET | `/client_balance?user_id=` | the client id in the query string | 200 with the client and its balance |
| POST | `/store_balances` | no input | 200 with the name of the generated file |

Client ids start at 1. Amounts are accepted as JSON numbers and returned as
strings, so that whoever consumes the API decides how to parse them instead of
being handed a float.

You can find a Postman collection in `postman/`.

## Validations

- The document number cannot be repeated. It is compared exactly as received, with no normalization, so `12.345.678` and `12345678` are two different documents.
- Credit and debit amounts must be strictly positive. Zero is rejected because a transaction with no effect is almost always a mistake by the caller.
- Amounts are validated before the client is looked up, so a negative amount on an unknown client answers 400 and not 404.
- A debit is never rejected for insufficient funds.

## The balance file

`store_balances` writes one line per client as `ID<space>BALANCE`, and names the file `DDMMYYYY_N.DAT`, where `N` counts the files generated since the service started. Ids are written as they are, without padding, and balances keep the scale they were stored with, which is what the example in the statement shows. A cut with no clients still writes an empty file and still consumes a number.

## Design decisions

### Balances are `Decimal`, not `f64`

Binary floating point cannot represent most decimal fractions exactly, so `0.1 + 0.2` does not equal `0.3`. Over a long run of credits and debits that error accumulates and balances stop reconciling, which is not acceptable in a payment processor. `rust_decimal` works in base 10, storing an integer along with its scale, so amounts survive every operation exactly as they were written. The statement recommends the Decimal format, and this is the reason behind it.

### Debits can leave a balance negative

A debit larger than the current balance goes through and the client ends up owing money. The sample file in the statement lists `03 -3000`, so a negative balance is a state the service is expected to reach, and rejecting the operation would contradict the example.

### The cut runs under the lock, the file is written outside

`store_balances` takes the balances and zeroes them in a single operation while holding the lock, and the file is written afterwards, on a blocking thread, with the snapshot already in hand. Writing to disk while holding the lock would stop every transaction for the length of a disk access.

### Domain errors and transport errors are separate types

`ServiceError` describes what the caller did wrong and knows nothing about HTTP. `ApiError` adds what can fail at the border, such as a poisoned lock or a failed write, and is the only type that decides status codes.

## Testing

`make test` runs the unit tests of the domain and of the file format. Each endpoint also has an acceptance script that drives the real API with curl:

```
make test_new_client
make test_new_credit_tx
make test_new_debit_tx
make test_client_balance
make test_store_balances
```
