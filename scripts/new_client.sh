#!/usr/bin/env bash
# Checks for POST /new_client. Needs the service running (make test_new_client).
URL=http://127.0.0.1:8080/new_client
DOC="doc-$$"
fails=0

check() { # check <what> <expected status> <actual status>
    if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: expected $2, got $3"; fails=1; fi
}

post() { curl -s -o /dev/null -w '%{http_code}' -X POST "$URL" -H 'Content-Type: application/json' -d "$1"; }

check "creates a client" 201 \
    "$(post "{\"client_name\":\"Juan\",\"birth_date\":\"1990-05-12\",\"document_number\":\"$DOC\",\"country\":\"AR\"}")"

check "rejects a duplicate document" 409 \
    "$(post "{\"client_name\":\"Maria\",\"birth_date\":\"1990-05-12\",\"document_number\":\"$DOC\",\"country\":\"AR\"}")"

check "rejects an incomplete body" 400 \
    "$(post '{"client_name":"Maria"}')"

check "rejects an empty field" 400 \
    "$(post "{\"client_name\":\"  \",\"birth_date\":\"1990-05-12\",\"document_number\":\"other-$$\",\"country\":\"AR\"}")"

check "rejects a birth date in the future" 400 \
    "$(post "{\"client_name\":\"Maria\",\"birth_date\":\"2090-01-01\",\"document_number\":\"other-$$\",\"country\":\"AR\"}")"

exit $fails
