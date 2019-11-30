import { Db, MongoClient } from 'mongodb';
import { parse } from 'url';

// Create cached connection variable
let cachedDb: Db | null = null;

/**
 * A function for connecting to MongoDB, taking a single paramater of the
 * connection string
 * @param uri - MongoDB connection string
 */
export async function connectToDatabase(uri: string): Promise<Db> {
  // If the database connection is cached,
  // use it instead of creating a new connection
  if (cachedDb) {
    return cachedDb;
  }

  // If no connection is cached, create a new one
  const client = await MongoClient.connect(uri, {
    /*
    Buffering allows Mongoose to queue up operations if MongoDB
    gets disconnected, and to send them upon reconnection.
    With serverless, it is better to fail fast when not connected.
  */
    bufferMaxEntries: 0,
    useNewUrlParser: true
  });

  // Select the database through the connection,
  // using the database path of the connection string
  const parsed = parse(uri);
  if (!parsed.pathname) {
    throw new Error('Cannot find pathname in connection string');
  }
  const db = client.db(parsed.pathname.substr(1));

  // Cache the database connection and return the connection
  cachedDb = db;
  return db;
}
