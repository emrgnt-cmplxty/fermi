import ArrowDropDownIcon from '@mui/icons-material/ArrowDropDown'
import ArrowDropUpIcon from '@mui/icons-material/ArrowDropUp'
import StarIcon from '@mui/icons-material/Star'
import {
  Box,
  Card,
  CardContent,
  Grid,
  Skeleton,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Typography,
} from '@mui/material'
import { AssetDisplay } from 'components/AssetDisplay'
import { useSelectMarketDataType } from 'hooks/react-query/useMarketOverview'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext, useMemo } from 'react'
import { Cell, Column, Row, useSortBy, useTable } from 'react-table'
import { formatNumber } from 'utils/formatters'
import { MarketData, MarketSymbol, MarketType } from 'utils/globals'

const FAVORITE_COLUMN_NAME = ' '

interface MarketTableProps {
  type: MarketType
}

const MarketTable = ({ type }: MarketTableProps) => {
  const { settingsState, settingsDispatch, isLoading } =
    useContext(SettingsContext)

  const isFav = (symbol: MarketSymbol) => {
    return settingsState?.favorites?.includes(symbol)
  }

  const handleModifyFavorites = (
    _: React.ChangeEvent<HTMLInputElement>,
    newSymbol: MarketSymbol,
  ) => {
    const isIncluded = settingsState?.favorites?.includes(newSymbol)
    const newFavorites = !isIncluded
      ? settingsState?.favorites?.concat(newSymbol)
      : settingsState?.favorites?.filter((ele) => {
          return ele !== newSymbol
        })
    settingsDispatch({
      type: 'updateSetting',
      payload: {
        favorites: newFavorites,
      },
    })
  }

  const columns: Column<MarketData>[] = useMemo(
    () => [
      {
        Header: FAVORITE_COLUMN_NAME,
        maxWidth: 25,
        minWidth: 25,
        width: 25,
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: MarketData }) => (
          <StarIcon color={isFav(value.symbol) ? 'primary' : 'disabled'} />
        ),
      },
      {
        Header: 'Asset',
        maxWidth: 200,
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: MarketData }) => (
          <Grid>
            <AssetDisplay
              symbol={value.baseSymbol}
              rightLabel={`${String(value.baseSymbol)}${
                value.type === 'futures' ? '-PERP' : ''
              }`}
              rightMetaLabel={value.baseName}
              farRightLabel={
                value.type === 'spot' ? `/${String(value.quoteSymbol)}` : ''
              }
            />
          </Grid>
        ),
      },
      {
        Header: 'Last Price',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: MarketData }) => (
          <Typography variant="h6">
            {formatNumber(
              Number((value?.price || 0).toFixed(value.suggestedDecimals)),
            )}
          </Typography>
        ),
      },
      {
        Header: '24h Change',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: MarketData }) => (
          <Typography
            sx={{
              color:
                (value?.dailyChange || 0) < 0
                  ? 'tradeColors.ask'
                  : 'tradeColors.bid',
            }}
            variant="h6"
          >
            {`${(100 * (value?.dailyChange || 0)).toFixed(2)}%`}
          </Typography>
        ),
      },
      {
        Header: '24h Volume',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: MarketData }) => (
          <Typography variant="h6">
            {formatNumber(
              Number(
                (value?.dailyVolume || 0).toFixed(value.suggestedDecimals),
              ),
            )}
          </Typography>
        ),
      },
      {
        Header: '24h Low',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: MarketData }) => (
          <Typography variant="h6">
            {formatNumber(
              Number((value?.dailyLow || 0).toFixed(value.suggestedDecimals)),
            )}
          </Typography>
        ),
      },
      {
        Header: '24h High',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: MarketData }) => (
          <Typography variant="h6">
            {formatNumber(
              Number((value?.dailyHigh || 0).toFixed(value.suggestedDecimals)),
            )}
          </Typography>
        ),
      },
      {
        Header: 'Open Interest',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: MarketData }) =>
          `$${formatNumber(
            value.openInterest.toFixed(value.suggestedDecimals),
          )}`,
      },
    ],
    [settingsState, type],
  )
  const isLoadingMarketData = false
  const { data: selectedMarketData } = useSelectMarketDataType(type, {
    refetchInterval: 1000,
  })
  // TODO - why do I need to wrap in || [] for useSelectMarketDataType result?
  const data = useMemo(() => selectedMarketData || [], [selectedMarketData])

  const { getTableProps, headerGroups, prepareRow, rows } = useTable(
    {
      columns,
      data,
      //autoResetPage: false,
    },
    useSortBy, //usePagination,
  )
  const onRowClick = (
    event: React.MouseEvent<HTMLTableCellElement, MouseEvent>,
    row: Row<MarketData>,
    column: string,
  ) => {
    if (column === FAVORITE_COLUMN_NAME) {
      handleModifyFavorites(event, row.symbol)
    } else {
      window.location.href = `/app/${
        selectedMarketData?.find((ele) => {
          return row.symbol === ele.symbol
        }).type
      }/${row.symbol}`
    }
  }
  return (
    <Card>
      <CardContent>
        <Typography variant={'h6'} fontWeight={700}>
          {type && type[0].toUpperCase() + type.slice(1)}
        </Typography>
      </CardContent>
      <TableContainer>
        <Table stickyHeader aria-label="sticky table" {...getTableProps()}>
          <TableHead>
            {headerGroups.map((headerGroup) => {
              const { key: headerGroupKey, ...getHeaderGroupProps } =
                headerGroup.getHeaderGroupProps()
              return (
                <TableRow key={headerGroupKey} {...getHeaderGroupProps}>
                  {headerGroup.headers.map((column) => {
                    const { key: headerKey, ...getHeaderProps } =
                      column.getHeaderProps(column.getSortByToggleProps({}))
                    return (
                      <TableCell key={headerKey} {...getHeaderProps}>
                        <Grid
                          container
                          direction="row"
                          alignItems="center"
                          sx={{ mt: -2, mb: -2 }}
                          style={{
                            maxWidth: column?.maxWidth,
                            width: column?.width,
                          }}
                        >
                          <Grid item>{column.render('Header')}</Grid>
                          {column.Header != FAVORITE_COLUMN_NAME && (
                            <Grid item>
                              <Grid container direction="column">
                                <Grid item>
                                  <ArrowDropUpIcon
                                    sx={{ mt: 1, pt: -1 }}
                                    color={
                                      column.isSorted && column.isSortedDesc
                                        ? 'primary'
                                        : undefined
                                    }
                                  />
                                </Grid>
                                <Grid item sx={{ mt: -2.5 }}>
                                  <ArrowDropDownIcon
                                    color={
                                      column.isSorted && !column.isSortedDesc
                                        ? 'primary'
                                        : undefined
                                    }
                                  />
                                </Grid>
                              </Grid>
                            </Grid>
                          )}
                        </Grid>
                      </TableCell>
                    )
                  })}
                </TableRow>
              )
            })}
          </TableHead>
          <TableBody>
            {isLoadingMarketData ? (
              <TableRow>
                <TableCell>
                  <Skeleton width={50} />
                </TableCell>
                {Array(5)
                  .fill(0)
                  .map((x) => (
                    <TableCell key={x}>
                      <Box
                        width={150}
                        textAlign="right"
                        justifyContent="flex-end"
                        display="flex"
                      >
                        <Skeleton width={50} />
                      </Box>
                    </TableCell>
                  ))}
              </TableRow>
            ) : (
              rows.map((row: Row<MarketData>) => {
                prepareRow(row)
                const { key: rowKey, ...rowProps } = row.getRowProps()
                return (
                  <TableRow
                    key={rowKey}
                    {...rowProps}
                    style={{ cursor: 'pointer' }}
                    hover
                  >
                    {row.cells.map((cell: Cell<MarketData>) => {
                      const { key: cellKey, ...cellProps } = cell.getCellProps()
                      return (
                        <TableCell
                          key={cellKey}
                          {...cellProps}
                          onClick={(event) => {
                            onRowClick(event, row.original, cell.column.Header)
                          }}
                          style={{
                            maxWidth: cell?.column?.maxWidth,
                            width: cell?.column?.width,
                          }}
                        >
                          {cell.render('Cell')}
                        </TableCell>
                      )
                    })}
                  </TableRow>
                )
              })
            )}
          </TableBody>
        </Table>
      </TableContainer>
    </Card>
  )
}

export default MarketTable
