export const createTimeLogger =
  (st = +new Date()) =>
  (msg: string) =>
    console.log(+new Date() - st + " " + msg);
