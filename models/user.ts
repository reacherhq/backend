import { Verification } from './verification';

export interface User {
  auth0Id: string;
  credits: number;
  verifications: Verification[];
}
