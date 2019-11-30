import { NowRequest, NowResponse } from '@now/node';

import { User } from '../models';
import { chain, checkJwt, cors, connectToDatabase, WithJwt } from '../util';

/**
 * Fetch or create a user
 */
async function user(
  req: NowRequest & WithJwt,
  res: NowResponse
): Promise<void> {
  if (!process.env.MONGODB_ATLAS_URI) {
    throw new Error('MONGODB_ATLAS_URI is not defined');
  }

  // Get a database connection, cached or otherwise,
  // using the connection string environment variable as the argument
  const db = await connectToDatabase(process.env.MONGODB_ATLAS_URI);

  const Users = db.collection<User>('User');

  // Select the users collection from the database
  const currentUser = await Users.findOneAndUpdate(
    {
      auth0Id: req.auth0.sub
    },
    // Create a new document is none exists
    { $set: { auth0Id: req.auth0.sub, credits: 100, verifications: [] } },
    { upsert: true }
  );

  // Respond with a JSON of current user
  res.status(200).json(currentUser.value);
}

export default chain<NowRequest & WithJwt>(cors, checkJwt)(user);
