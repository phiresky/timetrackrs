#!/bin/bash

dir="$(dirname $0)"

rm -rf "$dir/.dev-db/"
cd "$dir"
mkdir .dev-db