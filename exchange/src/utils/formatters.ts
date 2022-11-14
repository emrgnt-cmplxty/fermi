import { ChainId } from 'utils/globals'

export const numberWithCommas = (x: number) => {
  const parts = x.toString().split('.')
  parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, ',')
  return parts.join('.')
}

export const numberToMinPrecision = (
  x: number | string,
  minPrecision: number,
) => {
  const parts = x.toString().split('.')

  if (parts.length === 1) {
    return parts
  }
  if (minPrecision < 1) {
    const decimals = minPrecision.toString().length - 2
    parts[1] =
      parts[1].length >= decimals ? parts[1].slice(0, decimals) : parts[1]
    return parts.join('.')
  }
}

// 33 => "33", undefined => "--""
export const blankify = (value: unknown, condition?: boolean) => {
  if (!condition || (condition === undefined && !!value)) {
    return '--'
  }
  return value
}

export const roundUp = (num: number, upRound: number) => {
  return Math.ceil(num / upRound) * upRound
}

export const roundDown = (num: number, downRound: number) => {
  return Math.floor(num / downRound) * downRound
}

// 'hey' -> 'Hey', 'hey there' -> 'Hey There'
export const toTitleCase = (str: string) => {
  return str?.replace(/\w\S*/g, function (txt) {
    return txt.charAt(0).toUpperCase() + txt.substr(1).toLowerCase()
  });
}

// 'KWkhp3cBfZs5f/c/rdExeUNC7j7jnVcVv9V1g/WDrlg=' -> 'KWkh....rlg='
export const maskHash = (hash: string) => {
  return hash.slice(0, 4) + '...' + hash.slice(hash.length - 4, hash.length)
}
// 220, 120, 100 -> #XXXXXX
export const rgbToHex = (r: number, g: number, b: number) => {
  function componentToHex(c) {
    const hex = c.toString(16)
    return hex.length == 1 ? '0' + hex : hex
  }
  return '#' + componentToHex(r) + componentToHex(g) + componentToHex(b)
}

export const toHHMMSS = (element: number) => {
  const sec_num = parseInt(element, 10) // don't forget the second param
  let hours = Math.floor(sec_num / 3600)
  let minutes = Math.floor((sec_num - hours * 3600) / 60)
  let seconds = sec_num - hours * 3600 - minutes * 60

  if (hours < 10) {
    hours = '0' + hours
  }
  if (minutes < 10) {
    minutes = '0' + minutes
  }
  if (seconds < 10) {
    seconds = '0' + seconds
  }
  return hours + ':' + minutes + ':' + seconds
}

export function hexToAscii(_hex: string): string {
  const hex = _hex.toString()
  let str = ''
  for (let n = 0; n < hex.length; n += 2) {
    str += String.fromCharCode(parseInt(hex.substr(n, 2), 16))
  }
  return str
}

export interface CancelablePromise<T = unknown> {
  promise: Promise<T>
  cancel: () => void
}

export const makeCancelable = <T>(promise: Promise<T>) => {
  let hasCanceled_ = false

  const wrappedPromise = new Promise<T>((resolve, reject) => {
    promise.then(
      (val) => (hasCanceled_ ? reject({ isCanceled: true }) : resolve(val)),
      (error) => (hasCanceled_ ? reject({ isCanceled: true }) : reject(error)),
    )
  })

  return {
    promise: wrappedPromise,
    cancel() {
      hasCanceled_ = true
    },
  }
}

export const optimizedPath = (currentChainId: ChainId) => {
  return (
    // ||
    // currentChainId === ChainId.optimism_kovan
    (currentChainId === ChainId.arbitrum_one ||
    currentChainId === ChainId.arbitrum_rinkeby || currentChainId === ChainId.optimism)
  );
}

export const formatNumber = (arg: number, nDigis = 2): string => {
  return new Intl.NumberFormat('en-US', {
    minimumFractionDigits: nDigis,
    maximumFractionDigits: nDigis,
  }).format(arg)
}
