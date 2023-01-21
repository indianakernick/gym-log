<script setup lang="ts">
import {
  CognitoIdentityProviderClient,
  ConfirmSignUpCommand,
  InitiateAuthCommand,
  SignUpCommand,
  type AuthenticationResultType
} from '@aws-sdk/client-cognito-identity-provider';
import { ref } from 'vue';

const BASE_URL = 'https://pa36mmpygd.execute-api.ap-southeast-2.amazonaws.com/';
const THING_URL = `${BASE_URL}thing`;
const CLIENT_ID = '2s3btp4k59lncolr0talh0grb6';

let currentValue = ref('');
let newValue = '';

async function get() {
  const res = await fetch(THING_URL, {
    method: 'GET',
    headers: { Authorization: auth.AccessToken || '' }
  });
  if (res.ok) {
    currentValue.value = await res.text();
  }
}

async function put() {
  await fetch(THING_URL, {
    method: 'PUT',
    headers: { Authorization: auth.AccessToken || '' },
    body: newValue
  });
}

let email = '';
let password = '';

async function signUp() {
  const client = new CognitoIdentityProviderClient({
    region: 'ap-southeast-2',
  });

  const command = new SignUpCommand({
    ClientId: CLIENT_ID,
    Username: email,
    Password: password,
    UserAttributes: [{
      Name: 'email',
      Value: email
    }]
  });

  const output = await client.send(command);

  console.log(output);
}

let confirmationCode = '';

async function confirm() {
  const client = new CognitoIdentityProviderClient({
    region: 'ap-southeast-2',
  });

  const command = new ConfirmSignUpCommand({
    ClientId: CLIENT_ID,
    Username: email,
    ConfirmationCode: confirmationCode
  });

  const output = await client.send(command);

  console.log(output);
}

let auth: AuthenticationResultType = {};

async function login() {
  const client = new CognitoIdentityProviderClient({
    region: 'ap-southeast-2',
  });

  const command = new InitiateAuthCommand({
    ClientId: CLIENT_ID,
    AuthFlow: 'USER_PASSWORD_AUTH',
    AuthParameters: {
      USERNAME: email,
      PASSWORD: password
    }
  });

  const output = await client.send(command);

  console.log(output);

  if (output.AuthenticationResult) {
    auth = output.AuthenticationResult;
    (window as any).token = auth.AccessToken;
  }
}

async function refresh() {
  const client = new CognitoIdentityProviderClient({
    region: 'ap-southeast-2',
  });

  const command = new InitiateAuthCommand({
    ClientId: CLIENT_ID,
    AuthFlow: 'REFRESH_TOKEN_AUTH',
    AuthParameters: {
      REFRESH_TOKEN: auth.RefreshToken || ''
    }
  });

  const output = await client.send(command);

  console.log(output);

  if (output.AuthenticationResult) {
    auth = output.AuthenticationResult;
  }
}
</script>

<template>
  <main>
    <h1>Test</h1>

    <div>
      <button @click="get">Get</button>
      <span>The value is:</span>
      <output>{{ currentValue }}</output>
    </div>

    <div>
      <button @click="put">Put</button>
      <label for="value">New value:</label>
      <input type="text" id="value" v-model.lazy="newValue" />
    </div>

    <div>
      <label for="email">Email:</label>
      <input type="email" id="email" v-model.lazy="email" />
      <br/>
      <label for="password">Password:</label>
      <input type="password" id="password" v-model.lazy="password" />
      <br/>
      <button @click="signUp">Sign Up</button>
      <br/>
      <label for="code">Confirmation code:</label>
      <input type="text" id="code" v-model.lazy="confirmationCode" />
      <br/>
      <button @click="confirm">Confirm</button>
      <br/>
      <button @click="login">Login</button>
      <br/>
      <button @click="refresh">Refresh</button>
    </div>
  </main>
</template>
