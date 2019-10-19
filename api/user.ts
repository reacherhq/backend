import { NowRequest, NowResponse } from '@now/node';
import cors from 'cors';

import { User } from '../models';
import { chain, checkJwt, connectToDatabase, WithJwt } from '../util';

/**
 * Fetch a user
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
  const users = await Users.find({}).toArray();

  // Respond with a JSON string of all users in the collection
  res.status(200).json({ users });
}

export default chain<NowRequest & WithJwt, NowResponse>(cors(), checkJwt)(user);
