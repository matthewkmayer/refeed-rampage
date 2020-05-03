## Security tradeoffs and decisions made

Document purpose: describe the reasons behind security decisions made in this project.

### JWTs

JSON Web Tokens (JWTs) are used because of good support and I have experience using them. Protected endpoints require the `Authorization` header to have the format `bearer: tokenhere`.

### Refresh vs session token

https://github.com/matthewkmayer/refeed-rampage/issues/46 has more information on this topic. A brief recap: this project uses the bearer token as a session token. With the usual bearer and refresh token setup, bearer tokens should be short lived, such as fifteen minutes, and refresh tokens can live for days, months or more.

Security of the refresh token is important: if an attacker acquires it, they can impersonate the user for more than fifteen minutes. General consensus is prevent cross site scripting with an `HttpOnly` cookie, so malicious JavaScript can't access the refresh token. Cross site request forgeries are addressed with CORS options on the backend server.

The threat model of malicious JavaScript being introduced into the site is low: I'm the only user so won't be doing that and third party scripts such as Bootstrap are included with Subresource Integrity (SRI) tags. An exception is the Google fonts import that is dynamic thus cannot use SRI.

### Tracking sessions

In memory data store because I don't want to deal with two dynamo tables and don't care about losing sessions between backend deployments.

### Tradeoffs

Proper use and storage of the refresh token with an HttpOnly cookie was proving to be difficult for local development. Due to the limited attack vectors as described above, I'm okay making this convenience over security tradeoff.

As the only user of this app, I am also okay signing in again after a backend deployment.

## What if this was to be a project others could sign in to and use?

1. Use bearer and refresh tokens correctly.
2. Store refresh token in HttpOnly cookie.
3. Vendor the fonts instead of calling the Google API that cannot function with SRI.
4. Use a real data store to persist tokens across backend restarts and deploys.
