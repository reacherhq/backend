import { NowRequest, NowResponse } from '@now/node';
import rateLimit from 'express-rate-limit';
import fetch from 'node-fetch';

import { chain } from '../../util';

/**
 * Endpoint for a demo verification
 */
async function verifyDemo(req: NowRequest, res: NowResponse): Promise<void> {
  const toEmail = req.query.toEmail;

  const response = await fetch(
    `${process.env.SERVERLESS_VERIFIER_ENDPOINT}/?to_email=${toEmail}`
  );

  // Respond with a JSON string of all users in the collection
  res.status(200).json(await response.json());
}

export default chain(
  rateLimit({
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: 100 // limit each IP to 100 requests per windowMs
  })
)(verifyDemo);
