const g = globalThis as any;
g["_HOTMEMO_SALT_"] ??=
  new Date().getTime().toString(36) + Math.random().toString(36).slice(2);

export const hotMemo = async <Args extends readonly unknown[], R>(
  fn: (...args: Args) => R,
  args: readonly [...Args] = [] as unknown as readonly [...Args],
  key = `_HOTMEMO_${g["_HOTMEMO_SALT_"]}_${
    String(fn) + "(" + JSON.stringify(args) + ")"
  }`,
): Promise<Awaited<R>> => (g[key] ??= (await fn(...(args as Args))) as any);
