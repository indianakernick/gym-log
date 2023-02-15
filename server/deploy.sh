#!/bin/zsh

cargo lambda build --release --arm64 --output-format zip
# CloudFormation will not see that the S3 object has changed and won't update
# the Lambda function despite S3 objects having a modification date on them. The
# S3 key would need to change for CloudFormation to notice the change. This S3
# upload step is left here just in case CloudFormation decides to drop and
# recreate the Lambda function.
aws s3 cp \
  --profile gym-log \
  ./target/lambda/gym-log/bootstrap.zip \
  s3://indianakernick-lambda/gym-log.zip
# We can upload the ZIP file directly to the Lambda but we can only do this
# after the Lambda has been created for the first time. The code needs to be in
# S3 for it to be created.
aws lambda update-function-code \
  --profile gym-log \
  --function-name gym-log \
  --zip-file fileb://./target/lambda/gym-log/bootstrap.zip
