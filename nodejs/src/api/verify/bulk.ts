import { NowRequest, NowResponse } from '@now/node';
import fetch from 'node-fetch';

import { chain, checkJwt, cors, rateLimit, WithJwt } from '../../util';

/**
 * Endpoint for a bulk email verification
 */
async function verifyBulk(
  req: NowRequest & WithJwt,
  res: NowResponse
): Promise<void> {
  if (typeof req.body.name !== 'string') {
    res.status(422).send('Incorrect `name` field');

    return;
  }
  if (!Array.isArray(req.body.emails)) {
    res.status(422).send('Incorrect `emails` field');

    return;
  }

  const { emails, name }: { emails: string[]; name: string } = req.body;

  const allChecks = emails.map(email =>
    fetch(
      `${process.env.SERVERLESS_VERIFIER_ENDPOINT}/?to_email=${email}`
    ).then(response => response.json())
  );

  const allEmails = await Promise.all(allChecks);

  // Respond with a JSON string of all users in the collection
  res.status(200).json({
    name,
    report: allEmails
  });
}

export default chain<NowRequest & WithJwt>(rateLimit, cors, checkJwt)(
  verifyBulk
);
