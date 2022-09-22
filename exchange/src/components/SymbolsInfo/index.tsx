import KeyboardArrowRightIcon from '@mui/icons-material/KeyboardArrowRight'
import { Box, Grid, List, ListItemButton } from '@mui/material'
import { AssetDisplay } from 'components/AssetDisplay'
import { useSelectMarketDataType } from 'hooks/react-query/useMarketOverview'
import { useState } from 'react'
import { NavLink as RouterLink } from 'react-router-dom'
import { BaseSymbol, MarketData, MarketType } from 'utils/globals'

interface SymbolsInfoProps {
  width: number
  type: MarketType
  onClick: React.MouseEventHandler<HTMLAnchorElement>
}

export default function SymbolsInfo({
  width,
  type,
  onClick,
}: SymbolsInfoProps) {
  const { data: selectedMarketData } = useSelectMarketDataType(type)
  const [activeSymbol, setActiveSymbol] = useState<BaseSymbol | null>(null)

  return (
    <Grid
      sx={{
        backgroundColor: 'background.navmenu',
        borderRadius: 4,
        width: width,
        minWidth: width,
        maxHeight: 500,
        overflow: 'auto',
      }}
    >
      <List>
        {(selectedMarketData || []).map((pair) => {
          const onEnter = () => {
            setActiveSymbol(pair.baseSymbol)
          }
          const onLeave = () => {
            setActiveSymbol(null)
          }
          const isActive = activeSymbol === pair.baseSymbol
          return (
            <ListItemButton
              key={`spot-list-${String(pair.baseSymbol)}`}
              onMouseEnter={onEnter}
              onMouseLeave={onLeave}
              onClick={onClick}
              component={RouterLink}
              to={`/app/${type === 'spot' ? 'trade' : type}/${String(
                pair.baseSymbol,
              )}-${type === 'futures' ? 'PERP' : String(pair.quoteSymbol)}`}
            >
              <Grid item sx={{ pb: 1, pt: 1, pl: 2 }}>
                <AssetDisplay
                  symbol={pair.baseSymbol}
                  rightLabel={`${String(pair.baseSymbol)}${
                    type === 'futures' ? '-PERP' : ''
                  }`}
                  rightMetaLabel={pair.baseName}
                  farRightLabel={
                    type === 'spot' ? `/${String(pair.quoteSymbol)}` : ''
                  }
                  isActive={isActive}
                />
              </Grid>
              {isActive && (
                <>
                  <Grid sx={{ flexGrow: 1 }} />
                  <KeyboardArrowRightIcon color={'primary'} />
                </>
              )}
            </ListItemButton>
          )
        })}
      </List>
    </Grid>
  )
}
