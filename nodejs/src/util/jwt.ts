import expressJwt from 'express-jwt';
import { expressJwtSecret } from 'jwks-rsa';

/**
 * Once the middleware passes, the `req` object will have an `auth0` field with
 * the following properties
 */
export interface WithJwt {
  auth0: {
    iss: string;
    sub: string;
    aud: string[];
    iat: number;
    exp: number;
    azp: string;
    scope: string;
  };
}

// Set up Auth0 configuration
const authConfig = {
  domain: process.env.AUTH0_DOMAIN,
  audience: process.env.AUTH0_API_IDENTIFIER
};

// Define middleware that validates incoming bearer tokens
// using JWKS from YOUR_DOMAIN
export const checkJwt = expressJwt({
  algorithm: ['RS256'],
  audience: authConfig.audience,
  issuer: `https://${authConfig.domain}/`,
  requestProperty: 'auth0',
  secret: expressJwtSecret({
    cache: true,
    rateLimit: true,
    jwksRequestsPerMinute: 5,
    jwksUri: `https://${authConfig.domain}/.well-known/jwks.json`
  })
});
