const tenantId = "35c6fe40-0ec0-46b6-98c6-213ad4de6650";
const authority = `https://sociobotcustomers.ciamlogin.com/${tenantId}/`;
const clientId = "25c704f4-465a-47af-80ab-2c489466b697";
const scopes = ["openid", "profile", "email"];

let clientPromise: Promise<import("@azure/msal-browser").PublicClientApplication> | null = null;

async function client() {
  clientPromise ??= import("@azure/msal-browser").then(async ({ PublicClientApplication }) => {
    const instance = new PublicClientApplication({
      auth: {
        clientId,
        authority,
        knownAuthorities: ["sociobotcustomers.ciamlogin.com"],
        redirectUri: `${window.location.origin}/auth/callback`,
        postLogoutRedirectUri: `${window.location.origin}/`
      },
      cache: { cacheLocation: "sessionStorage" }
    });
    await instance.initialize();
    const redirect = await instance.handleRedirectPromise();
    if (redirect?.account) instance.setActiveAccount(redirect.account);
    return instance;
  });
  return clientPromise;
}

export async function accessToken(): Promise<string | null> {
  const testToken = sessionStorage.getItem("cct:test-access-token");
  if (testToken) return testToken;
  const instance = await client();
  const account = instance.getActiveAccount() ?? instance.getAllAccounts()[0];
  if (!account) return null;
  const result = await instance.acquireTokenSilent({ account, scopes });
  return bearerFromAuthenticationResult(result);
}

export function bearerFromAuthenticationResult(result: { accessToken: string; idToken: string }) {
  // OIDC-only requests may not return an API access token. The ID token is
  // still an RS256 JWT for this SPA's audience and is validated in full by
  // the API before its stable oid claim is used.
  return result.accessToken || result.idToken;
}

export async function signedIn(): Promise<boolean> {
  return Boolean(await accessToken().catch(() => null));
}

export async function signIn() {
  const instance = await client();
  await instance.loginRedirect({ scopes, redirectStartPage: `${window.location.origin}/app` });
}

export async function signOut() {
  sessionStorage.removeItem("cct:test-access-token");
  localStorage.removeItem("cct:workspace-key");
  const instance = await client();
  const account = instance.getActiveAccount() ?? instance.getAllAccounts()[0];
  if (account) await instance.logoutRedirect({ account });
}

export async function finishSignIn(): Promise<boolean> {
  const instance = await client();
  return Boolean(instance.getActiveAccount() ?? instance.getAllAccounts()[0]);
}
