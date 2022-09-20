import {
  Box,
  Button,
  Card,
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
import { PaginationFooter } from 'components/PaginationFooter'
import CloseModal from 'components/PositionCloseModal'
import { useUserData } from 'hooks/react-query/useUser'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext, useMemo, useState } from 'react'
import { Cell, Column, Row, usePagination, useTable } from 'react-table'
import { formatNumber } from 'utils/formatters'
import { MarketSymbol, PositionData } from 'utils/globals'

const Positions = () => {
  const { settingsState, settingsDispatch, isLoading } =
    useContext(SettingsContext)
  const { data: userData } = useUserData({ refetchInterval: 1000 })
  const [closeType, setCloseType] = useState('MARKET')
  const [openClose, setOpenClose] = useState(false)
  const [closeSymbol, setCloseSymbol] = useState<MarketSymbol>('BTC-PERP')

  const columns: Column<PositionData>[] = useMemo(
    () => [
      {
        Header: 'Contracts',
        maxWidth: 200,
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: PositionData }) => (
          <Grid>
            {
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
            }
            {/* {value.marketSymbol} */}
          </Grid>
        ),
      },
      {
        Header: 'Amount',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: PositionData }) => (
          <Grid
            sx={{
              color: value.size > 0 ? 'tradeColors.bid' : 'tradeColors.ask',
            }}
          >
            <Typography variant="h7">{formatNumber(value.size, 0)}</Typography>
          </Grid>
        ),
      },
      {
        Header: 'Value',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: PositionData }) => (
          <Grid>
            <Typography variant="h7">
              {`$${formatNumber(value.notional, 0)}`}
            </Typography>
          </Grid>
        ),
      },

      {
        Header: 'Entry Price',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: PositionData }) => (
          <Grid>
            <Typography variant="h7">
              {formatNumber(value.entryPrice, value.suggestedDecimals)}
            </Typography>
          </Grid>
        ),
      },
      {
        Header: 'Mark Price',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: PositionData }) => (
          <Grid>
            <Typography variant="h7">
              {formatNumber(value.markPrice, value.suggestedDecimals)}
            </Typography>
          </Grid>
        ),
      },
      {
        Header: 'Liq. Price',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: PositionData }) => (
          <Grid>
            <Typography variant="h7">
              {`${formatNumber(value.estLiqPrice, value.suggestedDecimals)}`}
            </Typography>
          </Grid>
        ),
      },
      {
        Header: 'Margin (%)',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: PositionData }) => (
          <Grid>
            <Typography variant="h7">
              {`$${formatNumber(value.absMarginUsed, 1)} USD (${
                value.percMarginUsed
              }%)`}
            </Typography>
          </Grid>
        ),
      },
      {
        Header: 'PnL ($)',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: PositionData }) => (
          <Grid
            sx={{
              color: value.pnlQuote > 0 ? 'tradeColors.bid' : 'tradeColors.ask',
            }}
          >
            <Typography variant="h7">
              {`$${formatNumber(value.pnlQuote, 0)}`}
            </Typography>
          </Grid>
        ),
      },
      {
        Header: 'TP / SL',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: PositionData }) => (
          <Grid>
            {value.takeProfit || value.stopLoss ? (
              <Grid container direction="column">
                <Grid item sx={{ color: 'tradeColors.bid' }}>
                  {value.takeProfit}
                </Grid>
                <Grid item sx={{ color: 'tradeColors.ask' }}>
                  {value.stopLoss}
                </Grid>
              </Grid>
            ) : (
              <Button variant="outlined" style={{ maxHeight: 30, padding: 0 }}>
                Add
              </Button>
            )}
          </Grid>
        ),
      },
      {
        Header: 'Trailing Stop',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: PositionData }) => (
          <Grid>
            <Button variant="outlined" style={{ maxHeight: 30, padding: 0 }}>
              Add
            </Button>
          </Grid>
        ),
      },
      {
        Header: 'Adjust Position',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: PositionData }) => (
          <Grid container direction="row" spacing={1}>
            <Grid item>
              <Button
                variant="outlined"
                style={{ maxHeight: 30, padding: 0 }}
                onClick={() => {
                  setOpenClose(true)
                  setCloseSymbol(value.marketSymbol)
                  setCloseType('LIMIT')
                }}
              >
                Limit
              </Button>
            </Grid>
            <Grid item>
              <Button
                variant="outlined"
                style={{ maxHeight: 30, padding: 0 }}
                onClick={() => {
                  setOpenClose(true)
                  setCloseSymbol(value.marketSymbol)
                  setCloseType('MARKET')
                }}
              >
                Market
              </Button>
            </Grid>
          </Grid>
        ),
      },
    ],
    [settingsState],
  )
  const isLoadingMarketData = false
  // const { data: selectedMarketData } = useSelectMarketDataType(type,  {refetchInterval: 1000})
  // TODO - why do I need to wrap in || [] for useSelectMarketDataType result?
  const data = useMemo(() => userData?.positions || [], [userData?.positions])

  const {
    getTableProps,
    headerGroups,
    prepareRow,
    page,
    canNextPage,
    canPreviousPage,
    pageCount,
    nextPage,
    previousPage,
    gotoPage,
    state: { pageIndex },
  } = useTable(
    {
      columns,
      data,
      initialState: { pageSize: 5 },
    },
    usePagination,
  )
  const onRowClick = (
    event: React.MouseEvent<HTMLTableCellElement, MouseEvent>,
    row: Row<PositionData>,
    column: string,
  ) => {
    return
  }

  return (
    <>
      <CloseModal
        openModal={openClose}
        setOpenModal={setOpenClose}
        marketSymbol={closeSymbol}
        closeType={closeType}
      />
      <Card sx={{ backgroundColor: 'background.component' }}>
        <TableContainer>
          <Table stickyHeader aria-label="sticky table" {...getTableProps()}>
            <TableHead sx={{ backgroundColor: 'background.component' }}>
              {headerGroups.map((headerGroup) => {
                const { key: headerGroupKey, ...getHeaderGroupProps } =
                  headerGroup.getHeaderGroupProps()
                return (
                  <TableRow key={headerGroupKey} {...getHeaderGroupProps}>
                    {headerGroup.headers.map((column) => {
                      const { key: headerKey, ...getHeaderProps } =
                        column.getHeaderProps()
                      return (
                        <TableCell
                          sx={{ backgroundColor: 'background.component' }}
                          key={headerKey}
                          {...getHeaderProps}
                        >
                          <Grid
                            container
                            direction="row"
                            alignItems="center"
                            sx={{ mt: -1, mb: -1 }}
                          >
                            <Grid item>
                              <Typography variant="h7" color="textSecondary">
                                {column.render('Header')}
                              </Typography>
                            </Grid>
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
                page.map((row: Row<PositionData>) => {
                  prepareRow(row)
                  const { key: rowKey, ...rowProps } = row.getRowProps()
                  return (
                    <TableRow
                      key={rowKey}
                      {...rowProps}
                      //style={{ cursor: 'pointer' }}
                      //hover
                    >
                      {row.cells.map((cell: Cell<PositionData>) => {
                        const { key: cellKey, ...cellProps } =
                          cell.getCellProps()
                        return (
                          <TableCell
                            sx={{ backgroundColor: 'background.component' }}
                            key={cellKey}
                            {...cellProps}
                            onClick={(event) => {
                              onRowClick(
                                event,
                                row.original,
                                cell.column.Header,
                              )
                            }}
                          >
                            <Grid sx={{ mt: -0.5, mb: -0.5 }}>
                              {cell.render('Cell')}
                            </Grid>
                          </TableCell>
                        )
                      })}
                    </TableRow>
                  )
                })
              )}
            </TableBody>
          </Table>
          {pageCount > 1 && (
            <PaginationFooter
              page={pageIndex + 1}
              gotoPage={gotoPage}
              nextPage={nextPage}
              previousPage={previousPage}
              canNextPage={canNextPage}
              canPreviousPage={canPreviousPage}
              pageCount={pageCount}
            />
          )}
        </TableContainer>
      </Card>
    </>
  )
}

export default Positions
