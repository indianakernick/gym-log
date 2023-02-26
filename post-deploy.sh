#!/bin/zsh

# Note that macOS sed is quite limited compared to GNU sed. Escape sequences
# such as \w and \s are not supported. There is a tab character between the []
# because neither \s or \t can be used.
aws cloudformation describe-stacks \
  --profile gym-log \
  --stack-name gym-log \
  --query "Stacks[0].Outputs" \
  --output text \
  | sed 's/\([a-zA-Z]*\)[	]*\(.*\)/CFN_\1="\2"/g' > .env
