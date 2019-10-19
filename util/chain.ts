import { NowRequest, NowResponse } from '@now/node';
import { RequestHandler } from 'express';

type AsyncVoid = void | Promise<void>;

type NowFunction<Req, Res> = (req: Req, res: Res) => AsyncVoid;

/**
 * Combine multiple middleware together.
 *
 * @param middlewares - Functions of form: `function(req, res, next) { ... }`, aka
 * express middlewares.
 *
 * @return - Single combined middleware
 */
function combineMiddleware(middlewares: RequestHandler[]): RequestHandler {
  return middlewares.reduce((acc, mid) => {
    return function(req, res, next): void {
      acc(req, res, err => {
        if (err) {
          return next(err);
        }

        mid(req, res, next);
      });
    };
  });
}

/**
 * Chain middlewares together, and expose them to be consumed by a `@now/node`
 * serverless function.
 *
 * @param middlewares - Functions of form: `function(req, res, next) { ... }`, aka
 * express middlewares.
 */
export function chain<Req = NowRequest, Res = NowResponse>(
  ...middlewares: RequestHandler[]
): (fn: NowFunction<Req, Res>) => NowFunction<Req, Res> {
  return function(fn: NowFunction<Req, Res>): NowFunction<Req, Res> {
    return function(req: Req, res: Res): AsyncVoid {
      // eslint-disable-next-line
      // @ts-ignore Need to cast (and verify everything works) from a
      // express.Request to a NowRequest
      return combineMiddleware(middlewares)(req, res, () => {
        fn(req, res);
      });
    };
  };
}
