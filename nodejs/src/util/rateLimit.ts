import expressLateLimit from 'express-rate-limit';

export const rateLimit = expressLateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100 // Limit each IP to 100 requests per windowMs
});
