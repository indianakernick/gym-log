#!/bin/zsh

npm run build
aws --profile gym-log s3 sync dist s3://gymlog.indianakernick.com
