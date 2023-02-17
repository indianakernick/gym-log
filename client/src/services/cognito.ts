import {
  CognitoIdentityProviderClient,
  ConfirmSignUpCommand,
  InitiateAuthCommand,
  ResendConfirmationCodeCommand,
  SignUpCommand,
  type AuthenticationResultType
} from '@aws-sdk/client-cognito-identity-provider';

const CLIENT_ID: string = import.meta.env.CFN_CognitoClientId;

export default new class {
  private client: CognitoIdentityProviderClient;

  constructor() {
    this.client = new CognitoIdentityProviderClient({
      region: 'ap-southeast-2'
    });
  }

  async signUp(email: string, password: string): Promise<void> {
    await this.client.send(new SignUpCommand({
      ClientId: CLIENT_ID,
      Username: email,
      Password: password,
      UserAttributes: [{
        Name: 'email',
        Value: email
      }]
    }));
  }

  async confirmSignUp(email: string, confirmationCode: string): Promise<void> {
    await this.client.send(new ConfirmSignUpCommand({
      ClientId: CLIENT_ID,
      Username: email,
      ConfirmationCode: confirmationCode
    }));
  }

  async resendConfirmSignUp(email: string): Promise<void> {
    await this.client.send(new ResendConfirmationCodeCommand({
      ClientId: CLIENT_ID,
      Username: email
    }));
  }

  async login(email: string, password: string): Promise<AuthenticationResultType> {
    // AuthenticationResult will always be present because MFA is disabled.
    return (await this.client.send(new InitiateAuthCommand({
      ClientId: CLIENT_ID,
      AuthFlow: 'USER_PASSWORD_AUTH',
      AuthParameters: {
        USERNAME: email,
        PASSWORD: password
      }
    }))).AuthenticationResult!;
  }

  async refresh(refreshToken: string): Promise<AuthenticationResultType> {
    // AuthenticationResult will always be present because MFA is disabled.
    return (await this.client.send(new InitiateAuthCommand({
      ClientId: CLIENT_ID,
      AuthFlow: 'REFRESH_TOKEN_AUTH',
      AuthParameters: {
        REFRESH_TOKEN: refreshToken
      }
    }))).AuthenticationResult!;
  }
}
