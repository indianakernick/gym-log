import { NotAuthorizedException } from '@aws-sdk/client-cognito-identity-provider';
import cognito from './cognito';
import db from './db';

export class UnauthenticatedError extends Error {}

export default new class {
  private accessToken?: {
    value: string;
    expiration: number;
  };

  async getAccessToken(): Promise<string> {
    if (this.accessToken && this.accessToken.expiration > Date.now()) {
      return this.accessToken.value;
    }

    const refreshToken = await db.getRefreshToken();

    // If there is no refresh token, the user is not authenticated.
    if (!refreshToken) throw new UnauthenticatedError();

    try {
      const auth = await cognito.refresh(refreshToken);

      this.accessToken = {
        value: auth.AccessToken!,
        expiration: Date.now() + auth.ExpiresIn! * 1000 - 30000
      };

      return this.accessToken.value;
    } catch (e) {
      if (e instanceof NotAuthorizedException) {
        // The refresh token is invalid. It must have expired. Clear the stored
        // refresh token so that we can quickly check whether the user is
        // authenticated.
        await db.clearRefreshToken();
        throw new UnauthenticatedError();
      }

      throw e;
    }
  }

  async isAuthenticated(): Promise<boolean> {
    // If the refresh token is not present, then the user definitely isn't
    // authenticated. However, if it is present, then it could have expired. I
    // believe the Amplify framework is able to extract the expiry date but it
    // involves some complex cryptography.
    return !!await db.getRefreshToken();
  }

  logout(): Promise<void> {
    return db.clearRefreshToken();
  }
}
