#!/bin/zsh

aws --profile gym-log s3 sync www s3://gymlog.indianakernick.com
