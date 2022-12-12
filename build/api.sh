#!/usr/bin/env bash

# cd repo root
cd $(dirname $0)/..

# generate .go files in /api
go run github.com/99designs/gqlgen generate
