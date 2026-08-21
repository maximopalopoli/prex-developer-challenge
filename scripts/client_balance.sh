#!/usr/bin/env bash
URL_1=http://127.0.0.1:8080/new_client
URL_2=http://127.0.0.1:8080/new_credit_transaction
URL_3=http://127.0.0.1:8080/client_balance
DOC="doc-$$"
fails=0

check() { # check <what> <expected> <actual>
    if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: expected $2, got $3"; fails=1; fi
}

post_body() { curl -s -X POST "$2" -H 'Content-Type: application/json' -d "$1"; }

get() { curl -s -o /dev/null -w '%{http_code}' "$1"; }

get_body() { curl -s "$1"; }

ID=$(post_body "{\"client_name\":\"Juan\",\"birth_date\":\"1990-05-12\",\"document_number\":\"$DOC\",\"country\":\"AR\"}" $URL_1 \
       | sed 's/.*"client_id":\([0-9]*\).*/\1/')

post_body "{\"client_id\":$ID,\"credit_amount\":0.1}" $URL_2 > /dev/null

check "returns the client with its balance" \
    "{\"client_name\":\"Juan\",\"birth_date\":\"1990-05-12\",\"document_number\":\"$DOC\",\"country\":\"AR\",\"client_balance\":\"0.1\"}" \
    "$(get_body "$URL_3?user_id=$ID")"

check "fails on nonexistent client" 404 "$(get "$URL_3?user_id=0")"

check "fails without the parameter" 400 "$(get "$URL_3")"

exit $fails
