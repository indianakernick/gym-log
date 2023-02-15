#!/bin/zsh

npm run build
aws s3 sync --profile gym-log --delete dist s3://gymlog.indianakernick.com
