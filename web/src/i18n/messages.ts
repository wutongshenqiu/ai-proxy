import { changeStudioEn, changeStudioZh } from './catalogs/changeStudio';
import { commandCenterEn, commandCenterZh } from './catalogs/commandCenter';
import { commonEn, commonZh } from './catalogs/common';
import { providerAtlasEn, providerAtlasZh } from './catalogs/providerAtlas';
import { routeStudioEn, routeStudioZh } from './catalogs/routeStudio';
import { trafficLabEn, trafficLabZh } from './catalogs/trafficLab';

function pseudoLocalize(message: string) {
  const map: Record<string, string> = {
    a: 'à',
    b: 'ƀ',
    c: 'ç',
    d: 'ď',
    e: 'ē',
    f: 'ƒ',
    g: 'ğ',
    h: 'ħ',
    i: 'į',
    j: 'ĵ',
    k: 'ķ',
    l: 'ľ',
    m: 'ɱ',
    n: 'ñ',
    o: 'õ',
    p: 'þ',
    q: 'ʠ',
    r: 'ř',
    s: 'ş',
    t: 'ŧ',
    u: 'ū',
    v: 'ṽ',
    w: 'ŵ',
    x: 'ẋ',
    y: 'ÿ',
    z: 'ž',
    A: 'Å',
    B: 'ß',
    C: 'Ç',
    D: 'Ð',
    E: 'Ē',
    F: 'Ƒ',
    G: 'Ğ',
    H: 'Ħ',
    I: 'Ī',
    J: 'Ĵ',
    K: 'Ķ',
    L: 'Ŀ',
    M: 'Ṁ',
    N: 'Ń',
    O: 'Ø',
    P: 'Þ',
    Q: 'Ǫ',
    R: 'Ŕ',
    S: 'Š',
    T: 'Ŧ',
    U: 'Ū',
    V: 'Ṽ',
    W: 'Ŵ',
    X: 'Ẋ',
    Y: 'Ŷ',
    Z: 'Ž',
  };

  const transformed = message
    .split(/(\{\w+\})/g)
    .map((part) => {
      if (/^\{\w+\}$/.test(part)) {
        return part;
      }
      return Array.from(part, (char) => map[char] ?? char).join('');
    })
    .join('');

  return `[!! ${transformed} !!]`;
}

export const enMessages = {
  ...commonEn,
  ...commandCenterEn,
  ...trafficLabEn,
  ...providerAtlasEn,
  ...routeStudioEn,
  ...changeStudioEn,
} as const;

export const zhMessages: Record<keyof typeof enMessages, string> = {
  ...commonZh,
  ...commandCenterZh,
  ...trafficLabZh,
  ...providerAtlasZh,
  ...routeStudioZh,
  ...changeStudioZh,
};

export const pseudoMessages: Record<keyof typeof enMessages, string> = Object.fromEntries(
  Object.entries(enMessages).map(([key, value]) => [key, pseudoLocalize(value)]),
) as Record<keyof typeof enMessages, string>;

export const messageCatalogs = {
  'en-US': enMessages,
  'zh-CN': zhMessages,
  'en-XA': pseudoMessages,
} as const;

export type MessageKey = keyof typeof enMessages;
