import {
  CognitoIdentityProviderClient,
  ConfirmSignUpCommand,
  InitiateAuthCommand,
  SignUpCommand,
  type AuthenticationResultType
} from '@aws-sdk/client-cognito-identity-provider';

const CLIENT_ID = '2s3btp4k59lncolr0talh0grb6';

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

  async login(email: string, password: string): Promise<AuthenticationResultType | undefined> {
    return (await this.client.send(new InitiateAuthCommand({
      ClientId: CLIENT_ID,
      AuthFlow: 'USER_PASSWORD_AUTH',
      AuthParameters: {
        USERNAME: email,
        PASSWORD: password
      }
    }))).AuthenticationResult;
  }

  async refresh(refreshToken: string): Promise<AuthenticationResultType | undefined> {
    return (await this.client.send(new InitiateAuthCommand({
      ClientId: CLIENT_ID,
      AuthFlow: 'REFRESH_TOKEN_AUTH',
      AuthParameters: {
        REFRESH_TOKEN: refreshToken
      }
    }))).AuthenticationResult;
  }
}
