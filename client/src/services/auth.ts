import cognito from './cognito';
import db from './db';

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

    // Refresh token might not exist.
    // Refresh token might have expired.
    // Refreshing could fail for many reasons.
    const auth = (await cognito.refresh(refreshToken!))!;

    this.accessToken = {
      value: auth.AccessToken!,
      expiration: Date.now() + auth.ExpiresIn! * 1000 - 30000
    };

    return this.accessToken.value;
  }

  async isLoggedIn(): Promise<boolean> {
    return !!await db.getRefreshToken();
  }
};
