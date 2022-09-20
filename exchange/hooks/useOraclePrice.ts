import { I80F48 } from '@gdexorg/ifm-client'
import { useCallback, useEffect, useState } from 'react'
import useMangoStore from '../stores/useMangoStore'
import { connectionSelector, mangoGroupSelector } from '../stores/selectors'

export default function useOraclePrice(): I80F48 | null {
  const mangoGroup = useMangoStore((s) => s.selectedMangoGroup.current)
  const mangoCache = useMangoStore((s) => s.selectedMangoGroup.cache)
  const selectedMarket = useMangoStore((s) => s.selectedMarket.config)
  const [oraclePrice, setOraclePrice] = useState<any>(null)

  // getPrice(tokenIndex: number, mangoCache: MangoCache): I80F48 {
  //   if (tokenIndex === QUOTE_INDEX) return ONE_I80F48;
  //   const decimalAdj = new Big(10).pow(
  //     this.getTokenDecimals(tokenIndex) - this.getTokenDecimals(QUOTE_INDEX),
  //   );

  //   return I80F48.fromBig(
  //     mangoCache.priceCache[tokenIndex]?.price.toBig().mul(decimalAdj),
  //   );

  const fetchOraclePrice = useCallback(() => {
    if (mangoGroup && mangoCache) {
      setOraclePrice(null)
      let marketIndex = 0
      if (selectedMarket.kind === 'spot') {
        marketIndex = mangoGroup.getSpotMarketIndex(selectedMarket.publicKey)
      } else {
        marketIndex = mangoGroup.getPerpMarketIndex(selectedMarket.publicKey)
      }
      setOraclePrice(mangoGroup.getPrice(marketIndex, mangoCache))
    }
  }, [mangoGroup, selectedMarket, mangoCache])

  useEffect(() => {
    fetchOraclePrice()
  }, [fetchOraclePrice])

  return oraclePrice
}
