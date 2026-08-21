#!/usr/bin/env bash
URL_1=http://127.0.0.1:8080/new_client
URL_2=http://127.0.0.1:8080/new_credit_transaction
URL_3=http://127.0.0.1:8080/client_balance
URL_4=http://127.0.0.1:8080/store_balances
DOC="doc-$$"
fails=0

TODAY=$(date +%d%m%Y)

check() { # check <what> <expected> <actual>
    if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: expected $2, got $3"; fails=1; fi
}

post_body() { curl -s -X POST "$2" -H 'Content-Type: application/json' -d "$1"; }

get_body() { curl -s "$1"; }

ID=$(post_body "{\"client_name\":\"Juan\",\"birth_date\":\"1990-05-12\",\"document_number\":\"$DOC\",\"country\":\"AR\"}" $URL_1 \
       | sed 's/.*"client_id":\([0-9]*\).*/\1/')

post_body "{\"client_id\":$ID,\"credit_amount\":5}" $URL_2 > /dev/null

check "returns the name of the generated file" \
    "{\"generated_file_name\":\"${TODAY}_1.DAT\"}" \
    "$(curl -s -X POST $URL_4)"

check "writes the balance of the client" "$ID 5" \
    "$(grep "^$ID " "${TODAY}_1.DAT" 2>/dev/null)"

check "leaves the client with no balance" "0" \
    "$(get_body "$URL_3?user_id=$ID" | sed 's/.*"client_balance":"\([^"]*\)".*/\1/')"

check "numbers the next file" \
    "{\"generated_file_name\":\"${TODAY}_2.DAT\"}" \
    "$(curl -s -X POST $URL_4)"

rm -f "${TODAY}_1.DAT" "${TODAY}_2.DAT"

exit $fails
