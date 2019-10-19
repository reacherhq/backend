import { IVerification } from './verification';

export interface IUser {
  auth0Id: string;
  tokens: number;
  verifications: IVerification[];
}
