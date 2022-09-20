import React, { useMemo } from 'react'
import useMangoStore from '../stores/useMangoStore'
import UiLock from './UiLock'
import ManualRefresh from './ManualRefresh'
import useOraclePrice from '../hooks/useOraclePrice'
// import DayHighLow from './DayHighLow'
import {
  getPrecisionDigits,
  perpContractPrecision,
  usdFormatter,
  commaFormatter,
  isEqual
} from '../utils'
import { useViewport } from '../hooks/useViewport'
import { breakpoints } from './TradePageGrid'
import { useTranslation } from 'next-i18next'
import SwitchMarketDropdown from './SwitchMarketDropdown'
import Tooltip from './Tooltip'
import { useWallet } from '@solana/wallet-adapter-react'
import { InformationCircleIcon } from '@heroicons/react/outline'
import usePrevious from '../hooks/usePrevious'
import {
  ArrowUpIcon,
  ArrowDownIcon,
} from '@heroicons/react/solid'
import useMarkPrice from '../hooks/useMarkPrice'

const MarkPriceComponent = React.memo<{ markPrice: number }>(
  ({ markPrice }) => {
    const previousMarkPrice: number = usePrevious(markPrice)
    console.log('Number(markPrice)?.toPrecision(6)=', Number(markPrice)?.toPrecision(6))

    return (
      <div
        className={`flex text-xxl ml-2 items-center justify-center font-bold md:w-1/3 md:text-base ${
          markPrice > previousMarkPrice
            ? `text-th-green`
            : markPrice < previousMarkPrice
            ? `text-th-red`
            : `text-th-fgd-1`
        }`}
      >
        {markPrice > previousMarkPrice && (
          <ArrowUpIcon className={`mr-1 h-4 w-4 text-th-green`} />
        )}
        {markPrice < previousMarkPrice && (
          <ArrowDownIcon className={`mr-1 h-4 w-4 text-th-red`} />
        )}
        {markPrice ? commaFormatter(Number(markPrice)?.toPrecision(6)) : '' }
      </div>
    )
  },
  (prevProps, nextProps) => isEqual(prevProps, nextProps, ['markPrice'])
)


const OraclePrice = () => {
  const oraclePrice = useOraclePrice()
  const selectedMarket = useMangoStore((s) => s.selectedMarket.current)

  const decimals = useMemo(
    () =>
      selectedMarket?.tickSize !== undefined
        ? getPrecisionDigits(selectedMarket?.tickSize)
        : null,
    [selectedMarket]
  )

  return (
    <div className="text-th-fgd-1 md:text-xs">
      {decimals && oraclePrice && selectedMarket
        ? (oraclePrice.toNumber() / 1000).toLocaleString(undefined, {
            minimumFractionDigits: decimals,
            maximumFractionDigits: decimals,
          })
        : '--'}
    </div>
  )
}

