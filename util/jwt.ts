import { NowRequest, NowResponse } from '@now/node';
import { verify, VerifyErrors } from 'jsonwebtoken';
import jwksClient, {
  JwksClient,
  CertSigningKey,
  RsaSigningKey
} from 'jwks-rsa';

// Set up Auth0 configuration
const authConfig = {
  domain: process.env.AUTH0_DOMAIN,
  audience: process.env.AUTH0_API_IDENTIFIER
};

// Define middleware that validates incoming bearer tokens using JWKS from
// auth0 domain
const client: JwksClient = jwksClient({
  cache: true,
  rateLimit: true,
  jwksRequestsPerMinute: 5,
  jwksUri: `https://${authConfig.domain}/.well-known/jwks.json`
});

/**
 * Verify using getKey callback
 * @see https://github.com/auth0/node-jsonwebtoken
 */
function getKey(
  header: { kid?: string },
  callback: (err: Error | null, result: string | undefined) => void
): void {
  console.log('header.kid', header.kid);
  if (!header.kid) {
    callback(new Error('No `kid` field in header'), undefined);

    return;
  }

  client.getSigningKey(header.kid, function(err, key) {
    if (err) {
      callback(err, undefined);

      return;
    }

    const signingKey =
      (key as CertSigningKey).publicKey || (key as RsaSigningKey).rsaPublicKey;

    console.log('signingKey', signingKey);
    callback(null, signingKey);
  });
}

export async function jwt(
  req: NowRequest,
  res: NowResponse
): Promise<string | object> {
  try {
    const bearerToken = req.headers.authorization;

    if (!bearerToken) {
      throw new Error('Missing Authorization header');
    }

    const token = bearerToken.replace('Bearer ', '');
    console.log('TOKEN', token, authConfig);
    const decoded = await new Promise<string | object>((resolve, reject) =>
      verify(
        token,
        getKey,
        {
          algorithms: ['RS256'],
          audience: authConfig.audience,
          issuer: `https://${authConfig.domain}/`
        },
        (err: VerifyErrors, decoded: string | object) => {
          if (err) {
            return reject(err);
          }

          resolve(decoded);
        }
      )
    );

    return decoded;
  } catch (error) {
    res.status(401).send(`Invalid token: ${error.message}`);

    throw error;
  }
}
