import { CognitoIdentityProviderServiceException } from '@aws-sdk/client-cognito-identity-provider';

export function getCognitoErrorMessage(e: unknown): string {
  return e instanceof CognitoIdentityProviderServiceException
    ? e.message
    : 'Something went wrong';
}
