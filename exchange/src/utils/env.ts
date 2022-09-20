//necessary for Jest, may remove later
export const isTestnet = () => {
  return import.meta.env?.VITE_IS_TESTNET
}

export const isProd = () => {
  return import.meta.env?.PROD
}