const MarketDetails = () => {
  const { t } = useTranslation('common')
  const { connected } = useWallet()
  const marketConfig = useMangoStore((s) => s.selectedMarket.config)
  const baseSymbol = marketConfig.baseSymbol
  const selectedMarketName = marketConfig.name
  const isPerpMarket = marketConfig.kind === 'perp'

  const { width } = useViewport()
  const isMobile = width ? width < breakpoints.sm : false

  const marketsInfo = useMangoStore((s) => s.marketsInfo)

  const market = useMemo(
    () => marketsInfo.find((market) => market.name === selectedMarketName),
    [marketsInfo, selectedMarketName]
  )
  const markPrice = useMarkPrice()

  return (
    <div
      className={`relative flex flex-col pb-0.0 pt-0 lg:flex-row lg:items-center lg:justify-between`}
    >
      <div className="flex flex-col lg:flex-row lg:items-center">
        <div className="hidden md:block md:pb-4 md:pr-6 lg:pb-0">
          <div className="flex items-center">
            <SwitchMarketDropdown />
          </div>
        </div>
        <div className="ml-1 -mt-5 grid grid-flow-row grid-cols-1 gap-2 md:grid-cols-3 lg:grid-flow-col lg:grid-cols-none lg:grid-rows-1 lg:gap-6">
          <div className="flex items-center justify-between md:block mt-2 ml-1 -mr-1">
           <MarkPriceComponent markPrice={markPrice} />
          </div>

          <div className="flex items-center justify-between md:block -ml-5">
            <div className="text-th-fgd-3 md:pb-0.5 md:text-[0.65rem]">
              {t('oracle-price')}
            </div>
            <OraclePrice />
          </div>
          {market ? (
            <>
              {isPerpMarket ? (
                <>
                  <div className="flex items-center justify-between md:block">
                    <div className="text-th-fgd-3 md:pb-0.5 md:text-[0.65rem]">
                      {'24h Change'}
                    </div>
                    <div className={`text-th-fgd-1 md:text-xs ${
                        market?.change24h > 0
                          ? `text-th-green`
                          : market?.change24h < 0
                          ? `text-th-red`
                          : `text-th-fgd-1`
                      }`}>
                     {`${markPrice? commaFormatter((market.change24h * markPrice).toPrecision(4)) : ''} ${market.change24h > 0? '+' : ''}${(market.change24h * 100).toFixed(2) + '%'}`}
                    </div>
                  </div>
                  
                  <div className="flex items-center justify-between md:block">
                    <div className="text-th-fgd-3 md:pb-0.5 md:text-[0.65rem]">
                      {'24h High'}
                    </div>
                    <div className="text-th-fgd-1 md:text-xs">
                      {commaFormatter(market?.high24h?.toPrecision(6))}
                    </div>
                  </div>
                  <div className="flex items-center justify-between md:block">
                    <div className="text-th-fgd-3 md:pb-0.5 md:text-[0.65rem]">
                      {'24h Low'}
                    </div>
                    <div className="text-th-fgd-1 md:text-xs">
                      {commaFormatter(market?.low24h?.toPrecision(6))}
                    </div>
                  </div>


                  <div className="flex items-center justify-between md:block">
                    <div className="text-th-fgd-3 md:pb-0.5 md:text-[0.65rem]">
                      {t('daily-volume')}
                    </div>
                    <div className="text-th-fgd-1 md:text-xs">
                      {usdFormatter(market?.volumeUsd24h, 0)}
                    </div>
                  </div>


                  <div className="flex items-center justify-between md:block">
                    <div className="flex items-center text-th-fgd-3 md:pb-0.5 md:text-[0.65rem]">
                      {t('average-funding')}
                      <Tooltip
                        content={t('tooltip-funding')}
                        placement={'bottom'}
                      >
                        <InformationCircleIcon className="ml-1.5 h-4 w-4 text-th-fgd-3 hover:cursor-help" />
                      </Tooltip>
                    </div>
                    <div className="text-th-fgd-1 md:text-xs">
                      {`${market?.funding1h.toFixed(4)}% (${(
                        market?.funding1h *
                        24 *
                        365
                      ).toFixed(2)}% APR)`}
                    </div>
                  </div>
                  <div className="flex items-center justify-between md:block">
                    <div className="text-th-fgd-3 md:pb-0.5 md:text-[0.65rem]">
                      {t('open-interest')}
                    </div>
                    <div className="flex items-center text-th-fgd-1 md:text-xs">
                      {usdFormatter(market?.openInterestUsd, 0)}
                      <Tooltip
                        content={`${market?.openInterest.toLocaleString(
                          undefined,
                          {
                            maximumFractionDigits:
                              perpContractPrecision[baseSymbol],
                          }
                        )} ${baseSymbol}`}
                        placement={'bottom'}
                      >
                        <InformationCircleIcon className="ml-1.5 h-4 w-4 text-th-fgd-3 hover:cursor-help" />
                      </Tooltip>
                    </div>
                  </div>
                </>
              ) : null}
              {/* <div className="flex items-center justify-between md:block">
                <div className="text-left text-th-fgd-3 md:pb-0.5 md:text-[0.65rem] xl:text-center">
                  {t('daily-range')}
                </div>
                <DayHighLow high={market?.high24h} low={market?.low24h} />
              </div> */}
            </>
          ) : (
            <>
              <MarketDataLoader />
              <MarketDataLoader />
              {isPerpMarket ? (
                <>
                  <MarketDataLoader />
                  <MarketDataLoader />
                  <MarketDataLoader />
                </>
              ) : null}
            </>
          )}
        </div>
      </div>
      <div className="absolute right-0 bottom-0 flex items-center justify-end space-x-2 sm:bottom-auto lg:right-3">
        {!isMobile ? (
          <div id="layout-tip">
            <UiLock />
          </div>
        ) : null}
        <div id="data-refresh-tip">
          {!isMobile && connected ? <ManualRefresh /> : null}
        </div>
      </div>
    </div>
  )
}

export default MarketDetails

export const MarketDataLoader = ({ width }: { width?: string }) => (
  <div
    className={`mt-0.5 h-8 ${
      width ? width : 'w-24'
    } animate-pulse rounded bg-th-bkg-3`}
  />
)
