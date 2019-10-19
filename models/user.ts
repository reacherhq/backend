import { Verification } from './verification';

export interface User {
  auth0Id: string;
  tokens: number;
  verifications: Verification[];
}
