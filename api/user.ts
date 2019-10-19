import { NowRequest, NowResponse } from '@now/node';
import cors from 'micro-cors';

import { IUser } from '../models';
import { connectToDatabase, jwt } from '../util';

/**
 * Fetch a user
 */
async function user(req: NowRequest, res: NowResponse) {
  if (req.method !== 'GET') {
    res.status(200).send('ok');
    return;
  }

  const decoded = await jwt(req, res);

  console.log('AAA', decoded);

  if (!process.env.MONGODB_ATLAS_URI) {
    throw new Error('MONGODB_ATLAS_URI is not defined');
  }

  // Get a database connection, cached or otherwise,
  // using the connection string environment variable as the argument
  const db = await connectToDatabase(process.env.MONGODB_ATLAS_URI);

  const User = db.collection<IUser>('User');

  // Select the users collection from the database
  const users = await User.find({}).toArray();

  // Respond with a JSON string of all users in the collection
  res.status(200).json({ users });
}

export default cors()(user);
