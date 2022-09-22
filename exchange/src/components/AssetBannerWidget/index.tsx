import { Grid, Typography } from '@mui/material'
import { AssetDisplay } from 'components/AssetDisplay'
import SymbolsInfo from 'components/SymbolsInfo'
import { useSelectMarketDataSymbol } from 'hooks/react-query/useMarketOverview'
import { HoverMenuStyled } from 'layouts/app/NavSection'
import {
  bindHover,
  bindMenu,
  usePopupState,
} from 'material-ui-popup-state/hooks'
import { CurrentMarketContext } from 'providers/CurrentMarketProvider'
import { useContext } from 'react'
import { formatNumber } from 'utils/formatters'
import { toHHMMSS } from 'utils/formatters'

const AssetBanner = () => {
  const {
    currentMarketState: { marketSymbol, recentTrades, type },
  } = useContext(CurrentMarketContext)
  const { data: selectedMarket } = useSelectMarketDataSymbol(marketSymbol, {
    refetchInterval: 1000,
  })

  const popupState = usePopupState({
    variant: 'popover',
    popupId: 'AssetBannerMenu',
  })
  const handleClose = () => {
    popupState.close()
  }

  const timeToFunding = selectedMarket?.nextFundingTime - new Date()
  const name =
    selectedMarket?.type === 'futures'
      ? `${selectedMarket?.baseName} Futures`
      : `${selectedMarket?.baseName} Spot`
  const spot = recentTrades?.[0]?.price || selectedMarket?.price
  const mark = selectedMarket?.markPrice
  const index = selectedMarket?.indexPrice
  const delta = selectedMarket?.dailyChange
  const funding = selectedMarket?.fundingRate
  const fundingTimer = toHHMMSS(timeToFunding / 1000)
  const openInterest = selectedMarket?.openInterest
  const high = selectedMarket?.dailyHigh
  const low = selectedMarket?.dailyLow
  const volume = selectedMarket?.dailyVolume

  return (
    <Grid>
      <Grid className="header" />
      <Grid
        sx={{ backgroundColor: 'background.default' }}
        container
        direction="row"
        alignItems="center"
        spacing={2}
      >
        <Grid item {...bindHover(popupState)}>
          <AssetDisplay
            symbol={selectedMarket?.baseSymbol}
            rightLabel={selectedMarket?.baseSymbol}
            rightMetaLabel={name}
            isActive={popupState.isOpen}
            isDropDown={true}
          />
          <HoverMenuStyled {...bindMenu(popupState)}>
            <SymbolsInfo width={400} type={type} onClick={handleClose} />
          </HoverMenuStyled>
        </Grid>
        <Grid
          container
          style={{ maxWidth: 135 }}
          sx={{ mt: 2 }}
          direction="row"
          justifyContent="center"
        >
          <Grid item>
            {spot !== undefined && (
              <Grid item sx={{ pl: 3 }}>
                <Typography variant="h4">
                  {formatNumber(spot, selectedMarket?.suggestedDecimals)}
                </Typography>
              </Grid>
            )}
          </Grid>
        </Grid>
        {delta !== undefined && (
          <Grid item>
            <Typography
              variant="h6"
              color={delta > 0 ? 'tradeColors.bid' : 'tradeColors.ask'}
            >
              {`${delta > 0 ? '+' : '-'}${(delta * 100).toFixed(2)}%`}
            </Typography>
          </Grid>
        )}
        {mark !== undefined && (
          <Grid item>
            <Grid container direction="column">
              <Grid item>
                <Typography variant="caption" color="textSecondary">
                  Mark
                </Typography>
              </Grid>
              <Grid item>
                <Typography variant="caption">
                  {formatNumber(mark, selectedMarket?.suggestedDecimals)}
                </Typography>
              </Grid>
            </Grid>
          </Grid>
        )}
        {index !== undefined && (
          <Grid item>
            <Grid container direction="column">
              <Grid item>
                <Typography variant="caption" color="textSecondary">
                  Index
                </Typography>
              </Grid>
              <Grid item>
                <Typography variant="caption">
                  {formatNumber(index, selectedMarket?.suggestedDecimals)}
                </Typography>
              </Grid>
            </Grid>
          </Grid>
        )}
        {funding !== undefined && (
          <Grid item>
            <Grid container direction="column">
              <Grid item>
                <Typography variant="caption" color="textSecondary">
                  Predcted Funding
                </Typography>
              </Grid>
              <Grid item>
                <Grid container direction="row">
                  <Grid item>
                    <Typography variant="caption">
                      {`${(funding * 100).toFixed(4)}%`}
                    </Typography>
                  </Grid>
                  <Grid item sx={{ ml: 1 }}>
                    <Typography variant="caption" color="textSecondary">
                      {' in ' + fundingTimer}
                    </Typography>
                  </Grid>
                </Grid>
              </Grid>
            </Grid>
          </Grid>
        )}
        {openInterest !== undefined && (
          <Grid item>
            <Grid container direction="column">
              <Grid item>
                <Typography variant="caption" color="textSecondary">
                  Open Interest
                </Typography>
              </Grid>
              <Grid item>
                <Typography variant="caption">
                  {`${formatNumber(openInterest)} ${
                    selectedMarket?.baseSymbol
                  }`}
                </Typography>
              </Grid>
            </Grid>
          </Grid>
        )}
        {high !== undefined && (
          <Grid item>
            <Grid container direction="column">
              <Grid item>
                <Typography variant="caption" color="textSecondary">
                  24h High
                </Typography>
              </Grid>
              <Grid item>
                <Typography variant="caption">
                  {formatNumber(high, selectedMarket?.suggestedDecimals)}
                </Typography>
              </Grid>
            </Grid>
          </Grid>
        )}
        {low !== undefined && (
          <Grid item>
            <Grid container direction="column">
              <Grid item>
                <Typography variant="caption" color="textSecondary">
                  24h Low
                </Typography>
              </Grid>
              <Grid item>
                <Typography variant="caption">
                  {formatNumber(low, selectedMarket?.suggestedDecimals)}
                </Typography>
              </Grid>
            </Grid>
          </Grid>
        )}
        {volume !== undefined && (
          <Grid item>
            <Grid container direction="column">
              <Grid item>
                <Typography variant="caption" color="textSecondary">
                  24h Volume
                </Typography>
              </Grid>
              <Grid item>
                <Typography variant="caption">
                  {`$${formatNumber(volume, 1)}`}
                </Typography>
              </Grid>
            </Grid>
          </Grid>
        )}
      </Grid>
    </Grid>
  )
}

export default AssetBanner
