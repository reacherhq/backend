import { NowRequest, NowResponse } from '@now/node';
import fetch from 'node-fetch';

import { chain, rateLimit } from '../../util';

/**
 * Endpoint for a demo verification
 */
async function verifyDemo(req: NowRequest, res: NowResponse): Promise<void> {
  const toEmail = req.query.toEmail;

  if (!toEmail) {
    res.status(422).send('Missing `toEmail` query param');

    return;
  }

  const response = await fetch(
    `${process.env.SERVERLESS_VERIFIER_ENDPOINT}/?to_email=${toEmail}`
  );

  // Respond with a JSON string of all users in the collection
  res.status(200).json(await response.json());
}

export default chain(rateLimit)(verifyDemo);
