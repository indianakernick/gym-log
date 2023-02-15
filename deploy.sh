#!/bin/zsh

aws cloudformation deploy \
  --profile gym-log \
  --template-file template.yaml \
  --stack-name gym-log \
  --capabilities CAPABILITY_NAMED_IAM
