#!/usr/bin/env bash
URL_1=http://127.0.0.1:8080/new_client
URL_2=http://127.0.0.1:8080/new_credit_transaction
DOC="doc-$$"
fails=0

check() { # check <what> <expected status> <actual status>
    if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: expected $2, got $3"; fails=1; fi
}

post() { curl -s -o /dev/null -w '%{http_code}' -X POST "$2" -H 'Content-Type: application/json' -d "$1"; }

post_body() { curl -s -X POST "$2" -H 'Content-Type: application/json' -d "$1"; }

ID=$(post_body "{\"client_name\":\"Ada\",\"birth_date\":\"1990-05-12\",\"document_number\":\"$DOC\",\"country\":\"AR\"}" $URL_1 \
       | sed 's/.*"client_id":\([0-9]*\).*/\1/')

check "creates a credit tx" 200 \
    "$(post "{\"client_id\":$ID,\"credit_amount\":0.5}" $URL_2 )" 


exit $fails
