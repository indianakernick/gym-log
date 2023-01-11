#!/bin/zsh

aws --profile gym-log s3 sync client/www s3://gymlog.indianakernick.com
