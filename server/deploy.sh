#!/bin/zsh

cargo lambda build --release --output-format zip
aws lambda update-function-code \
  --profile gym-log \
  --function-name gym-log \
  --zip-file fileb://./target/lambda/gym-log/bootstrap.zip
